use crate::StrStack;

/// Iterator over [`StrStack`] segments.
pub struct StrStackIter<'a> {
    stack: &'a StrStack,
    index: usize,
    back_index: usize,
}

impl<'a> StrStackIter<'a> {
    #[inline]
    pub fn new(stack: &'a StrStack) -> Self {
        Self {
            back_index: stack.ends_as_slice().len(),
            stack,
            index: 0,
        }
    }

    #[inline]
    fn bounds(&self, index: usize) -> (usize, usize) {
        let ends = self.stack.ends_as_slice();
        let start = if index > 0 {
            ends[index - 1] as usize
        } else {
            0
        };
        let end = ends[index] as usize;
        (start, end)
    }
}

impl<'a> Iterator for StrStackIter<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.back_index {
            return None;
        }
        let (begin, end) = self.bounds(self.index);
        self.index += 1;
        // SAFETY: `StrStackIter` is constructed from a valid `StrStack` and advances using `ends` boundaries.
        // `StrStack` only stores UTF-8 segments pushed via `push(&str)`, so `[begin..end]` is in-bounds and valid UTF-8.
        Some(unsafe { self.stack.get_unchecked_internal(begin, end) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a> DoubleEndedIterator for StrStackIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back_index <= self.index {
            return None;
        }
        self.back_index -= 1;
        let (begin, end) = self.bounds(self.back_index);
        // SAFETY: same as `next()` — bounds from `ends` are valid UTF-8 segment boundaries.
        Some(unsafe { self.stack.get_unchecked_internal(begin, end) })
    }
}

impl<'a> ExactSizeIterator for StrStackIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.back_index - self.index
    }
}

impl<'a> std::iter::FusedIterator for StrStackIter<'a> {}

impl<'a> IntoIterator for &'a StrStack {
    type Item = <StrStackIter<'a> as Iterator>::Item;
    type IntoIter = StrStackIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        StrStackIter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter() {
        let mut stack = StrStack::new();

        stack.push("123");
        stack.push("456");
        stack.push("789");

        let mut iter = StrStackIter::new(&stack);
        assert_eq!(iter.next(), Some("123"));
        assert_eq!(iter.next(), Some("456"));
        assert_eq!(iter.next(), Some("789"));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_empty() {
        let stack = StrStack::new();

        let mut iter = StrStackIter::new(&stack);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_exact_size_len_and_size_hint_decrease() {
        let mut stack = StrStack::new();
        stack.push("a");
        stack.push("bb");
        stack.push("ccc");

        let mut it = stack.iter();
        assert_eq!(it.len(), 3);
        assert_eq!(it.size_hint(), (3, Some(3)));

        assert_eq!(it.next(), Some("a"));
        assert_eq!(it.len(), 2);
        assert_eq!(it.size_hint(), (2, Some(2)));

        assert_eq!(it.next(), Some("bb"));
        assert_eq!(it.len(), 1);
        assert_eq!(it.size_hint(), (1, Some(1)));

        assert_eq!(it.next(), Some("ccc"));
        assert_eq!(it.len(), 0);
        assert_eq!(it.size_hint(), (0, Some(0)));

        assert_eq!(it.next(), None);
        assert_eq!(it.len(), 0);
        assert_eq!(it.size_hint(), (0, Some(0)));
    }

    #[test]
    fn test_iter_reverse() {
        let mut stack = StrStack::new();
        stack.push("a");
        stack.push("b");
        stack.push("c");

        let collected: Vec<&str> = stack.iter().rev().collect();
        assert_eq!(collected, vec!["c", "b", "a"]);
    }

    #[test]
    fn test_iter_double_ended_meet_in_middle() {
        let mut stack = StrStack::new();
        stack.push("a");
        stack.push("b");
        stack.push("c");
        stack.push("d");

        let mut it = stack.iter();
        assert_eq!(it.next(), Some("a"));
        assert_eq!(it.next_back(), Some("d"));
        assert_eq!(it.next(), Some("b"));
        assert_eq!(it.next_back(), Some("c"));
        assert_eq!(it.next(), None);
        assert_eq!(it.next_back(), None);
    }

    #[test]
    fn test_iter_fused() {
        let mut stack = StrStack::new();
        stack.push("x");

        let mut it = stack.iter();
        assert_eq!(it.next(), Some("x"));
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);
    }
}
