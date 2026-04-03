use std::fmt;
use std::str::from_utf8_unchecked;

use super::str_list::StrListIter;
use super::StrStack;

/// Error returned by [`StrListRef::new`] when the input data/ends are invalid.
#[derive(Debug, Clone)]
pub enum StrListValidationError {
    /// A boundary value exceeds the data length.
    BoundaryOutOfRange {
        index: usize,
        value: u32,
        data_len: usize,
    },
    /// Boundary values are not monotonically non-decreasing.
    BoundaryNotMonotonic {
        index: usize,
        prev: u32,
        current: u32,
    },
    /// A segment between two boundaries is not valid UTF-8.
    InvalidUtf8 { index: usize, start: u32, end: u32 },
}

impl fmt::Display for StrListValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BoundaryOutOfRange {
                index,
                value,
                data_len,
            } => {
                write!(
                    f,
                    "boundary out of range at index {}: value {} exceeds data length {}",
                    index, value, data_len
                )
            }
            Self::BoundaryNotMonotonic {
                index,
                prev,
                current,
            } => {
                write!(
                    f,
                    "boundaries not monotonic at index {}: {} > {}",
                    index, prev, current
                )
            }
            Self::InvalidUtf8 { index, start, end } => {
                write!(
                    f,
                    "invalid UTF-8 in segment {} (bytes {}..{})",
                    index, start, end
                )
            }
        }
    }
}

/// A borrowed, read-only view over string list data.
///
/// `StrListRef` borrows `&[u8]` (data) and `&[u32]` (boundary table) and provides
/// the same read-only API as [`StrList`](super::str_list::StrList). Use it for
/// zero-copy views over external buffers (e.g., memory-mapped files).
///
/// # Invariants
///
/// Same as `StrList`: data is valid UTF-8 and ends are valid boundaries.
/// When constructed via [`new`](Self::new), these invariants are validated.
/// When constructed via `From<&StrStack>` or `From<&StrList>`, they are inherited.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StrListRef<'a> {
    data: &'a [u8],
    ends: &'a [u32],
}

impl<'a> StrListRef<'a> {
    /// Creates a `StrListRef` from raw data and boundary slices, validating
    /// UTF-8 and boundary consistency.
    ///
    /// Validation checks (O(total bytes)):
    /// - `ends` values are monotonically non-decreasing
    /// - Last value does not exceed `data.len()`
    /// - Each segment `data[ends[i-1]..ends[i]]` (with `ends[-1] = 0`) is valid UTF-8
    pub fn new(data: &'a [u8], ends: &'a [u32]) -> Result<Self, StrListValidationError> {
        let mut prev: u32 = 0;
        for (i, &end) in ends.iter().enumerate() {
            if end < prev {
                return Err(StrListValidationError::BoundaryNotMonotonic {
                    index: i,
                    prev,
                    current: end,
                });
            }
            if (end as usize) > data.len() {
                return Err(StrListValidationError::BoundaryOutOfRange {
                    index: i,
                    value: end,
                    data_len: data.len(),
                });
            }
            if std::str::from_utf8(&data[prev as usize..end as usize]).is_err() {
                return Err(StrListValidationError::InvalidUtf8 {
                    index: i,
                    start: prev,
                    end,
                });
            }
            prev = end;
        }
        Ok(Self { data, ends })
    }

    /// Creates a `StrListRef` from trusted internal data. No validation.
    ///
    /// Used by `From<&StrStack>`, `From<&StrList>`, and `StrList::as_ref()`.
    #[inline]
    pub(super) fn from_raw_parts(data: &'a [u8], ends: &'a [u32]) -> Self {
        Self { data, ends }
    }

    /// Returns the number of string segments.
    #[inline]
    pub fn len(&self) -> usize {
        self.ends.len()
    }

    /// Returns `true` if the list contains no segments.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ends.is_empty()
    }

    /// Returns the total byte length of the data buffer.
    #[inline]
    pub fn bytes_len(&self) -> u32 {
        self.data.len() as u32
    }

    /// Returns the concatenation of all segments as a single `&str`.
    #[inline]
    pub fn as_str(&self) -> &'a str {
        // SAFETY: data is always valid UTF-8 (validated in new() or inherited from StrStack/StrList).
        unsafe { from_utf8_unchecked(self.data) }
    }

    /// Returns the segment at `index`, or `None` if out of bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&'a str> {
        if index >= self.ends.len() {
            return None;
        }
        let start = if index > 0 {
            self.ends[index - 1] as usize
        } else {
            0
        };
        let end = self.ends[index] as usize;
        // SAFETY: bounds validated by ends; data is valid UTF-8.
        Some(unsafe { from_utf8_unchecked(self.data.get_unchecked(start..end)) })
    }

    /// Returns the byte offset bounds `(start, end)` for the segment at `index`.
    #[inline]
    pub fn get_bounds(&self, index: usize) -> Option<(u32, u32)> {
        if index >= self.ends.len() {
            return None;
        }
        let start = if index > 0 { self.ends[index - 1] } else { 0 };
        Some((start, self.ends[index]))
    }

    /// Returns the last segment, or `None` if empty.
    #[inline]
    pub fn last(&self) -> Option<&'a str> {
        if self.ends.is_empty() {
            None
        } else {
            self.get(self.ends.len() - 1)
        }
    }

    /// Returns an iterator over the string segments.
    #[inline]
    pub fn iter(&self) -> StrListIter<'a> {
        StrListIter {
            data: self.data,
            ends: self.ends,
            index: 0,
            back_index: self.ends.len(),
        }
    }
}

impl<'a> From<&'a StrStack> for StrListRef<'a> {
    #[inline]
    fn from(stack: &'a StrStack) -> Self {
        Self::from_raw_parts(stack.data_as_slice(), stack.ends_as_slice())
    }
}

impl<'a> From<&'a super::str_list::StrList> for StrListRef<'a> {
    #[inline]
    fn from(list: &'a super::str_list::StrList) -> Self {
        list.as_ref()
    }
}

impl<'a> IntoIterator for StrListRef<'a> {
    type Item = &'a str;
    type IntoIter = StrListIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> fmt::Debug for StrListRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_stack() {
        let mut stack = StrStack::new();
        stack.push("hello");
        stack.push("world");
        let view = StrListRef::from(&stack);
        assert_eq!(view.len(), 2);
        assert_eq!(view.get(0), Some("hello"));
        assert_eq!(view.get(1), Some("world"));
        assert_eq!(view.as_str(), "helloworld");
    }

    #[test]
    fn test_new_valid() {
        let data = b"helloworld";
        let ends = [5u32, 10u32];
        let view = StrListRef::new(data, &ends).unwrap();
        assert_eq!(view.len(), 2);
        assert_eq!(view.get(0), Some("hello"));
        assert_eq!(view.get(1), Some("world"));
    }

    #[test]
    fn test_new_empty() {
        let view = StrListRef::new(b"", &[]).unwrap();
        assert!(view.is_empty());
        assert_eq!(view.as_str(), "");
    }

    #[test]
    fn test_new_invalid_boundary_out_of_range() {
        let data = b"hello";
        let ends = [10u32]; // 10 > 5
        let err = StrListRef::new(data, &ends).unwrap_err();
        assert!(matches!(
            err,
            StrListValidationError::BoundaryOutOfRange { .. }
        ));
    }

    #[test]
    fn test_new_invalid_not_monotonic() {
        let data = b"helloworld";
        let ends = [5u32, 3u32]; // 3 < 5
        let err = StrListRef::new(data, &ends).unwrap_err();
        assert!(matches!(
            err,
            StrListValidationError::BoundaryNotMonotonic { .. }
        ));
    }

    #[test]
    fn test_new_invalid_utf8() {
        let data = &[0xff, 0xfe, 0xfd];
        let ends = [3u32];
        let err = StrListRef::new(data, &ends).unwrap_err();
        assert!(matches!(err, StrListValidationError::InvalidUtf8 { .. }));
    }

    #[test]
    fn test_new_unicode() {
        let data = "你好😊".as_bytes();
        let ends = [6u32, 10u32]; // 你好 = 6 bytes, 😊 = 4 bytes
        let view = StrListRef::new(data, &ends).unwrap();
        assert_eq!(view.get(0), Some("你好"));
        assert_eq!(view.get(1), Some("😊"));
    }

    #[test]
    fn test_get_bounds() {
        let data = b"abcdef";
        let ends = [3u32, 6u32];
        let view = StrListRef::new(data, &ends).unwrap();
        assert_eq!(view.get_bounds(0), Some((0, 3)));
        assert_eq!(view.get_bounds(1), Some((3, 6)));
        assert_eq!(view.get_bounds(2), None);
    }

    #[test]
    fn test_last() {
        let data = b"abc";
        let ends = [1u32, 2u32, 3u32];
        let view = StrListRef::new(data, &ends).unwrap();
        assert_eq!(view.last(), Some("c"));
    }

    #[test]
    fn test_iter() {
        let data = b"abcdef";
        let ends = [2u32, 4u32, 6u32];
        let view = StrListRef::new(data, &ends).unwrap();
        let collected: Vec<&str> = view.iter().collect();
        assert_eq!(collected, vec!["ab", "cd", "ef"]);
    }

    #[test]
    fn test_iter_rev() {
        let data = b"abcdef";
        let ends = [2u32, 4u32, 6u32];
        let view = StrListRef::new(data, &ends).unwrap();
        let collected: Vec<&str> = view.iter().rev().collect();
        assert_eq!(collected, vec!["ef", "cd", "ab"]);
    }

    #[test]
    fn test_empty_segments() {
        let data = b"abc";
        let ends = [0u32, 3u32, 3u32]; // "", "abc", ""
        let view = StrListRef::new(data, &ends).unwrap();
        assert_eq!(view.len(), 3);
        assert_eq!(view.get(0), Some(""));
        assert_eq!(view.get(1), Some("abc"));
        assert_eq!(view.get(2), Some(""));
    }

    #[test]
    fn test_copy() {
        let data = b"hello";
        let ends = [5u32];
        let view = StrListRef::new(data, &ends).unwrap();
        let view2 = view; // Copy
        assert_eq!(view, view2);
    }

    #[test]
    fn test_debug() {
        let data = b"helloworld";
        let ends = [5u32, 10u32];
        let view = StrListRef::new(data, &ends).unwrap();
        let debug = format!("{:?}", view);
        assert_eq!(debug, r#"["hello", "world"]"#);
    }
}
