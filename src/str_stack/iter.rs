use crate::StrStack;

#[derive(Clone, Copy, Debug)]
struct Cursor {
    index: usize,
    begin: usize,
    end: usize,
}

impl Cursor {
    #[inline]
    pub fn new(index: usize, begin: usize, end: usize) -> Self {
        Self { index, begin, end }
    }
}

pub struct StrStackIter<'a> {
    stack: &'a StrStack,
    next: Option<Cursor>,
}

impl<'a> StrStackIter<'a> {
    #[inline]
    pub fn new(stack: &'a StrStack) -> Self {
        let next = stack
            .ends
            .first()
            .copied()
            .map(|end| Cursor::new(0, 0, end));
        Self { stack, next }
    }
}

impl<'a> Iterator for StrStackIter<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Cursor { index, begin, end } = self.next?;
        let next_index = index + 1;
        self.next = self
            .stack
            .ends
            .get(next_index)
            .copied()
            .map(|next_end| Cursor::new(next_index, end, next_end));
        Some(unsafe { self.stack.get_unchecked(begin, end) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for StrStackIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.next
            .map(|c| self.stack.ends.len() - c.index)
            .unwrap_or(0)
    }
}

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
}
