use std::fmt;
use std::iter::FusedIterator;

use crate::PascalString;

// -- Inner storage --------------------------------------------------------------------------------

enum IntoCharsInner<const N: usize> {
    Stack(PascalString<N>),
    Heap(String),
}

impl<const N: usize> IntoCharsInner<N> {
    #[inline]
    fn as_str(&self) -> &str {
        match self {
            Self::Stack(s) => s.as_str(),
            Self::Heap(s) => s.as_str(),
        }
    }
}

impl<const N: usize> Clone for IntoCharsInner<N> {
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Self::Stack(s) => Self::Stack(*s),
            Self::Heap(s) => Self::Heap(s.clone()),
        }
    }
}

// -- IntoChars ------------------------------------------------------------------------------------

/// A by-value iterator over the `char`s of a [`SmartString`].
///
/// This struct is created by [`SmartString::into_chars`].  It supports both
/// forward and backward traversal ([`DoubleEndedIterator`]), exact remaining-char
/// counting ([`ExactSizeIterator`]), and fused behaviour ([`FusedIterator`]).
///
/// [`SmartString`]: crate::SmartString
/// [`SmartString::into_chars`]: crate::SmartString::into_chars
pub struct IntoChars<const N: usize> {
    inner: IntoCharsInner<N>,
    /// Byte index of the next char to yield from the front.
    front: usize,
    /// Byte index (exclusive) of the next char to yield from the back.
    back: usize,
}

impl<const N: usize> IntoChars<N> {
    #[inline]
    pub(crate) fn new_stack(s: PascalString<N>) -> Self {
        let back = s.len();
        Self {
            inner: IntoCharsInner::Stack(s),
            front: 0,
            back,
        }
    }

    #[inline]
    pub(crate) fn new_heap(s: String) -> Self {
        let back = s.len();
        Self {
            inner: IntoCharsInner::Heap(s),
            front: 0,
            back,
        }
    }

    /// Returns the remaining (not yet consumed) portion of the string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.inner.as_str()[self.front..self.back]
    }

    /// Consumes the iterator and returns the remaining (not yet consumed) portion as a `String`.
    #[inline]
    #[must_use]
    pub fn into_string(self) -> String {
        self.as_str().to_owned()
    }
}

impl<const N: usize> Clone for IntoChars<N> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            front: self.front,
            back: self.back,
        }
    }
}

impl<const N: usize> fmt::Debug for IntoChars<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntoChars")
            .field("remaining", &self.as_str())
            .finish()
    }
}

impl<const N: usize> fmt::Display for IntoChars<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// -- Iterator -------------------------------------------------------------------------------------

impl<const N: usize> Iterator for IntoChars<N> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            return None;
        }
        let s = self.as_str();
        // SAFETY: `as_str()` returns a valid UTF-8 `&str`; `chars().next()` on a non-empty str
        // always returns `Some`.
        let ch = s.chars().next().expect("non-empty str has first char");
        self.front += ch.len_utf8();
        Some(ch)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<const N: usize> DoubleEndedIterator for IntoChars<N> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            return None;
        }
        // Walk backward from `back` to find the start of the last char.
        // Continuation bytes have the pattern `10xxxxxx` (i.e. `(b & 0xC0) == 0x80`).
        // We step back one byte at a time until we land on a leading byte.
        let bytes = self.inner.as_str().as_bytes();
        let mut pos = self.back - 1;
        while (bytes[pos] & 0xC0) == 0x80 {
            pos -= 1;
        }
        // `pos` is now the start of the last char.
        // SAFETY: `pos..self.back` is a valid UTF-8 scalar value within the string.
        let ch_str = &self.inner.as_str()[pos..self.back];
        let ch = ch_str.chars().next().expect("non-empty char slice");
        self.back = pos;
        Some(ch)
    }
}

impl<const N: usize> ExactSizeIterator for IntoChars<N> {
    #[inline]
    fn len(&self) -> usize {
        self.as_str().chars().count()
    }
}

impl<const N: usize> FusedIterator for IntoChars<N> {}

// -- Tests ----------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SmartString;

    // Helper: create a stack-backed IntoChars with a string that fits within 30 bytes.
    fn stack_iter(s: &str) -> IntoChars<30> {
        SmartString::<30>::from(s).into_chars()
    }

    // Helper: create a heap-backed IntoChars with a string longer than 30 bytes.
    fn heap_iter(s: &str) -> IntoChars<30> {
        SmartString::<30>::from(s).into_chars()
    }

    #[test]
    fn test_stack_forward() {
        let chars: Vec<char> = stack_iter("hello").collect();
        assert_eq!(chars, vec!['h', 'e', 'l', 'l', 'o']);
    }

    #[test]
    fn test_heap_forward() {
        // 32 'a's — exceeds default stack capacity of 30.
        let s = "a".repeat(32);
        let chars: Vec<char> = heap_iter(&s).collect();
        assert_eq!(chars, vec!['a'; 32]);
    }

    #[test]
    fn test_empty() {
        let mut it = stack_iter("");
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_multibyte_emoji() {
        let chars: Vec<char> = stack_iter("🦀ab").collect();
        assert_eq!(chars, vec!['🦀', 'a', 'b']);
    }

    #[test]
    fn test_multibyte_cjk() {
        let chars: Vec<char> = stack_iter("你好").collect();
        assert_eq!(chars, vec!['你', '好']);
    }

    #[test]
    fn test_double_ended_reversed() {
        let chars: Vec<char> = stack_iter("abc").rev().collect();
        assert_eq!(chars, vec!['c', 'b', 'a']);
    }

    #[test]
    fn test_double_ended_multibyte_reversed() {
        let chars: Vec<char> = stack_iter("🦀你好").rev().collect();
        assert_eq!(chars, vec!['好', '你', '🦀']);
    }

    #[test]
    fn test_mixed_forward_backward() {
        let mut it = stack_iter("abcde");
        assert_eq!(it.next(), Some('a'));
        assert_eq!(it.next_back(), Some('e'));
        assert_eq!(it.next(), Some('b'));
        assert_eq!(it.next_back(), Some('d'));
        assert_eq!(it.next(), Some('c'));
        assert_eq!(it.next(), None);
        assert_eq!(it.next_back(), None);
    }

    #[test]
    fn test_exact_size_len() {
        let mut it = stack_iter("abc");
        assert_eq!(it.len(), 3);
        it.next();
        assert_eq!(it.len(), 2);
        it.next();
        assert_eq!(it.len(), 1);
        it.next();
        assert_eq!(it.len(), 0);
        it.next();
        assert_eq!(it.len(), 0);
    }

    #[test]
    fn test_exact_size_multibyte() {
        let mut it = stack_iter("🦀你好");
        assert_eq!(it.len(), 3);
        it.next();
        assert_eq!(it.len(), 2);
    }

    #[test]
    fn test_as_str_partial() {
        let mut it = stack_iter("hello");
        it.next(); // consume 'h'
        assert_eq!(it.as_str(), "ello");
        it.next_back(); // consume 'o'
        assert_eq!(it.as_str(), "ell");
    }

    #[test]
    fn test_into_string_partial() {
        let mut it = stack_iter("hello");
        it.next(); // consume 'h'
        it.next_back(); // consume 'o'
        assert_eq!(it.into_string(), "ell");
    }

    #[test]
    fn test_clone_independent() {
        let mut it = stack_iter("abc");
        let mut clone = it.clone();

        assert_eq!(it.next(), Some('a'));
        // Clone is independent — still at 'a'.
        assert_eq!(clone.next(), Some('a'));
        assert_eq!(clone.next(), Some('b'));
        // Original is still only past 'a'.
        assert_eq!(it.next(), Some('b'));
    }

    #[test]
    fn test_fused_returns_none_after_exhaustion() {
        let mut it = stack_iter("x");
        assert_eq!(it.next(), Some('x'));
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);
        assert_eq!(it.next_back(), None);
        assert_eq!(it.next_back(), None);
    }

    #[test]
    fn test_size_hint() {
        let it = stack_iter("abc");
        assert_eq!(it.size_hint(), (3, Some(3)));
    }
}
