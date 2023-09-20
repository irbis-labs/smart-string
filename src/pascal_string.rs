use std::borrow::Borrow;
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

    /// Creates a new `PascalString<CAPACITY>` from a `&str`.
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
    pub fn into_inner(self) -> (u8, [u8; CAPACITY]) {
        (self.len, self.data)
    }

    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        self
    }

    #[inline(always)]
    pub fn as_str_mut(&mut self) -> &str {
        self
    }

    #[inline]
    pub fn try_push_str(&mut self, string: &str) -> Result<(), ()> {
        let len = self.len();
        let new_len = len + string.len();

        if new_len > CAPACITY {
            return Err(());
        }

        self.data[len..new_len].copy_from_slice(string.as_bytes());
        self.len = new_len as u8;

        Ok(())
    }

    #[inline]
    pub fn try_push(&mut self, ch: char) -> Result<(), ()> {
        // TODO special case for ch.len_utf8() == 1
        self.try_push_str(ch.encode_utf8(&mut [0; 4]))
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
        let ch = self.chars().rev().next()?;
        let newlen = self.len() - ch.len_utf8();
        self.len = newlen as u8;
        Some(ch)
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

// -- Conversions ----------------------------------------------------------------------------------

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

impl<'a, const CAPACITY: usize> TryFrom<&'a str> for PascalString<CAPACITY> {
    type Error = ();

    #[inline]
    fn try_from(value: &'a str) -> Result<PascalString<CAPACITY>, Self::Error> {
        let _ = Self::CAPACITY;

        let bytes = value.as_bytes();
        let len = bytes.len();

        if len > CAPACITY {
            return Err(());
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
    type Error = ();

    #[inline]
    fn try_from(value: char) -> Result<PascalString<CAPACITY>, Self::Error> {
        let _ = Self::CAPACITY;

        PascalString::try_from(value.encode_utf8(&mut [0; 4]).as_ref())
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
}
