use std::borrow::Borrow;
use std::borrow::Cow;
use std::cmp;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops;
use std::rc::Rc;
use std::string::FromUtf16Error;
use std::string::FromUtf8Error;
use std::sync::Arc;

use crate::pascal_string;
use crate::DisplayExt;
use crate::PascalString;

#[cfg(feature = "serde")]
mod with_serde;

pub const DEFAULT_CAPACITY: usize = 30;

/// A string that stores short values on the stack and longer values on the heap.
///
/// ### Storage semantics (explicit conversions)
///
/// This type may **promote** from stack to heap during mutating operations (e.g. `push_str`, `reserve`) when the stack
/// capacity is exceeded.
///
/// It does **not** automatically demote from heap to stack when the contents become shorter (including during
/// in-place deserialization). This is intentional: implicit demotion can introduce surprising realloc/dealloc churn in
/// real workloads (e.g. shorten → re-grow). If you want to attempt a demotion, call `try_into_stack`.
#[derive(Clone)]
pub enum SmartString<const N: usize = DEFAULT_CAPACITY> {
    Heap(String),
    Stack(PascalString<N>),
}

impl<const N: usize> SmartString<N> {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self::Stack(PascalString::new())
    }

    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity <= N {
            Self::new()
        } else {
            Self::Heap(String::with_capacity(capacity))
        }
    }

    #[inline]
    pub fn from_utf8(vec: Vec<u8>) -> Result<Self, FromUtf8Error> {
        String::from_utf8(vec).map(Self::Heap)
    }

    // TBD What to do with this?
    // #[cfg(not(no_global_oom_handling))]
    // #[inline]
    // #[must_use]
    // pub fn from_utf8_lossy(v: &[u8]) -> Cow<'_, str> {
    //     match String::from_utf8_lossy(v) {
    //         Cow::Borrowed(s) => Cow::Borrowed(s),
    //         Cow::Owned(s) => Cow::Owned(Self::Heap(s)),
    //     }
    // }

    pub fn from_utf16(v: &[u16]) -> Result<Self, FromUtf16Error> {
        String::from_utf16(v).map(Self::Heap)
    }

    #[must_use]
    #[inline]
    pub fn from_utf16_lossy(v: &[u16]) -> Self {
        Self::Heap(String::from_utf16_lossy(v))
    }

    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self
    }

    #[inline]
    #[must_use]
    pub fn as_mut_str(&mut self) -> &mut str {
        self
    }

    #[inline]
    pub fn is_heap(&self) -> bool {
        matches!(self, Self::Heap(_))
    }

    #[inline]
    pub fn is_stack(&self) -> bool {
        matches!(self, Self::Stack(_))
    }

    #[inline]
    #[must_use]
    pub fn into_heap(self) -> Self {
        Self::Heap(match self {
            Self::Stack(s) => s.to_string(),
            Self::Heap(s) => s,
        })
    }

    #[inline]
    #[must_use]
    pub fn try_into_stack(self) -> Self {
        match self {
            Self::Stack(s) => Self::Stack(s),
            Self::Heap(s) => match PascalString::try_from(s.as_str()) {
                Ok(s) => Self::Stack(s),
                Err(pascal_string::TryFromStrError::TooLong) => Self::Heap(s),
            },
        }
    }

    #[inline]
    pub fn push_str(&mut self, string: &str) {
        match self {
            Self::Heap(s) => s.push_str(string),
            Self::Stack(s) => match s.try_push_str(string) {
                Ok(()) => (),
                Err(pascal_string::TryFromStrError::TooLong) => {
                    let mut new = String::with_capacity(s.len() + string.len());
                    new.push_str(s.as_str());
                    new.push_str(string);
                    *self = Self::Heap(new);
                }
            },
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        match self {
            Self::Heap(s) => s.capacity(),
            Self::Stack(s) => s.capacity(),
        }
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        match self {
            Self::Heap(s) => s.reserve(additional),
            Self::Stack(s) => {
                if s.capacity() - s.len() < additional {
                    let mut new = String::with_capacity(s.len() + additional);
                    new.push_str(s.as_str());
                    *self = Self::Heap(new);
                }
            }
        }
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        match self {
            Self::Heap(s) => s.reserve_exact(additional),
            Self::Stack(s) => {
                if s.capacity() - s.len() < additional {
                    let mut new = String::new();
                    new.reserve_exact(s.len() + additional);
                    new.push_str(s.as_str());
                    *self = Self::Heap(new);
                }
            }
        }
    }

    #[rustversion::since(1.57)]
    pub fn try_reserve(
        &mut self,
        additional: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        match self {
            Self::Heap(s) => s.try_reserve(additional),
            Self::Stack(s) => {
                if s.capacity() - s.len() < additional {
                    let mut new = String::new();
                    new.try_reserve(s.len() + additional)?;
                    new.push_str(s.as_str());
                    *self = Self::Heap(new);
                }
                Ok(())
            }
        }
    }

    #[rustversion::since(1.57)]
    pub fn try_reserve_exact(
        &mut self,
        additional: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        match self {
            Self::Heap(s) => s.try_reserve_exact(additional),
            Self::Stack(s) => {
                if s.capacity() - s.len() < additional {
                    let mut new = String::new();
                    new.try_reserve_exact(s.len() + additional)?;
                    new.push_str(s.as_str());
                    *self = Self::Heap(new);
                }
                Ok(())
            }
        }
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        match self {
            Self::Heap(s) => s.shrink_to_fit(),
            Self::Stack(_) => (),
        }
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        match self {
            Self::Heap(s) => s.shrink_to(min_capacity),
            Self::Stack(_) => (),
        }
    }

    pub fn push(&mut self, ch: char) {
        match self {
            Self::Heap(s) => s.push(ch),
            Self::Stack(s) => match s.try_push(ch) {
                Ok(()) => (),
                Err(pascal_string::TryFromStrError::TooLong) => {
                    let mut new = String::with_capacity(s.len() + ch.len_utf8());
                    new.push_str(s.as_str());
                    new.push(ch);
                    *self = Self::Heap(new);
                }
            },
        }
    }

    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        match self {
            Self::Heap(s) => s.truncate(new_len),
            Self::Stack(s) => s.truncate(new_len),
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        match self {
            Self::Heap(s) => s.pop(),
            Self::Stack(s) => s.pop(),
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        match self {
            Self::Heap(s) => s.clear(),
            Self::Stack(s) => s.clear(),
        }
    }
}

// -- Common traits --------------------------------------------------------------------------------

impl<const N: usize> Default for SmartString<N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ops::Deref<Target = str> + ?Sized, const CAPACITY: usize> PartialEq<T>
    for SmartString<CAPACITY>
{
    #[inline(always)]
    fn eq(&self, other: &T) -> bool {
        self.as_str().eq(other.deref())
    }
}

macro_rules! impl_reverse_eq_for_str_types {
    ($($t:ty),*) => {
        $(
            impl<const N: usize> PartialEq<SmartString<N>> for $t {
                #[inline(always)]
                fn eq(&self, other: &SmartString<N>) -> bool {
                    let a: &str = self.as_ref();
                    let b = other.as_str();
                    a.eq(b)
                }
            }

            impl<const N: usize> PartialEq<SmartString<N>> for &$t {
                #[inline(always)]
                fn eq(&self, other: &SmartString<N>) -> bool {
                    let a: &str = self.as_ref();
                    let b = other.as_str();
                    a.eq(b)
                }
            }

            impl<const N: usize> PartialEq<SmartString<N>> for &mut $t {
                #[inline(always)]
                fn eq(&self, other: &SmartString<N>) -> bool {
                    let a: &str = self.as_ref();
                    let b = other.as_str();
                    a.eq(b)
                }
            }
        )*
    };
}

impl_reverse_eq_for_str_types!(String, str, Cow<'_, str>, Box<str>, Rc<str>, Arc<str>);

impl<const M: usize, const N: usize> PartialEq<SmartString<N>> for &PascalString<M> {
    #[inline(always)]
    fn eq(&self, other: &SmartString<N>) -> bool {
        let a: &str = self.as_ref();
        let b = other.as_str();
        a.eq(b)
    }
}

impl<const M: usize, const N: usize> PartialEq<SmartString<N>> for &mut PascalString<M> {
    #[inline(always)]
    fn eq(&self, other: &SmartString<N>) -> bool {
        let a: &str = self.as_ref();
        let b = other.as_str();
        a.eq(b)
    }
}

impl<const N: usize> Eq for SmartString<N> {}

impl<T: ops::Deref<Target = str>, const N: usize> PartialOrd<T> for SmartString<N> {
    #[inline(always)]
    fn partial_cmp(&self, other: &T) -> Option<cmp::Ordering> {
        self.as_str().partial_cmp(other.deref())
    }
}

impl<const N: usize> Ord for SmartString<N> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<const N: usize> Hash for SmartString<N> {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

// -- Formatting -----------------------------------------------------------------------------------

impl<const N: usize> fmt::Debug for SmartString<N> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name: PascalString<39> = format_args!("SmartString<{N}>")
            .try_to_fmt()
            .unwrap_or_else(|_| "SmartString<?>".to_fmt());
        f.debug_tuple(&name).field(&self.as_str()).finish()
    }
}

impl<const N: usize> fmt::Display for SmartString<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Heap(s) => s.fmt(f),
            Self::Stack(s) => s.fmt(f),
        }
    }
}

// -- Reference ------------------------------------------------------------------------------------

impl<const N: usize> ops::Deref for SmartString<N> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Heap(s) => s.deref(),
            Self::Stack(s) => s.deref(),
        }
    }
}

impl<const N: usize> ops::DerefMut for SmartString<N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Heap(s) => s.deref_mut(),
            Self::Stack(s) => s.deref_mut(),
        }
    }
}

impl<const N: usize> Borrow<str> for SmartString<N> {
    #[inline(always)]
    fn borrow(&self) -> &str {
        self
    }
}

impl<const N: usize> AsRef<str> for SmartString<N> {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self
    }
}

impl<const N: usize> AsRef<[u8]> for SmartString<N> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

// -- Conversion -----------------------------------------------------------------------------------

impl<const N: usize> From<String> for SmartString<N> {
    #[inline]
    fn from(s: String) -> Self {
        Self::Heap(s)
    }
}

impl<const M: usize, const N: usize> From<PascalString<M>> for SmartString<N> {
    #[inline]
    fn from(s: PascalString<M>) -> Self {
        PascalString::try_from(s.as_str())
            .map(Self::Stack)
            .unwrap_or_else(|pascal_string::TryFromStrError::TooLong| Self::Heap(s.to_string()))
    }
}

impl<const N: usize> From<&str> for SmartString<N> {
    #[inline]
    fn from(s: &str) -> Self {
        PascalString::try_from(s)
            .map(Self::Stack)
            .unwrap_or_else(|pascal_string::TryFromStrError::TooLong| Self::Heap(String::from(s)))
    }
}

impl<const N: usize> From<&mut str> for SmartString<N> {
    #[inline]
    fn from(s: &mut str) -> Self {
        Self::from(&*s)
    }
}

impl<const N: usize> From<Cow<'_, str>> for SmartString<N> {
    #[inline]
    fn from(s: Cow<'_, str>) -> Self {
        match s {
            Cow::Borrowed(s) => Self::from(s),
            Cow::Owned(s) => Self::Heap(s),
        }
    }
}

impl<const N: usize> FromIterator<char> for SmartString<N> {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let mut s = Self::new();
        // TODO s.extend(iter);
        for ch in iter {
            s.push(ch);
        }
        s
    }
}

impl<'a, const N: usize> FromIterator<&'a str> for SmartString<N> {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let mut s = Self::new();
        // TODO s.extend(iter);
        for string in iter {
            s.push_str(string);
        }
        s
    }
}

// -- IO -------------------------------------------------------------------------------------------

impl<const N: usize> fmt::Write for SmartString<N> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

// -- ops ------------------------------------------------------------------------------------------

impl<const N: usize, T: ops::Deref<Target = str>> ops::Add<T> for SmartString<N> {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: T) -> Self::Output {
        self.push_str(&rhs);
        self
    }
}

// -- Tests ----------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    #[test]
    fn test_size() {
        // Default stack capacity is 30 bytes, corresponding to 32 bytes of the enum.
        assert_eq!(mem::size_of::<SmartString>(), 32);

        // The minimal size of the enum is 24 bytes, even if the stack variant capacity
        // is less than 15 bytes.
        assert_eq!(mem::size_of::<SmartString<0>>(), 24);
        assert_eq!(mem::size_of::<SmartString<1>>(), 24);
        assert_eq!(mem::size_of::<SmartString<15>>(), 24);

        // It is unclear why the size of the enum grows to 32 bytes
        // starting from 17 bytes of size for the stack variant.
        assert_eq!(mem::size_of::<SmartString<16>>(), 32);
        assert_eq!(mem::size_of::<SmartString<22>>(), 32);

        // The size of the enum is expected to be 32 bytes for the following capacities.
        assert_eq!(mem::size_of::<SmartString<23>>(), 32);
        assert_eq!(mem::size_of::<SmartString<30>>(), 32);

        // Additional bytes of capacity increases the size of the enum by size of a pointer
        // (8 bytes on 64-bit platforms) by steps of size of a pointer.
        assert_eq!(mem::size_of::<SmartString<31>>(), 40);
        assert_eq!(mem::size_of::<SmartString<38>>(), 40);

        assert_eq!(mem::size_of::<SmartString<39>>(), 48);
        assert_eq!(mem::size_of::<SmartString<46>>(), 48);
    }

    #[test]
    fn test_from_str_picks_stack_or_heap() {
        let s = SmartString::<4>::from("abcd");
        assert!(s.is_stack());

        let s = SmartString::<4>::from("abcde");
        assert!(s.is_heap());
    }

    #[test]
    fn test_push_str_transitions_stack_to_heap() {
        let mut s = SmartString::<4>::new();
        assert!(s.is_stack());

        s.push_str("ab");
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "ab");

        s.push_str("cd");
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "abcd");

        // Overflow stack capacity => move to heap.
        s.push_str("e");
        assert!(s.is_heap());
        assert_eq!(s.as_str(), "abcde");
    }

    #[test]
    fn test_push_char_and_unicode_boundaries() {
        let mut s = SmartString::<4>::new();
        s.push('€'); // 3 bytes
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "€");

        s.push('a'); // +1 byte => exactly 4
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "€a");

        // +1 byte => overflow => heap
        s.push('b');
        assert!(s.is_heap());
        assert_eq!(s.as_str(), "€ab");

        // Truncate on UTF-8 boundary should work for both stack and heap variants.
        s.truncate(3);
        assert_eq!(s.as_str(), "€");
        assert_eq!(s.pop(), Some('€'));
        assert_eq!(s.as_str(), "");
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn test_reserve_transitions_stack_to_heap() {
        let mut s = SmartString::<4>::from("ab");
        assert!(s.is_stack());

        // Fits within remaining stack capacity.
        s.reserve(2);
        assert!(s.is_stack());

        // Requires more than remaining capacity => transition to heap.
        s.reserve(3);
        assert!(s.is_heap());
        assert_eq!(s.as_str(), "ab");
    }

    #[test]
    fn test_try_into_stack_converts_short_heap_string() {
        let s = SmartString::<4>::from(String::from("abc"));
        assert!(s.is_heap());

        let s = s.try_into_stack();
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "abc");
    }

    #[test]
    fn test_into_heap_always_returns_heap_variant() {
        let s = SmartString::<4>::from("abc");
        assert!(s.is_stack());

        let s = s.into_heap();
        assert!(s.is_heap());
        assert_eq!(s.as_str(), "abc");
    }

    #[test]
    fn test_truncate_does_not_demote_heap_to_stack() {
        let mut s = SmartString::<4>::from("abcde");
        assert!(s.is_heap());

        s.truncate(2);
        assert_eq!(s.as_str(), "ab");
        assert!(s.is_heap());

        let s = s.try_into_stack();
        assert_eq!(s.as_str(), "ab");
        assert!(s.is_stack());
    }

    #[rustversion::since(1.57)]
    #[test]
    fn test_try_reserve_transitions_stack_to_heap() {
        let mut s = SmartString::<4>::from("ab");
        assert!(s.is_stack());

        // Fits within remaining stack capacity.
        s.try_reserve(2).unwrap();
        assert!(s.is_stack());

        // Exceeds remaining stack capacity => transition to heap.
        s.try_reserve(3).unwrap();
        assert!(s.is_heap());
        assert_eq!(s.as_str(), "ab");
    }

    #[rustversion::since(1.57)]
    #[test]
    fn test_try_reserve_exact_transitions_stack_to_heap() {
        let mut s = SmartString::<4>::from("ab");
        assert!(s.is_stack());

        // Exceeds remaining stack capacity => transition to heap.
        s.try_reserve_exact(3).unwrap();
        assert!(s.is_heap());
        assert_eq!(s.as_str(), "ab");
    }
}
