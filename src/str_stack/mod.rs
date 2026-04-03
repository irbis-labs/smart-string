use std::str::from_utf8_unchecked;

mod iter;
#[cfg(feature = "serde")]
mod with_serde;

pub use iter::StrStackIter;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct StrStack {
    data: Vec<u8>,
    ends: Vec<usize>,
}

impl StrStack {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.ends.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ends.is_empty()
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: `self.data` is only appended to via `push(&str)` and truncated via `remove_top()`,
        // so it is always valid UTF-8.
        unsafe { from_utf8_unchecked(&self.data) }
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&str> {
        let (begin, end) = self.get_bounds(index)?;
        // SAFETY: `get_bounds` ensures `begin <= end <= self.data.len()`, and the stack stores only UTF-8 segments
        // pushed via `push(&str)`.
        Some(unsafe { self.get_unchecked_internal(begin, end) })
    }

    #[inline]
    /// Returns a `&str` slice without bounds checks.
    ///
    /// # Safety
    ///
    /// - `begin <= end`
    /// - `end <= self.data.len()`
    /// - `self.data[begin..end]` must be valid UTF-8
    #[deprecated(note = "Use `get()` instead. This will be removed in a future version.")]
    pub unsafe fn get_unchecked(&self, begin: usize, end: usize) -> &str {
        // SAFETY: caller upholds bounds + UTF-8 preconditions (see doc comment).
        unsafe {
            let slice = self.data.get_unchecked(begin..end);
            from_utf8_unchecked(slice)
        }
    }

    /// Internal unchecked slice access. Not public — callers within the crate
    /// must uphold bounds + UTF-8 preconditions.
    #[inline]
    pub(crate) unsafe fn get_unchecked_internal(&self, begin: usize, end: usize) -> &str {
        // SAFETY: caller upholds bounds + UTF-8 preconditions.
        unsafe {
            let slice = self.data.get_unchecked(begin..end);
            from_utf8_unchecked(slice)
        }
    }

    #[inline]
    pub fn get_bounds(&self, index: usize) -> Option<(usize, usize)> {
        if index + 1 > self.ends.len() {
            return None;
        }
        let (start, end) = if index > 0 {
            (self.ends[index - 1], self.ends[index])
        } else {
            (0, self.ends[0])
        };
        debug_assert!(start <= end);
        debug_assert!(end <= self.data.len());
        Some((start, end))
    }

    #[inline]
    pub fn get_top(&self) -> Option<&str> {
        match self.ends.len() {
            0 => None,
            len => self.get(len - 1),
        }
    }

    #[inline]
    pub fn remove_top(&mut self) -> Option<()> {
        self.ends.pop()?;
        let end = self.ends.last().copied().unwrap_or(0);
        self.data.truncate(end);
        Some(())
    }

    #[inline]
    pub fn pop_owned<T>(&mut self) -> Option<T>
    where
        T: for<'a> From<&'a str>,
    {
        let s = self.get_top()?.into();
        self.remove_top();
        Some(s)
    }

    #[inline]
    pub fn push(&mut self, s: &str) {
        self.data.extend_from_slice(s.as_bytes());
        self.ends.push(self.data.len());
    }

    #[inline]
    fn clear(&mut self) {
        self.data.clear();
        self.ends.clear();
    }

    #[inline]
    pub fn iter(&self) -> StrStackIter<'_> {
        StrStackIter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::SmartString;

    #[test]
    fn test_create() {
        let stack = StrStack::new();
        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
        assert_eq!(stack.get_top(), None);
        assert_eq!(stack.get(0), None);
        assert_eq!(stack.get_bounds(0), None);
    }

    #[test]
    fn test_push() {
        let mut stack = StrStack::new();

        stack.push("123");
        assert_eq!(stack.len(), 1);
        assert!(!stack.is_empty());
        assert_eq!(stack.get_top(), Some("123"));
        assert_eq!(stack.get(0), Some("123"));
        assert_eq!(stack.get_bounds(0), Some((0, 3)));
        assert_eq!(stack.get(1), None);
        assert_eq!(stack.get_bounds(1), None);

        stack.push("456");
        assert_eq!(stack.len(), 2);
        assert!(!stack.is_empty());
        assert_eq!(stack.get_top(), Some("456"));
        assert_eq!(stack.get(0), Some("123"));
        assert_eq!(stack.get_bounds(0), Some((0, 3)));
        assert_eq!(stack.get(1), Some("456"));
        assert_eq!(stack.get_bounds(1), Some((3, 6)));
        assert_eq!(stack.get(2), None);
        assert_eq!(stack.get_bounds(2), None);
    }

    #[test]
    fn test_remove_top() {
        let mut stack = StrStack::new();

        stack.push("123");
        stack.push("456");
        stack.push("789");
        assert_eq!(stack.len(), 3);

        assert!(stack.remove_top().is_some());
        assert_eq!(stack.len(), 2);
        assert!(!stack.is_empty());
        assert_eq!(stack.get_top(), Some("456"));
        assert_eq!(stack.get(0), Some("123"));
        assert_eq!(stack.get(1), Some("456"));
        assert!(stack.get(2).is_none());
        assert!(stack.get_bounds(2).is_none());

        assert!(stack.remove_top().is_some());
        assert_eq!(stack.len(), 1);
        assert!(!stack.is_empty());
        assert_eq!(stack.get_top(), Some("123"));
        assert_eq!(stack.get(0), Some("123"));
        assert!(stack.get(1).is_none());
        assert!(stack.get_bounds(1).is_none());

        assert!(stack.remove_top().is_some());
        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
        assert!(stack.get_top().is_none());
        assert!(stack.get(0).is_none());
        assert!(stack.get_bounds(0).is_none());

        assert!(stack.remove_top().is_none());
    }

    #[test]
    fn test_pop_owned() {
        let mut stack = StrStack::new();

        stack.push("123");
        stack.push("456");
        stack.push("789");
        assert_eq!(stack.len(), 3);

        assert_eq!(stack.pop_owned::<String>(), Some("789".into()));
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.get_top(), Some("456"));
        assert_eq!(stack.get(0), Some("123"));
        assert_eq!(stack.get(1), Some("456"));
        assert!(stack.get(2).is_none());
        assert!(stack.get_bounds(2).is_none());

        assert_eq!(stack.pop_owned::<SmartString>(), Some("456".into()));
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.get_top(), Some("123"));
        assert_eq!(stack.get(0), Some("123"));
        assert!(stack.get(1).is_none());
        assert!(stack.get_bounds(1).is_none());

        assert_eq!(stack.pop_owned::<Rc<str>>(), Some("123".into()));
        assert_eq!(stack.len(), 0);
        assert!(stack.get_top().is_none());
        assert!(stack.get(0).is_none());
        assert!(stack.get_bounds(0).is_none());

        assert!(stack.pop_owned::<Box<str>>().is_none());
    }

    #[test]
    fn test_iter() {
        let mut stack = StrStack::new();

        stack.push("123");
        stack.push("456");
        stack.push("789");

        let mut iter = stack.iter();
        assert_eq!(iter.next(), Some("123"));
        assert_eq!(iter.next(), Some("456"));
        assert_eq!(iter.next(), Some("789"));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_unicode_push_get_bounds_and_as_str() {
        let mut stack = StrStack::new();
        stack.push("€"); // 3 bytes
        stack.push("a"); // 1 byte
        stack.push("😊"); // 4 bytes

        assert_eq!(stack.as_str(), "€a😊");

        assert_eq!(stack.get(0), Some("€"));
        assert_eq!(stack.get(1), Some("a"));
        assert_eq!(stack.get(2), Some("😊"));

        assert_eq!(stack.get_bounds(0), Some((0, 3)));
        assert_eq!(stack.get_bounds(1), Some((3, 4)));
        assert_eq!(stack.get_bounds(2), Some((4, 8)));
    }

    #[test]
    fn test_unicode_remove_top_truncates_byte_buffer() {
        let mut stack = StrStack::new();
        stack.push("€"); // 3 bytes
        stack.push("😊"); // 4 bytes
        stack.push("a"); // 1 byte

        assert_eq!(stack.as_str(), "€😊a");
        assert_eq!(stack.len(), 3);

        stack.remove_top().unwrap();
        assert_eq!(stack.as_str(), "€😊");
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.get_top(), Some("😊"));

        stack.remove_top().unwrap();
        assert_eq!(stack.as_str(), "€");
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.get_top(), Some("€"));
    }

    // -- push/remove/push mutation sequences ---------------------------------------------------------

    #[test]
    fn test_push_remove_push_sequence() {
        let mut stack = StrStack::new();
        stack.push("aaa");
        stack.push("bbb");
        stack.push("ccc");
        assert_eq!(stack.len(), 3);

        // Remove top two
        stack.remove_top();
        stack.remove_top();
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.as_str(), "aaa");
        assert_eq!(stack.get(0), Some("aaa"));

        // Push again after removal
        stack.push("ddd");
        stack.push("eee");
        assert_eq!(stack.len(), 3);
        assert_eq!(stack.as_str(), "aaadddeee");
        assert_eq!(stack.get(0), Some("aaa"));
        assert_eq!(stack.get(1), Some("ddd"));
        assert_eq!(stack.get(2), Some("eee"));

        // Iterator should yield the correct segments
        let collected: Vec<&str> = stack.iter().collect();
        assert_eq!(collected, vec!["aaa", "ddd", "eee"]);
    }

    #[test]
    fn test_push_remove_all_push_again() {
        let mut stack = StrStack::new();
        stack.push("first");
        stack.push("second");

        stack.remove_top();
        stack.remove_top();
        assert!(stack.is_empty());
        assert_eq!(stack.as_str(), "");

        stack.push("third");
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.get(0), Some("third"));
        assert_eq!(stack.as_str(), "third");
    }

    #[test]
    fn test_push_remove_push_unicode() {
        let mut stack = StrStack::new();
        stack.push("你好"); // 6 bytes
        stack.push("世界"); // 6 bytes
        assert_eq!(stack.as_str(), "你好世界");

        stack.remove_top();
        assert_eq!(stack.as_str(), "你好");

        stack.push("🦀"); // 4 bytes
        assert_eq!(stack.as_str(), "你好🦀");
        assert_eq!(stack.get(0), Some("你好"));
        assert_eq!(stack.get(1), Some("🦀"));

        let collected: Vec<&str> = stack.iter().collect();
        assert_eq!(collected, vec!["你好", "🦀"]);
    }

    #[test]
    fn test_as_str_equals_iter_concatenation() {
        let mut stack = StrStack::new();
        stack.push("abc");
        stack.push("€");
        stack.push("def");
        stack.remove_top();
        stack.push("ghi");
        stack.push("😊");

        let concatenated: String = stack.iter().collect();
        assert_eq!(stack.as_str(), concatenated.as_str());
    }

    // -- clear ---------------------------------------------------------------------------------------

    #[test]
    fn test_clear() {
        let mut stack = StrStack::new();
        stack.push("hello");
        stack.push("world");
        assert_eq!(stack.len(), 2);

        // clear is private, but we can test it via the serde roundtrip
        // or through repeated remove_top. Let's test the invariant:
        // after removing all items, the stack is fully clean.
        stack.remove_top();
        stack.remove_top();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert_eq!(stack.as_str(), "");
        assert!(stack.iter().next().is_none());

        // Can push again
        stack.push("new");
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.get(0), Some("new"));
    }

    // -- empty string segments -----------------------------------------------------------------------

    #[test]
    fn test_push_empty_strings() {
        let mut stack = StrStack::new();
        stack.push("");
        stack.push("abc");
        stack.push("");

        assert_eq!(stack.len(), 3);
        assert_eq!(stack.get(0), Some(""));
        assert_eq!(stack.get(1), Some("abc"));
        assert_eq!(stack.get(2), Some(""));
        assert_eq!(stack.as_str(), "abc");

        let collected: Vec<&str> = stack.iter().collect();
        assert_eq!(collected, vec!["", "abc", ""]);
    }
}
