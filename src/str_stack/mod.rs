use std::str::from_utf8_unchecked;

mod iter;

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
        unsafe { from_utf8_unchecked(&self.data) }
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&str> {
        let (begin, end) = self.get_bounds(index)?;
        Some(unsafe { self.get_unchecked(begin, end) })
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, begin: usize, end: usize) -> &str {
        let slice = unsafe { self.data.get_unchecked(begin..end) };
        unsafe { from_utf8_unchecked(slice) }
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
        let end = self.ends.pop()?;
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
    pub fn iter(&self) -> StrStackIter {
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
}
