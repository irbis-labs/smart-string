use std::str::from_utf8_unchecked;

use super::str_list_ref::StrListRef;
use super::StrStack;

/// An immutable, compact representation of a list of string slices.
///
/// `StrList` is the "frozen" counterpart to [`StrStack`]. It stores the same data
/// (a contiguous UTF-8 byte buffer plus a `u32` boundary table) but uses boxed slices
/// instead of `Vec`s, shedding excess capacity.
///
/// Typically created by converting a finished [`StrStack`] via `From<StrStack>` or
/// by collecting an iterator of `&str`.
///
/// # Invariants
///
/// - `data` is always valid UTF-8.
/// - `ends` values are monotonically non-decreasing and the last value does not
///   exceed `data.len()`.
#[derive(Clone, PartialEq, Eq)]
pub struct StrList {
    data: Box<[u8]>,
    ends: Box<[u32]>,
}

impl StrList {
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
    pub fn as_str(&self) -> &str {
        // SAFETY: `data` is always valid UTF-8 (invariant from StrStack or FromIterator).
        unsafe { from_utf8_unchecked(&self.data) }
    }

    /// Returns the segment at `index`, or `None` if out of bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&str> {
        let (start, end) = self.get_bounds_usize(index)?;
        // SAFETY: bounds validated by `get_bounds_usize`; data is valid UTF-8.
        Some(unsafe { from_utf8_unchecked(self.data.get_unchecked(start..end)) })
    }

    /// Returns the byte offset bounds `(start, end)` for the segment at `index`.
    #[inline]
    pub fn get_bounds(&self, index: usize) -> Option<(u32, u32)> {
        if index >= self.ends.len() {
            return None;
        }
        let start = if index > 0 { self.ends[index - 1] } else { 0 };
        let end = self.ends[index];
        debug_assert!(start <= end);
        debug_assert!((end as usize) <= self.data.len());
        Some((start, end))
    }

    /// Returns the last segment, or `None` if empty.
    #[inline]
    pub fn last(&self) -> Option<&str> {
        if self.ends.is_empty() {
            None
        } else {
            self.get(self.ends.len() - 1)
        }
    }

    /// Returns a borrowed view over this list.
    #[inline]
    pub fn as_ref(&self) -> StrListRef<'_> {
        StrListRef::from_raw_parts(&self.data, &self.ends)
    }

    /// Returns an iterator over the string segments.
    #[inline]
    pub fn iter(&self) -> StrListIter<'_> {
        StrListIter {
            data: &self.data,
            ends: &self.ends,
            index: 0,
            back_index: self.ends.len(),
        }
    }

    #[inline]
    fn get_bounds_usize(&self, index: usize) -> Option<(usize, usize)> {
        if index >= self.ends.len() {
            return None;
        }
        let start = if index > 0 {
            self.ends[index - 1] as usize
        } else {
            0
        };
        let end = self.ends[index] as usize;
        Some((start, end))
    }
}

impl From<StrStack> for StrList {
    #[inline]
    fn from(stack: StrStack) -> Self {
        Self {
            data: stack.data_as_slice().to_vec().into_boxed_slice(),
            ends: stack.ends_as_slice().to_vec().into_boxed_slice(),
        }
    }
}

impl<'a> FromIterator<&'a str> for StrList {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let mut data = Vec::new();
        let mut ends = Vec::new();
        for s in iter {
            data.extend_from_slice(s.as_bytes());
            let end: u32 = data
                .len()
                .try_into()
                .expect("StrList: total byte length exceeds u32::MAX");
            ends.push(end);
        }
        Self {
            data: data.into_boxed_slice(),
            ends: ends.into_boxed_slice(),
        }
    }
}

impl<'a> IntoIterator for &'a StrList {
    type Item = &'a str;
    type IntoIter = StrListIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl std::fmt::Debug for StrList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for StrList {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for s in self.iter() {
            seq.serialize_element(s)?;
        }
        seq.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for StrList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize as Vec<String>, then collect into StrList
        let strings: Vec<String> = serde::Deserialize::deserialize(deserializer)?;
        Ok(strings.iter().map(|s| s.as_str()).collect())
    }
}

/// Iterator over `StrList` or `StrListRef` segments.
pub struct StrListIter<'a> {
    pub(super) data: &'a [u8],
    pub(super) ends: &'a [u32],
    pub(super) index: usize,
    pub(super) back_index: usize,
}

impl<'a> Iterator for StrListIter<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.back_index {
            return None;
        }
        let start = if self.index > 0 {
            self.ends[self.index - 1] as usize
        } else {
            0
        };
        let end = self.ends[self.index] as usize;
        self.index += 1;
        // SAFETY: bounds come from `ends` which are valid boundaries; data is valid UTF-8.
        Some(unsafe { from_utf8_unchecked(self.data.get_unchecked(start..end)) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.back_index - self.index;
        (len, Some(len))
    }
}

impl<'a> DoubleEndedIterator for StrListIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back_index <= self.index {
            return None;
        }
        self.back_index -= 1;
        let start = if self.back_index > 0 {
            self.ends[self.back_index - 1] as usize
        } else {
            0
        };
        let end = self.ends[self.back_index] as usize;
        // SAFETY: bounds come from `ends` which are valid boundaries; data is valid UTF-8.
        Some(unsafe { from_utf8_unchecked(self.data.get_unchecked(start..end)) })
    }
}

impl<'a> ExactSizeIterator for StrListIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.back_index - self.index
    }
}

impl<'a> std::iter::FusedIterator for StrListIter<'a> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_stack() {
        let mut stack = StrStack::new();
        stack.push("hello");
        stack.push("world");
        let list = StrList::from(stack);
        assert_eq!(list.len(), 2);
        assert_eq!(list.get(0), Some("hello"));
        assert_eq!(list.get(1), Some("world"));
        assert_eq!(list.as_str(), "helloworld");
    }

    #[test]
    fn test_from_iterator() {
        let list: StrList = ["aaa", "bbb", "ccc"].iter().copied().collect();
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0), Some("aaa"));
        assert_eq!(list.get(1), Some("bbb"));
        assert_eq!(list.get(2), Some("ccc"));
    }

    #[test]
    fn test_empty() {
        let list: StrList = std::iter::empty::<&str>().collect();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert_eq!(list.as_str(), "");
        assert_eq!(list.last(), None);
        assert_eq!(list.get(0), None);
    }

    #[test]
    fn test_last() {
        let list: StrList = ["a", "b", "c"].iter().copied().collect();
        assert_eq!(list.last(), Some("c"));
    }

    #[test]
    fn test_get_bounds() {
        let list: StrList = ["abc", "€", "😊"].iter().copied().collect();
        assert_eq!(list.get_bounds(0), Some((0, 3)));
        assert_eq!(list.get_bounds(1), Some((3, 6)));
        assert_eq!(list.get_bounds(2), Some((6, 10)));
        assert_eq!(list.get_bounds(3), None);
    }

    #[test]
    fn test_bytes_len() {
        let list: StrList = ["abc", "€"].iter().copied().collect();
        assert_eq!(list.bytes_len(), 6);
    }

    #[test]
    fn test_iter_forward() {
        let list: StrList = ["a", "b", "c"].iter().copied().collect();
        let collected: Vec<&str> = list.iter().collect();
        assert_eq!(collected, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_iter_reverse() {
        let list: StrList = ["a", "b", "c"].iter().copied().collect();
        let collected: Vec<&str> = list.iter().rev().collect();
        assert_eq!(collected, vec!["c", "b", "a"]);
    }

    #[test]
    fn test_iter_exact_size() {
        let list: StrList = ["a", "b", "c"].iter().copied().collect();
        let mut it = list.iter();
        assert_eq!(it.len(), 3);
        it.next();
        assert_eq!(it.len(), 2);
        it.next_back();
        assert_eq!(it.len(), 1);
    }

    #[test]
    fn test_iter_double_ended_meet_in_middle() {
        let list: StrList = ["a", "b", "c", "d"].iter().copied().collect();
        let mut it = list.iter();
        assert_eq!(it.next(), Some("a"));
        assert_eq!(it.next_back(), Some("d"));
        assert_eq!(it.next(), Some("b"));
        assert_eq!(it.next_back(), Some("c"));
        assert_eq!(it.next(), None);
        assert_eq!(it.next_back(), None);
    }

    #[test]
    fn test_as_ref() {
        let list: StrList = ["hello", "world"].iter().copied().collect();
        let view = list.as_ref();
        assert_eq!(view.len(), 2);
        assert_eq!(view.get(0), Some("hello"));
        assert_eq!(view.get(1), Some("world"));
    }

    #[test]
    fn test_clone_eq() {
        let list: StrList = ["a", "b"].iter().copied().collect();
        let list2 = list.clone();
        assert_eq!(list, list2);
    }

    #[test]
    fn test_debug() {
        let list: StrList = ["hello", "world"].iter().copied().collect();
        let debug = format!("{:?}", list);
        assert_eq!(debug, r#"["hello", "world"]"#);
    }

    #[test]
    fn test_empty_segments() {
        let list: StrList = ["", "abc", ""].iter().copied().collect();
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0), Some(""));
        assert_eq!(list.get(1), Some("abc"));
        assert_eq!(list.get(2), Some(""));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_roundtrip() {
        let list: StrList = ["hello", "world", "€"].iter().copied().collect();
        let json = serde_json::to_string(&list).unwrap();
        assert_eq!(json, r#"["hello","world","€"]"#);
        let deserialized: StrList = serde_json::from_str(&json).unwrap();
        assert_eq!(list, deserialized);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_matches_strstack() {
        let mut stack = StrStack::new();
        stack.push("aaa");
        stack.push("bbb");
        stack.push("ccc");

        let stack_json = serde_json::to_string(&stack).unwrap();
        let list = StrList::from(stack);
        let list_json = serde_json::to_string(&list).unwrap();
        assert_eq!(stack_json, list_json);
    }
}
