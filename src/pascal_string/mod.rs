use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::borrow::Cow;
use std::cmp;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops;
use std::rc::Rc;
use std::str::from_utf8_unchecked;
use std::str::from_utf8_unchecked_mut;
use std::sync::Arc;

use crate::DisplayExt;

mod error;
#[cfg(feature = "serde")]
mod with_serde;

pub use error::TryFromBytesError;
pub use error::InsertError;
pub use error::RemoveError;
pub use error::TryFromStrError;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PascalString<const CAPACITY: usize> {
    len: u8,
    data: [u8; CAPACITY],
}

impl<const CAPACITY: usize> PascalString<CAPACITY> {
    pub const CAPACITY: usize = {
        assert!(
            CAPACITY <= u8::MAX as usize,
            "PascalString max capacity is 255"
        );
        CAPACITY
    };

    #[inline(always)]
    pub const fn new() -> Self {
        // This line triggers a compile time error, if CAPACITY > 255.
        // TODO look for a better way to assert CAPACITY.
        let _ = Self::CAPACITY;

        Self {
            len: 0,
            data: [0; CAPACITY],
        }
    }

    /// Creates a new `PascalString<CAPACITY>` instance from a `&str` within a const context.
    /// This implementation prioritizes const context compatibility over performance.
    /// If a const context is not required, use `try_from` for better performance.
    /// In the future, once const in trait methods is stabilized, this method will be deprecated
    /// in favor of `try_from`.
    pub const fn try_from_str_const(string: &str) -> Option<Self> {
        let _ = Self::CAPACITY;

        if string.len() > CAPACITY {
            return None;
        }
        let mut this = PascalString {
            len: string.len() as u8,
            data: [0; CAPACITY],
        };
        let bytes = string.as_bytes();
        let mut i = 0;
        while i < string.len() {
            this.data[i] = bytes[i];
            i += 1;
        }
        Some(this)
    }

    /// Creates a new `PascalString<CAPACITY>` instance from a `&str`.
    /// If the length of the string exceeds `CAPACITY`,
    /// the string is truncated at the nearest valid UTF-8 boundary
    /// to ensure its length does not exceed `CAPACITY`.
    #[inline]
    pub fn from_str_truncated(string: &str) -> Self {
        let _ = Self::CAPACITY;

        if let Ok(ps) = Self::try_from(string) {
            return ps;
        }

        let mut ps = Self::new();
        ps.push_str_truncated(string);
        ps
    }

    #[inline(always)]
    pub const fn into_inner(self) -> (u8, [u8; CAPACITY]) {
        (self.len, self.data)
    }

    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        self
    }

    #[inline(always)]
    pub fn as_mut_str(&mut self) -> &mut str {
        self
    }

    #[inline(always)]
    #[deprecated(note = "Use `as_mut_str()` (this method name suggests `&mut str` but returns `&str`).")]
    pub fn as_str_mut(&mut self) -> &str {
        self
    }

    #[inline]
    pub fn try_push_str(&mut self, string: &str) -> Result<(), TryFromStrError> {
        let len = self.len();
        let new_len = len + string.len();

        if new_len > CAPACITY {
            return Err(TryFromStrError::TooLong);
        }

        self.data[len..new_len].copy_from_slice(string.as_bytes());
        self.len = new_len as u8;

        Ok(())
    }

    #[inline]
    pub fn try_push(&mut self, ch: char) -> Result<(), TryFromStrError> {
        // TODO special case for ch.len_utf8() == 1
        self.try_push_str(ch.encode_utf8(&mut [0; 4]))
    }

    /// Appends a string slice, panicking if the capacity would be exceeded.
    ///
    /// This mirrors `String::push_str`’s “cannot fail” ergonomics; use `try_push_str` if you want a recoverable error.
    #[inline]
    #[deprecated(note = "PascalString is fixed-capacity; prefer `try_push_str`, `push_str_truncated`, or `push_str_expect_capacity`.")]
    pub fn push_str(&mut self, string: &str) {
        self.push_str_expect_capacity(string);
    }

    /// Appends a character, panicking if the capacity would be exceeded.
    ///
    /// This mirrors `String::push`’s “cannot fail” ergonomics; use `try_push` if you want a recoverable error.
    #[inline]
    #[deprecated(note = "PascalString is fixed-capacity; prefer `try_push`, `push_str_truncated`, or `push_expect_capacity`.")]
    pub fn push(&mut self, ch: char) {
        self.push_expect_capacity(ch);
    }

    /// Appends a string slice, panicking if the capacity would be exceeded.
    #[inline]
    pub fn push_str_expect_capacity(&mut self, string: &str) {
        self.try_push_str(string)
            .expect("PascalString capacity exceeded");
    }

    /// Appends a character, panicking if the capacity would be exceeded.
    #[inline]
    pub fn push_expect_capacity(&mut self, ch: char) {
        self.try_push(ch).expect("PascalString capacity exceeded");
    }

    /// Inserts a string slice at the given byte index.
    ///
    /// This is a true `try_` API: it **never panics**. All failure modes are returned as `InsertError`.
    #[inline]
    pub fn try_insert_str(&mut self, idx: usize, string: &str) -> Result<(), InsertError> {
        let len = self.len();
        if idx > len {
            return Err(InsertError::OutOfBounds { idx, len });
        }
        if !self.is_char_boundary(idx) {
            return Err(InsertError::NotCharBoundary { idx });
        }

        let insert_len = string.len();
        let new_len = len + insert_len;
        if new_len > CAPACITY {
            return Err(InsertError::TooLong);
        }

        // Shift tail to make room.
        self.data.copy_within(idx..len, idx + insert_len);
        // Copy inserted bytes.
        self.data[idx..idx + insert_len].copy_from_slice(string.as_bytes());
        self.len = new_len as u8;
        Ok(())
    }

    /// Inserts a string slice at the given byte index, truncating the inserted string to available capacity.
    ///
    /// Returns the remainder that did not fit.
    ///
    /// This is a true `try_` API: it **never panics**. Index/boundary errors are returned as `InsertError`.
    #[inline]
    pub fn try_insert_str_truncated<'s>(
        &mut self,
        idx: usize,
        string: &'s str,
    ) -> Result<&'s str, InsertError> {
        let len = self.len();
        if idx > len {
            return Err(InsertError::OutOfBounds { idx, len });
        }
        if !self.is_char_boundary(idx) {
            return Err(InsertError::NotCharBoundary { idx });
        }

        let available = CAPACITY.saturating_sub(len);
        if available >= string.len() {
            self.try_insert_str(idx, string)?;
            return Ok("");
        }

        let mut prefix_len = 0;
        for c in string.chars() {
            let l = c.len_utf8();
            if prefix_len + l > available {
                break;
            }
            prefix_len += l;
        }

        let (prefix, remainder) = string.split_at(prefix_len);
        // Prefix is constructed from `chars()` boundaries, so it is valid UTF-8 and fits by construction.
        self.try_insert_str(idx, prefix)?;
        Ok(remainder)
    }

    /// Inserts a string slice at the given byte index, truncating to capacity, panicking on invalid index/boundary.
    ///
    /// Returns the remainder that did not fit.
    #[inline]
    pub fn insert_str_truncated<'s>(&mut self, idx: usize, string: &'s str) -> &'s str {
        self.try_insert_str_truncated(idx, string)
            .expect("invalid index or char boundary")
    }

    /// Inserts a string slice at the given byte index, panicking if the capacity would be exceeded.
    ///
    /// This is an explicit opt-in panicking API for fixed-capacity strings.
    #[inline]
    pub fn insert_str_expect_capacity(&mut self, idx: usize, string: &str) {
        self.try_insert_str(idx, string)
            .expect("PascalString insert failed");
    }

    /// Inserts a string slice at the given byte index, panicking if the capacity would be exceeded.
    #[inline]
    #[deprecated(note = "PascalString is fixed-capacity; prefer `try_insert_str`, `try_insert_str_truncated`, or `insert_str_expect_capacity`.")]
    pub fn insert_str(&mut self, idx: usize, string: &str) {
        self.insert_str_expect_capacity(idx, string);
    }

    /// Inserts a character at the given byte index.
    ///
    /// This is a true `try_` API: it **never panics**. All failure modes are returned as `InsertError`.
    #[inline]
    pub fn try_insert(&mut self, idx: usize, ch: char) -> Result<(), InsertError> {
        let mut buf = [0_u8; 4];
        let s = ch.encode_utf8(&mut buf);
        self.try_insert_str(idx, s)
    }

    /// Inserts a character at the given byte index, panicking if the capacity would be exceeded.
    #[inline]
    pub fn insert_expect_capacity(&mut self, idx: usize, ch: char) {
        self.try_insert(idx, ch)
            .expect("PascalString insert failed");
    }

    /// Inserts a character at the given byte index, panicking if the capacity would be exceeded.
    #[inline]
    #[deprecated(note = "PascalString is fixed-capacity; prefer `try_insert`, `try_insert_str_truncated`, or `insert_expect_capacity`.")]
    pub fn insert(&mut self, idx: usize, ch: char) {
        self.insert_expect_capacity(idx, ch);
    }

    /// Removes and returns the `char` at the given byte index.
    ///
    /// # Panics
    ///
    /// - If `idx >= self.len()`
    /// - If `idx` is not on a UTF-8 character boundary
    #[inline]
    pub fn remove(&mut self, idx: usize) -> char {
        let len = self.len();
        assert!(idx < len, "index out of bounds");
        assert!(self.is_char_boundary(idx), "index is not a char boundary");

        let ch = self.as_str()[idx..].chars().next().expect("idx < len");
        let ch_len = ch.len_utf8();

        // Shift tail left to close the gap.
        self.data.copy_within(idx + ch_len..len, idx);
        let new_len = len - ch_len;
        self.len = new_len as u8;

        // Keep deterministic contents beyond len (not required for soundness, but helps debugging/tests).
        self.data[new_len..len].fill(0);

        ch
    }

    /// Removes and returns the `char` at the given byte index.
    ///
    /// This is a true `try_` API: it **never panics**. All failure modes are returned as `RemoveError`.
    #[inline]
    pub fn try_remove(&mut self, idx: usize) -> Result<char, RemoveError> {
        let len = self.len();
        if idx >= len {
            return Err(RemoveError::OutOfBounds { idx, len });
        }
        if !self.is_char_boundary(idx) {
            return Err(RemoveError::NotCharBoundary { idx });
        }
        Ok(self.remove(idx))
    }

    /// Returns the remainder of the string that was not pushed.
    #[inline]
    pub fn push_str_truncated<'s>(&mut self, string: &'s str) -> &'s str {
        if self.try_push_str(string).is_ok() {
            return "";
        }

        // TODO is there more efficient way to do this?
        //   Maybe iter four bytes from the end of the slice and find the UTF-8 boundary?

        let mut new_len = self.len();
        for c in string.chars() {
            let len = c.len_utf8();
            if new_len + len > CAPACITY {
                break;
            };
            new_len += len;
        }

        let pos = new_len - self.len();
        let (substring, remainder) = string.split_at(pos);
        self.try_push_str(substring).unwrap();

        remainder
    }

    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        if new_len <= self.len() {
            assert!(self.is_char_boundary(new_len));
            self.len = new_len as u8;
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        let ch = self.chars().next_back()?;
        let newlen = self.len() - ch.len_utf8();
        self.len = newlen as u8;
        Some(ch)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

// -- Common traits --------------------------------------------------------------------------------

impl<const CAPACITY: usize> Default for PascalString<CAPACITY> {
    #[inline(always)]
    fn default() -> Self {
        let _ = Self::CAPACITY;

        Self::new()
    }
}

impl<T: ops::Deref<Target = str> + ?Sized, const CAPACITY: usize> PartialEq<T>
    for PascalString<CAPACITY>
{
    #[inline(always)]
    fn eq(&self, other: &T) -> bool {
        self.as_str().eq(other.deref())
    }
}

macro_rules! impl_reverse_eq_for_str_types {
    ($($t:ty),*) => {
        $(
            impl<const CAPACITY: usize> PartialEq<PascalString<CAPACITY>> for $t {
                #[inline(always)]
                fn eq(&self, other: &PascalString<CAPACITY>) -> bool {
                    let a: &str = self.as_ref();
                    let b = other.as_str();
                    a.eq(b)
                }
            }

            impl<const CAPACITY: usize> PartialEq<PascalString<CAPACITY>> for &$t {
                #[inline(always)]
                fn eq(&self, other: &PascalString<CAPACITY>) -> bool {
                    let a: &str = self.as_ref();
                    let b = other.as_str();
                    a.eq(b)
                }
            }

            impl<const CAPACITY: usize> PartialEq<PascalString<CAPACITY>> for &mut $t {
                #[inline(always)]
                fn eq(&self, other: &PascalString<CAPACITY>) -> bool {
                    let a: &str = self.as_ref();
                    let b = other.as_str();
                    a.eq(b)
                }
            }
        )*
    };
}

impl_reverse_eq_for_str_types!(String, str, Cow<'_, str>, Box<str>, Rc<str>, Arc<str>);

impl<const CAPACITY: usize> Eq for PascalString<CAPACITY> {}

impl<T: ops::Deref<Target = str>, const CAPACITY: usize> PartialOrd<T> for PascalString<CAPACITY> {
    #[inline(always)]
    fn partial_cmp(&self, other: &T) -> Option<cmp::Ordering> {
        self.as_str().partial_cmp(other.deref())
    }
}

impl<const CAPACITY: usize> Ord for PascalString<CAPACITY> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<const CAPACITY: usize> Hash for PascalString<CAPACITY> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

// -- Formatting -----------------------------------------------------------------------------------

impl<const CAPACITY: usize> fmt::Debug for PascalString<CAPACITY> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name: PascalString<39> = format_args!("PascalString<{CAPACITY}>")
            .try_to_fmt()
            .unwrap_or_else(|_| "PascalString<?>".to_fmt());
        f.debug_tuple(&name).field(&self.as_str()).finish()
    }
}

impl<const CAPACITY: usize> fmt::Display for PascalString<CAPACITY> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

// -- Reference ------------------------------------------------------------------------------------

impl<const CAPACITY: usize> ops::Deref for PascalString<CAPACITY> {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        // SAFETY: PascalString maintains its length invariant.
        let bytes = unsafe { self.data.get_unchecked(..self.len()) };
        // SAFETY: PascalString maintains its utf8 invariant.
        unsafe { from_utf8_unchecked(bytes) }
    }
}

impl<const CAPACITY: usize> ops::DerefMut for PascalString<CAPACITY> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let len = self.len();
        // SAFETY: PascalString maintains its length invariant.
        let bytes = unsafe { self.data.get_unchecked_mut(..len) };
        // SAFETY: PascalString maintains its utf8 invariant.
        unsafe { from_utf8_unchecked_mut(bytes) }
    }
}

impl<const CAPACITY: usize> Borrow<str> for PascalString<CAPACITY> {
    #[inline(always)]
    fn borrow(&self) -> &str {
        self
    }
}

impl<const CAPACITY: usize> AsRef<str> for PascalString<CAPACITY> {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self
    }
}

impl<const CAPACITY: usize> AsRef<[u8]> for PascalString<CAPACITY> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<const CAPACITY: usize> AsMut<str> for PascalString<CAPACITY> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut str {
        self
    }
}

impl<const CAPACITY: usize> BorrowMut<str> for PascalString<CAPACITY> {
    #[inline(always)]
    fn borrow_mut(&mut self) -> &mut str {
        self
    }
}

// -- Conversion -----------------------------------------------------------------------------------

impl<'a, const CAPACITY: usize> TryFrom<&'a [u8]> for PascalString<CAPACITY> {
    type Error = TryFromBytesError;

    #[inline]
    fn try_from(bytes: &'a [u8]) -> Result<PascalString<CAPACITY>, Self::Error> {
        let _ = Self::CAPACITY;

        let string = core::str::from_utf8(bytes)?;
        Ok(Self::try_from(string)?)
    }
}

impl<'a, const CAPACITY: usize> TryFrom<&'a mut str> for PascalString<CAPACITY> {
    type Error = TryFromStrError;

    #[inline]
    fn try_from(value: &'a mut str) -> Result<PascalString<CAPACITY>, Self::Error> {
        Self::try_from(&*value)
    }
}

impl<'a, const CAPACITY: usize> TryFrom<&'a str> for PascalString<CAPACITY> {
    type Error = TryFromStrError;

    #[inline]
    fn try_from(value: &'a str) -> Result<PascalString<CAPACITY>, Self::Error> {
        let _ = Self::CAPACITY;

        let bytes = value.as_bytes();
        let len = bytes.len();

        if len > CAPACITY {
            return Err(TryFromStrError::TooLong);
        }

        let data = match <[u8; CAPACITY]>::try_from(bytes).ok() {
            Some(data) => data,
            None => {
                let mut data = [0; CAPACITY];
                data[..len].copy_from_slice(bytes);
                data
            }
        };

        let len = len as u8;

        Ok(PascalString { len, data })
    }
}

impl<const CAPACITY: usize> TryFrom<char> for PascalString<CAPACITY> {
    type Error = TryFromStrError;

    #[inline]
    fn try_from(value: char) -> Result<PascalString<CAPACITY>, Self::Error> {
        let _ = Self::CAPACITY;

        Self::try_from(value.encode_utf8(&mut [0; 4]))
    }
}

impl<const CAPACITY: usize> std::str::FromStr for PascalString<CAPACITY> {
    type Err = TryFromStrError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

// -- IO -------------------------------------------------------------------------------------------

impl<const CAPACITY: usize> fmt::Write for PascalString<CAPACITY> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.try_push_str(s).map_err(|_| fmt::Error)
    }
}

// -- Tests ----------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    #[test]
    fn test_eq() {
        use std::fmt::Write;

        let s = String::from("abc");
        let ps = PascalString::<4>::try_from("abc").unwrap();

        assert_eq!(ps, s);
        // assert_eq!(ps.as_view(), s);
        // assert_eq!(ps.as_view(), ps);

        let s = String::from("abcd");
        let mut ps = PascalString::<4>::new();
        write!(&mut ps, "abcd").unwrap();

        assert_eq!(ps, s);
    }

    #[test]
    fn test_ord() {
        let ps1 = PascalString::<4>::try_from("abc").unwrap();
        let ps2 = PascalString::<4>::try_from("abcd").unwrap();

        assert!(ps1 < ps2);
        assert!(ps1 <= ps2);
        assert!(ps2 > ps1);
        assert!(ps2 >= ps1);
    }

    #[test]
    fn test_size() {
        assert_eq!(mem::size_of::<PascalString<0>>(), 1);
        assert_eq!(mem::size_of::<PascalString<1>>(), 2);
        assert_eq!(mem::size_of::<PascalString<2>>(), 3);
        assert_eq!(mem::size_of::<PascalString<3>>(), 4);
        assert_eq!(mem::size_of::<PascalString<4>>(), 5);
    }

    // TODO use https://github.com/Manishearth/compiletest-rs to test compile errors.
    // #[test]
    fn _test_max_size() {
        // Every of these lines should not compile with a compile error:
        // "the evaluated program panicked at 'PascalString max capacity is 255'".
        //
        // Also, compiler should point to the line that triggered the error, e.g.:
        //
        // note: the above error was encountered while instantiating `fn <PascalString<256> as std::default::Default>::default`
        //    --> src/lib.rs:254:37
        //     |
        // 254 |         let _x: PascalString<256> = PascalString::default();
        //     |                                     ^^^^^^^^^^^^^^^^^^^^^^^
        //
        let _x: PascalString<256> = PascalString::default();
        let _x: PascalString<256> = PascalString::new();
        let _x: PascalString<256> = PascalString::try_from("").unwrap();
        let _x: PascalString<256> = PascalString::from_str_truncated("");
    }

    #[test]
    fn test_deref() {
        let ps = PascalString::<3>::try_from("abc").unwrap();
        let map: std::collections::HashSet<_> = ["abc"].into_iter().collect();
        assert!(map.contains(&*ps));
    }

    #[test]
    fn test_try_push_str_too_long_does_not_modify() {
        let mut ps = PascalString::<4>::try_from("ab").unwrap();
        assert_eq!(ps.as_str(), "ab");

        let err = ps.try_push_str("cde").unwrap_err();
        assert_eq!(err, TryFromStrError::TooLong);
        assert_eq!(ps.as_str(), "ab");
    }

    #[test]
    fn test_try_push_char_too_long_does_not_modify() {
        let mut ps = PascalString::<3>::new();
        ps.try_push('€').unwrap(); // 3 bytes
        assert_eq!(ps.as_str(), "€");

        let err = ps.try_push('a').unwrap_err(); // +1 would overflow
        assert_eq!(err, TryFromStrError::TooLong);
        assert_eq!(ps.as_str(), "€");
    }

    #[test]
    fn test_push_str_truncated_respects_utf8_boundary() {
        let mut ps = PascalString::<4>::new();

        // "€" is 3 bytes. "€a" is 4 bytes. "€ab" is 5 bytes.
        let remainder = ps.push_str_truncated("€ab");
        assert_eq!(ps.as_str(), "€a");
        assert_eq!(remainder, "b");
    }

    #[test]
    fn test_from_str_truncated_truncates_on_boundary() {
        let ps = PascalString::<4>::from_str_truncated("€ab");
        assert_eq!(ps.as_str(), "€a");
        assert_eq!(ps.len(), 4);
    }

    #[test]
    fn test_truncate_requires_char_boundary() {
        let mut ps = PascalString::<4>::new();
        ps.try_push('€').unwrap(); // 3 bytes
        ps.try_push('a').unwrap(); // 1 byte => 4
        assert_eq!(ps.as_str(), "€a");

        // 1 is in the middle of the 3-byte UTF-8 sequence for '€'.
        let result = std::panic::catch_unwind(move || {
            let mut ps = ps;
            ps.truncate(1);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_bytes_invalid_utf8() {
        let err = PascalString::<8>::try_from(&[0xff_u8][..]).unwrap_err();
        match err {
            TryFromBytesError::Utf8Error(_) => {}
            _ => panic!("expected Utf8Error, got: {err:?}"),
        }
    }

    #[test]
    fn test_try_from_bytes_too_long() {
        let err = PascalString::<2>::try_from(&b"abc"[..]).unwrap_err();
        assert_eq!(err, TryFromBytesError::TooLong);
    }

    #[test]
    fn test_capacity_zero_behavior() {
        let mut ps = PascalString::<0>::new();
        assert_eq!(ps.len(), 0);
        assert!(ps.is_empty());
        assert_eq!(ps.as_str(), "");

        assert_eq!(ps.try_push_str(""), Ok(()));
        assert_eq!(ps.try_push_str("a"), Err(TryFromStrError::TooLong));

        let rem = ps.push_str_truncated("hello");
        assert_eq!(ps.as_str(), "");
        assert_eq!(rem, "hello");

        assert_eq!(PascalString::<0>::from_str_truncated("hello").as_str(), "");
        assert!(PascalString::<0>::try_from("").is_ok());
        assert_eq!(
            PascalString::<0>::try_from("a").unwrap_err(),
            TryFromStrError::TooLong
        );
    }

    #[test]
    fn test_into_inner_invariants() {
        let ps = PascalString::<4>::try_from("ab").unwrap();
        let (len, data) = ps.into_inner();
        assert_eq!(len, 2);
        assert_eq!(&data[..2], b"ab");
        assert_eq!(&data[2..], &[0, 0]);
    }

    #[test]
    fn test_as_mut_str_allows_in_place_mutation() {
        let mut ps = PascalString::<4>::try_from("ab").unwrap();
        ps.as_mut_str().make_ascii_uppercase();
        assert_eq!(ps.as_str(), "AB");
    }

    #[test]
    fn test_push_str_panics_on_overflow() {
        let result = std::panic::catch_unwind(|| {
            let mut ps = PascalString::<4>::new();
            ps.push_str_expect_capacity("abcde");
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_insert_str_and_remove_unicode_boundaries() {
        let mut ps = PascalString::<8>::try_from("ab").unwrap();
        ps.insert_str_expect_capacity(1, "€"); // 3 bytes
        assert_eq!(ps.as_str(), "a€b");

        let removed = ps.remove(1);
        assert_eq!(removed, '€');
        assert_eq!(ps.as_str(), "ab");
    }

    #[test]
    fn test_try_insert_str_too_long_does_not_modify() {
        let mut ps = PascalString::<4>::try_from("ab").unwrap();
        let err = ps.try_insert_str(1, "€").unwrap_err(); // would become 5 bytes
        assert_eq!(err, InsertError::TooLong);
        assert_eq!(ps.as_str(), "ab");
    }

    #[test]
    fn test_try_from_str_const() {
        const PS: Option<PascalString<4>> = PascalString::<4>::try_from_str_const("ab");
        let ps = PS.unwrap();
        assert_eq!(ps.as_str(), "ab");

        const TOO_LONG: Option<PascalString<2>> = PascalString::<2>::try_from_str_const("abc");
        assert!(TOO_LONG.is_none());
    }
}
