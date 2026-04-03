use std::fmt;
use std::str::from_utf8_unchecked;

mod iter;
#[cfg(feature = "serde")]
mod with_serde;

pub use iter::StrStackIter;

/// Error returned by [`StrStack::try_push`] when the total byte length would exceed `u32::MAX`.
#[derive(Debug, Clone)]
pub struct StrStackOverflow {
    attempted: usize,
}

impl fmt::Display for StrStackOverflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StrStack overflow: attempted total byte length {} exceeds u32::MAX",
            self.attempted
        )
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct StrStack {
    data: Vec<u8>,
    ends: Vec<u32>,
}

impl StrStack {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `StrStack` with pre-allocated capacity for `items` string segments
    /// and `bytes` total bytes of string data.
    #[inline]
    pub fn with_capacity(items: usize, bytes: usize) -> Self {
        Self {
            data: Vec::with_capacity(bytes),
            ends: Vec::with_capacity(items),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.ends.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ends.is_empty()
    }

    /// Returns the total byte length of the data buffer.
    ///
    /// Returns `u32` to match the internal boundary representation,
    /// making the 4 GB limit explicit at the call site.
    #[inline]
    pub fn bytes_len(&self) -> u32 {
        // `push` guarantees `self.data.len() <= u32::MAX`, so this cast is safe.
        self.data.len() as u32
    }

    /// Reserves capacity for at least `additional` more string segments.
    #[inline]
    pub fn reserve_items(&mut self, additional: usize) {
        self.ends.reserve(additional);
    }

    /// Reserves capacity for at least `additional` more bytes of string data.
    #[inline]
    pub fn reserve_bytes(&mut self, additional: usize) {
        self.data.reserve(additional);
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
        Some(unsafe { self.get_unchecked_internal(begin as usize, end as usize) })
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
    pub fn get_bounds(&self, index: usize) -> Option<(u32, u32)> {
        if index + 1 > self.ends.len() {
            return None;
        }
        let (start, end) = if index > 0 {
            (self.ends[index - 1], self.ends[index])
        } else {
            (0, self.ends[0])
        };
        debug_assert!(start <= end);
        debug_assert!((end as usize) <= self.data.len());
        Some((start, end))
    }

    #[inline]
    pub fn get_top(&self) -> Option<&str> {
        match self.ends.len() {
            0 => None,
            len => self.get(len - 1),
        }
    }

    /// Returns the last (topmost) string segment, or `None` if empty.
    ///
    /// Alias for [`get_top`](Self::get_top).
    #[inline]
    pub fn last(&self) -> Option<&str> {
        self.get_top()
    }

    #[inline]
    pub fn remove_top(&mut self) -> Option<()> {
        self.ends.pop()?;
        let end = self.ends.last().copied().unwrap_or(0) as usize;
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
        let new_end: u32 = self
            .data
            .len()
            .try_into()
            .expect("StrStack: total byte length exceeds u32::MAX");
        self.ends.push(new_end);
    }

    /// Fallible push: appends a string segment, returning `Err` if the total byte
    /// length would exceed `u32::MAX`.
    #[inline]
    pub fn try_push(&mut self, s: &str) -> Result<(), StrStackOverflow> {
        let new_len = self.data.len() + s.len();
        let new_end: u32 = new_len
            .try_into()
            .map_err(|_| StrStackOverflow { attempted: new_len })?;
        self.data.extend_from_slice(s.as_bytes());
        self.ends.push(new_end);
        Ok(())
    }

    /// Removes all string segments, resetting the stack to empty.
    ///
    /// Does not release allocated memory (use `shrink_to_fit` on the underlying
    /// vecs if needed — not yet exposed).
    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
        self.ends.clear();
    }

    /// Keeps the first `len` string segments and removes the rest.
    ///
    /// If `len >= self.len()`, this is a no-op (matches [`Vec::truncate`] semantics).
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len >= self.ends.len() {
            return;
        }
        self.ends.truncate(len);
        let byte_end = self.ends.last().copied().unwrap_or(0) as usize;
        self.data.truncate(byte_end);
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
        assert_eq!(stack.get_bounds(0), Some((0u32, 3u32)));
        assert_eq!(stack.get(1), None);
        assert_eq!(stack.get_bounds(1), None);

        stack.push("456");
        assert_eq!(stack.len(), 2);
        assert!(!stack.is_empty());
        assert_eq!(stack.get_top(), Some("456"));
        assert_eq!(stack.get(0), Some("123"));
        assert_eq!(stack.get_bounds(0), Some((0u32, 3u32)));
        assert_eq!(stack.get(1), Some("456"));
        assert_eq!(stack.get_bounds(1), Some((3u32, 6u32)));
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

        assert_eq!(stack.get_bounds(0), Some((0u32, 3u32)));
        assert_eq!(stack.get_bounds(1), Some((3u32, 4u32)));
        assert_eq!(stack.get_bounds(2), Some((4u32, 8u32)));
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

    // -- ergonomics APIs (v0.3) -----------------------------------------------------------------------

    #[test]
    fn test_with_capacity() {
        let stack = StrStack::with_capacity(10, 100);
        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_bytes_len() {
        let mut stack = StrStack::new();
        assert_eq!(stack.bytes_len(), 0);
        stack.push("abc"); // 3 bytes
        assert_eq!(stack.bytes_len(), 3);
        stack.push("€"); // 3 bytes
        assert_eq!(stack.bytes_len(), 6);
        stack.push("😊"); // 4 bytes
        assert_eq!(stack.bytes_len(), 10);
    }

    #[test]
    fn test_last() {
        let mut stack = StrStack::new();
        assert_eq!(stack.last(), None);
        stack.push("first");
        assert_eq!(stack.last(), Some("first"));
        stack.push("second");
        assert_eq!(stack.last(), Some("second"));
        // last() and get_top() return the same value
        assert_eq!(stack.last(), stack.get_top());
    }

    #[test]
    fn test_clear_public() {
        let mut stack = StrStack::new();
        stack.push("hello");
        stack.push("world");
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.bytes_len(), 10);

        stack.clear();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert_eq!(stack.bytes_len(), 0);
        assert_eq!(stack.as_str(), "");

        // Can push again after clear
        stack.push("new");
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.get(0), Some("new"));
    }

    #[test]
    fn test_truncate() {
        let mut stack = StrStack::new();
        stack.push("aaa");
        stack.push("bbb");
        stack.push("ccc");
        stack.push("ddd");
        assert_eq!(stack.len(), 4);

        // Truncate to 2 items
        stack.truncate(2);
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.get(0), Some("aaa"));
        assert_eq!(stack.get(1), Some("bbb"));
        assert_eq!(stack.get(2), None);
        assert_eq!(stack.as_str(), "aaabbb");
    }

    #[test]
    fn test_truncate_noop() {
        let mut stack = StrStack::new();
        stack.push("aaa");
        stack.push("bbb");

        // Truncate to >= len is a no-op
        stack.truncate(5);
        assert_eq!(stack.len(), 2);
        stack.truncate(2);
        assert_eq!(stack.len(), 2);
    }

    #[test]
    fn test_truncate_to_zero() {
        let mut stack = StrStack::new();
        stack.push("aaa");
        stack.push("bbb");

        stack.truncate(0);
        assert!(stack.is_empty());
        assert_eq!(stack.as_str(), "");
    }

    #[test]
    fn test_truncate_unicode() {
        let mut stack = StrStack::new();
        stack.push("你好"); // 6 bytes
        stack.push("世界"); // 6 bytes
        stack.push("🦀"); // 4 bytes

        stack.truncate(1);
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.get(0), Some("你好"));
        assert_eq!(stack.bytes_len(), 6);
    }

    #[test]
    fn test_try_push_success() {
        let mut stack = StrStack::new();
        assert!(stack.try_push("hello").is_ok());
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.get(0), Some("hello"));
    }

    #[test]
    fn test_reserve() {
        let mut stack = StrStack::new();
        stack.reserve_items(10);
        stack.reserve_bytes(100);
        // Reserving doesn't change length
        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
        // But we can push without reallocation
        for i in 0..10 {
            stack.push(&format!("{}", i));
        }
        assert_eq!(stack.len(), 10);
    }
}
