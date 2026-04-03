use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::borrow::Cow;
use std::cmp;
use std::convert::Infallible;
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

mod error;
pub use error::Utf16DecodeError;

mod into_chars;
pub use into_chars::IntoChars;

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
    fn ensure_heap_mut(&mut self) -> &mut String {
        if let Self::Stack(s) = self {
            *self = Self::Heap(s.to_string());
        }
        match self {
            Self::Heap(s) => s,
            Self::Stack(_) => unreachable!("just promoted to heap"),
        }
    }

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

    /// Fallible version of `with_capacity`.
    ///
    /// This never panics; allocation failures are returned as `TryReserveError`.
    ///
    /// ## Note on `String::try_with_capacity`
    ///
    /// Newer Rust versions provide `String::try_with_capacity`, but this crate’s MSRV is **Rust 1.59**
    /// (see crate `README.md`). To keep `SmartString::try_with_capacity` available under MSRV, we
    /// implement the equivalent semantics via `String::new()` + `try_reserve_exact`, which is
    /// available starting from **Rust 1.57**.
    ///
    /// When MSRV is bumped to a version where `String::try_with_capacity` is available, this should
    /// be revisited for closer upstream parity and to reduce “looks like a copy/paste bug” risk.
    #[inline]
    pub fn try_with_capacity(capacity: usize) -> Result<Self, std::collections::TryReserveError> {
        if capacity <= N {
            return Ok(Self::new());
        }

        let mut s = String::new();
        // NOTE(MSRV): use `try_reserve_exact` instead of `String::try_with_capacity` (not available on our MSRV).
        s.try_reserve_exact(capacity)?;
        Ok(Self::Heap(s))
    }

    #[inline]
    pub fn from_utf8(vec: Vec<u8>) -> Result<Self, FromUtf8Error> {
        String::from_utf8(vec).map(Self::Heap)
    }

    pub fn from_utf16(v: &[u16]) -> Result<Self, FromUtf16Error> {
        String::from_utf16(v).map(|s| Self::from(s.as_str()))
    }

    #[must_use]
    #[inline]
    pub fn from_utf16_lossy(v: &[u16]) -> Self {
        let s = String::from_utf16_lossy(v);
        Self::from(s.as_str())
    }

    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self
    }

    /// Returns the underlying UTF-8 bytes.
    ///
    /// This is equivalent to `self.as_str().as_bytes()`, but provided as an inherent method for
    /// `std::string::String` API parity and rustdoc discoverability.
    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.as_str().is_empty()
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

    // --- String-like APIs that require heap delegation -------------------------------------------

    /// Converts the `SmartString` into an iterator over its `char`s.
    ///
    /// The returned [`IntoChars`] iterator consumes `self`.  It supports forward and backward
    /// traversal, exact remaining-char counting, and fused behaviour.
    #[inline]
    #[must_use]
    pub fn into_chars(self) -> IntoChars<N> {
        match self {
            Self::Stack(s) => IntoChars::new_stack(s),
            Self::Heap(s) => IntoChars::new_heap(s),
        }
    }

    #[inline]
    #[must_use]
    pub fn into_string(self) -> String {
        self.into()
    }

    #[inline]
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.into_string().into_bytes()
    }

    #[inline]
    #[must_use]
    pub fn into_boxed_str(self) -> Box<str> {
        self.into_string().into_boxed_str()
    }

    #[inline]
    #[must_use]
    pub fn leak<'a>(self) -> &'a mut str {
        self.into_string().leak()
    }

    #[inline]
    #[must_use]
    pub fn from_utf8_lossy(v: &[u8]) -> Cow<'_, str> {
        String::from_utf8_lossy(v)
    }

    /// Like `String::from_utf8_lossy_owned`: consumes the byte buffer and returns an owned string,
    /// replacing invalid sequences with U+FFFD.
    ///
    /// This never panics. The returned value uses the stack variant when it fits.
    #[inline]
    #[must_use]
    pub fn from_utf8_lossy_owned(v: Vec<u8>) -> Self {
        let owned = match String::from_utf8(v) {
            Ok(s) => s,
            Err(e) => String::from_utf8_lossy(&e.into_bytes()).into_owned(),
        };

        if owned.len() <= N {
            // Avoid panics: fall back to heap on unexpected conversion failure.
            if let Ok(ps) = PascalString::<N>::try_from(owned.as_str()) {
                return Self::Stack(ps);
            }
        }
        Self::Heap(owned)
    }

    #[inline]
    pub fn insert(&mut self, idx: usize, ch: char) {
        match self {
            Self::Heap(s) => s.insert(idx, ch),
            Self::Stack(s) => match s.try_insert(idx, ch) {
                Ok(()) => (),
                Err(pascal_string::InsertError::TooLong) => self.ensure_heap_mut().insert(idx, ch),
                Err(_) => panic!("invalid index or char boundary"),
            },
        }
    }

    #[inline]
    pub fn insert_str(&mut self, idx: usize, string: &str) {
        match self {
            Self::Heap(s) => s.insert_str(idx, string),
            Self::Stack(s) => match s.try_insert_str(idx, string) {
                Ok(()) => (),
                Err(pascal_string::InsertError::TooLong) => {
                    self.ensure_heap_mut().insert_str(idx, string)
                }
                Err(_) => panic!("invalid index or char boundary"),
            },
        }
    }

    /// Inserts a string slice, truncating when stored on stack; returns the remainder that did not fit.
    ///
    /// - If this value is stored on the heap, insertion is complete and the remainder is always `""`.
    /// - If this value is stored on the stack, insertion is best-effort and the remainder is returned.
    ///
    /// Panics if `idx` is out of bounds or not on a UTF-8 boundary (matches `String` semantics).
    #[inline]
    pub fn insert_str_truncated<'s>(&mut self, idx: usize, string: &'s str) -> &'s str {
        self.try_insert_str_truncated(idx, string)
            .expect("invalid index or char boundary")
    }

    /// Non-panicking variant of `insert_str_truncated`.
    ///
    /// Returns `InsertError` on invalid indices/boundaries.
    #[inline]
    pub fn try_insert_str_truncated<'s>(
        &mut self,
        idx: usize,
        string: &'s str,
    ) -> Result<&'s str, pascal_string::InsertError> {
        match self {
            Self::Heap(s) => {
                let len = s.len();
                if idx > len {
                    return Err(pascal_string::InsertError::OutOfBounds { idx, len });
                }
                if !s.is_char_boundary(idx) {
                    return Err(pascal_string::InsertError::NotCharBoundary { idx });
                }
                s.insert_str(idx, string);
                Ok("")
            }
            Self::Stack(s) => s.try_insert_str_truncated(idx, string),
        }
    }

    #[inline]
    pub fn remove(&mut self, idx: usize) -> char {
        match self {
            Self::Heap(s) => s.remove(idx),
            Self::Stack(s) => s.remove(idx),
        }
    }

    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(char) -> bool,
    {
        match self {
            Self::Heap(s) => s.retain(f),
            Self::Stack(s) => {
                // `retain` can only keep or delete existing characters, so it cannot overflow capacity.
                let mut out = PascalString::<N>::new();
                for ch in s.as_str().chars() {
                    if f(ch) {
                        out.try_push(ch).expect("retain cannot overflow");
                    }
                }
                *s = out;
            }
        }
    }

    #[inline]
    pub fn drain<R>(&mut self, range: R) -> std::string::Drain<'_>
    where
        R: std::ops::RangeBounds<usize>,
    {
        self.ensure_heap_mut().drain(range)
    }

    #[inline]
    pub fn split_off(&mut self, at: usize) -> Self {
        match self {
            Self::Heap(s) => {
                let len = s.len();
                assert!(at <= len, "index out of bounds");
                assert!(s.is_char_boundary(at), "index is not a char boundary");

                // Optimization: if the tail fits on the stack, avoid allocating a new `String` via
                // `String::split_off`. Instead, copy the tail into a `PascalString` and truncate in place.
                let tail = &s[at..];
                if tail.len() <= N {
                    let other = PascalString::try_from(tail)
                        .expect("tail length checked against stack capacity");
                    s.truncate(at);
                    return SmartString::Stack(other);
                }

                SmartString::Heap(s.split_off(at))
            }
            Self::Stack(s) => {
                let len = s.len();
                assert!(at <= len, "index out of bounds");
                assert!(s.is_char_boundary(at), "index is not a char boundary");
                let tail = &s.as_str()[at..];
                let other = PascalString::try_from(tail)
                    .expect("tail of a PascalString must fit within the same capacity");
                s.truncate(at);
                SmartString::Stack(other)
            }
        }
    }

    #[inline]
    pub fn replace_range<R>(&mut self, range: R, replace_with: &str)
    where
        R: std::ops::RangeBounds<usize>,
    {
        let s = self.as_str();
        let len = s.len();

        let start = match range.start_bound() {
            std::ops::Bound::Included(&n) => n,
            std::ops::Bound::Excluded(&n) => n + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&n) => n + 1,
            std::ops::Bound::Excluded(&n) => n,
            std::ops::Bound::Unbounded => len,
        };

        match self {
            Self::Heap(s) => s.replace_range(start..end, replace_with),
            Self::Stack(ps) => match ps.try_replace_range_bounds(start, end, replace_with) {
                Ok(()) => (),
                Err(pascal_string::ReplaceRangeError::TooLong) => {
                    self.ensure_heap_mut()
                        .replace_range(start..end, replace_with);
                }
                Err(_) => panic!("invalid range or char boundary"),
            },
        }
    }

    /// Copies the byte range `src` from `self` and appends it to the end.
    ///
    /// Panics if the range is out of bounds or not on UTF-8 char boundaries.
    pub fn extend_from_within<R: std::ops::RangeBounds<usize>>(&mut self, src: R) {
        use std::ops::Bound;
        let len = self.len();
        let start = match src.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n + 1,
            Bound::Unbounded => 0,
        };
        let end = match src.end_bound() {
            Bound::Included(&n) => n + 1,
            Bound::Excluded(&n) => n,
            Bound::Unbounded => len,
        };
        assert!(
            start <= end,
            "extend_from_within: start ({start}) > end ({end})"
        );
        assert!(end <= len, "extend_from_within: end ({end}) > len ({len})");
        let s = self.as_str();
        assert!(
            s.is_char_boundary(start),
            "extend_from_within: start not on char boundary"
        );
        assert!(
            s.is_char_boundary(end),
            "extend_from_within: end not on char boundary"
        );
        // Extract the slice as an owned string to avoid borrow conflict
        let to_append = self.as_str()[start..end].to_string();
        self.push_str(&to_append);
    }

    /// Removes all non-overlapping occurrences of `pat` from the string in-place.
    ///
    /// If `pat` is empty, this is a no-op.
    pub fn remove_matches(&mut self, pat: &str) {
        if pat.is_empty() {
            return;
        }
        match self {
            Self::Stack(s) => s.remove_matches(pat),
            Self::Heap(s) => {
                // Manual implementation since String::remove_matches is post-MSRV (1.59).
                let mut result = String::with_capacity(s.len());
                let mut src = 0;
                let pat_len = pat.len();
                while src < s.len() {
                    if s[src..].starts_with(pat) {
                        src += pat_len;
                    } else {
                        let ch = s[src..].chars().next().unwrap();
                        result.push(ch);
                        src += ch.len_utf8();
                    }
                }
                *s = result;
            }
        }
    }

    /// Removes all occurrences of the character `pat` from the string in-place.
    ///
    /// This is a convenience wrapper around `retain`.
    #[inline]
    pub fn remove_matches_char(&mut self, pat: char) {
        self.retain(|c| c != pat);
    }

    /// Replaces the first occurrence of `pat` with `replacement`.
    ///
    /// If `pat` is not found, `self` is unchanged.
    #[inline]
    pub fn replace_first(&mut self, pat: &str, replacement: &str) {
        if let Some(start) = self.as_str().find(pat) {
            let end = start + pat.len();
            self.replace_range(start..end, replacement);
        }
    }

    /// Replaces the first occurrence of the character `pat` with `replacement`.
    ///
    /// If `pat` is not found, `self` is unchanged.
    #[inline]
    pub fn replace_first_char(&mut self, pat: char, replacement: &str) {
        if let Some(start) = self.as_str().find(pat) {
            let end = start + pat.len_utf8();
            self.replace_range(start..end, replacement);
        }
    }

    /// Replaces the last occurrence of `pat` with `replacement`.
    ///
    /// If `pat` is not found, `self` is unchanged.
    #[inline]
    pub fn replace_last(&mut self, pat: &str, replacement: &str) {
        if let Some(start) = self.as_str().rfind(pat) {
            let end = start + pat.len();
            self.replace_range(start..end, replacement);
        }
    }

    /// Replaces the last occurrence of the character `pat` with `replacement`.
    ///
    /// If `pat` is not found, `self` is unchanged.
    #[inline]
    pub fn replace_last_char(&mut self, pat: char, replacement: &str) {
        if let Some(start) = self.as_str().rfind(pat) {
            let end = start + pat.len_utf8();
            self.replace_range(start..end, replacement);
        }
    }

    /// Decode a UTF-16BE byte slice into a `SmartString`.
    ///
    /// Takes `&[u8]` (raw bytes in big-endian order), matching the upstream
    /// `String::from_utf16be` signature.
    ///
    /// Returns [`Utf16DecodeError`] if the input contains an unpaired surrogate or
    /// an odd number of bytes.
    pub fn from_utf16be(v: &[u8]) -> Result<Self, Utf16DecodeError> {
        decode_utf16_bytes(v, u16::from_be_bytes, false)
    }

    /// Decode a UTF-16BE byte slice into a `SmartString`, replacing invalid data with U+FFFD.
    ///
    /// Accepts an odd trailing byte or unpaired surrogates and substitutes U+FFFD in their place.
    #[must_use]
    pub fn from_utf16be_lossy(v: &[u8]) -> Self {
        decode_utf16_bytes(v, u16::from_be_bytes, true).unwrap()
    }

    /// Decode a UTF-16LE byte slice into a `SmartString`.
    ///
    /// Takes `&[u8]` (raw bytes in little-endian order), matching the upstream
    /// `String::from_utf16le` signature.
    ///
    /// Returns [`Utf16DecodeError`] if the input contains an unpaired surrogate or
    /// an odd number of bytes.
    pub fn from_utf16le(v: &[u8]) -> Result<Self, Utf16DecodeError> {
        decode_utf16_bytes(v, u16::from_le_bytes, false)
    }

    /// Decode a UTF-16LE byte slice into a `SmartString`, replacing invalid data with U+FFFD.
    ///
    /// Accepts an odd trailing byte or unpaired surrogates and substitutes U+FFFD in their place.
    #[must_use]
    pub fn from_utf16le_lossy(v: &[u8]) -> Self {
        decode_utf16_bytes(v, u16::from_le_bytes, true).unwrap()
    }
}

// -- UTF-16 decode helper -------------------------------------------------------------------------

/// Decode a UTF-16 byte sequence with explicit byte order into a `SmartString`.
///
/// `to_u16` converts a pair of bytes into a `u16` value (BE or LE).
/// If `lossy` is `true`, invalid surrogates and odd trailing bytes are replaced with U+FFFD.
/// If `lossy` is `false`, returns `Err` on any invalid surrogate sequence or odd byte count.
fn decode_utf16_bytes<const N: usize>(
    v: &[u8],
    to_u16: fn([u8; 2]) -> u16,
    lossy: bool,
) -> Result<SmartString<N>, Utf16DecodeError> {
    let pair_count = v.len() / 2;
    let mut buf = String::with_capacity(pair_count); // at least one char per code unit

    let mut i = 0;
    while i < pair_count {
        let code_unit = to_u16([v[i * 2], v[i * 2 + 1]]);

        if (0xD800..=0xDBFF).contains(&code_unit) {
            // High surrogate — must be followed by a low surrogate.
            if i + 1 < pair_count {
                let low = to_u16([v[(i + 1) * 2], v[(i + 1) * 2 + 1]]);
                if (0xDC00..=0xDFFF).contains(&low) {
                    // Valid surrogate pair.
                    let cp = 0x10000 + ((code_unit as u32 - 0xD800) << 10) + (low as u32 - 0xDC00);
                    if let Some(ch) = char::from_u32(cp) {
                        buf.push(ch);
                    } else if lossy {
                        buf.push('\u{FFFD}');
                    } else {
                        return Err(Utf16DecodeError::new());
                    }
                    i += 2;
                    continue;
                }
            }
            // Unpaired high surrogate.
            if lossy {
                buf.push('\u{FFFD}');
            } else {
                return Err(Utf16DecodeError::new());
            }
        } else if (0xDC00..=0xDFFF).contains(&code_unit) {
            // Unpaired low surrogate.
            if lossy {
                buf.push('\u{FFFD}');
            } else {
                return Err(Utf16DecodeError::new());
            }
        } else {
            // BMP character — all values in 0x0000..=0xFFFF excluding surrogates are valid Unicode.
            // SAFETY: values 0x0000–0xD7FF and 0xE000–0xFFFF are all valid Unicode scalar values.
            let ch = unsafe { char::from_u32_unchecked(code_unit as u32) };
            buf.push(ch);
        }
        i += 1;
    }

    // Odd trailing byte — not a complete code unit.
    if v.len() % 2 != 0 {
        if lossy {
            buf.push('\u{FFFD}');
        } else {
            return Err(Utf16DecodeError::new());
        }
    }

    Ok(SmartString::from(buf.as_str()))
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

impl<const N: usize> AsMut<str> for SmartString<N> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut str {
        self
    }
}

impl<const N: usize> BorrowMut<str> for SmartString<N> {
    #[inline(always)]
    fn borrow_mut(&mut self) -> &mut str {
        self
    }
}

// -- Conversion -----------------------------------------------------------------------------------

impl<const N: usize> From<String> for SmartString<N> {
    #[inline]
    fn from(s: String) -> Self {
        Self::Heap(s)
    }
}

impl<const N: usize> From<SmartString<N>> for String {
    #[inline]
    fn from(s: SmartString<N>) -> Self {
        match s {
            SmartString::Heap(s) => s,
            SmartString::Stack(s) => s.to_string(),
        }
    }
}

impl<const N: usize> std::str::FromStr for SmartString<N> {
    type Err = Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
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

impl<const N: usize> From<char> for SmartString<N> {
    #[inline]
    fn from(ch: char) -> Self {
        let mut s = Self::new();
        s.push(ch);
        s
    }
}

impl<const N: usize> From<&String> for SmartString<N> {
    #[inline]
    fn from(s: &String) -> Self {
        Self::from(s.as_str())
    }
}

impl<const N: usize> From<&mut str> for SmartString<N> {
    #[inline]
    fn from(s: &mut str) -> Self {
        Self::from(&*s)
    }
}

impl<const N: usize> From<Box<str>> for SmartString<N> {
    #[inline]
    fn from(s: Box<str>) -> Self {
        Self::from(s.as_ref())
    }
}

impl<const N: usize> From<&Box<str>> for SmartString<N> {
    #[inline]
    fn from(s: &Box<str>) -> Self {
        Self::from(s.as_ref())
    }
}

impl<const N: usize> From<Rc<str>> for SmartString<N> {
    #[inline]
    fn from(s: Rc<str>) -> Self {
        Self::from(s.as_ref())
    }
}

impl<const N: usize> From<&Rc<str>> for SmartString<N> {
    #[inline]
    fn from(s: &Rc<str>) -> Self {
        Self::from(s.as_ref())
    }
}

impl<const N: usize> From<Arc<str>> for SmartString<N> {
    #[inline]
    fn from(s: Arc<str>) -> Self {
        Self::from(s.as_ref())
    }
}

impl<const N: usize> From<&Arc<str>> for SmartString<N> {
    #[inline]
    fn from(s: &Arc<str>) -> Self {
        Self::from(s.as_ref())
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

impl<const N: usize> From<&Cow<'_, str>> for SmartString<N> {
    #[inline]
    fn from(s: &Cow<'_, str>) -> Self {
        Self::from(s.as_ref())
    }
}

impl<const N: usize> FromIterator<char> for SmartString<N> {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let mut s = Self::new();
        s.extend(iter);
        s
    }
}

impl<'a, const N: usize> FromIterator<&'a str> for SmartString<N> {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let mut s = Self::new();
        s.extend(iter);
        s
    }
}

impl<const N: usize> Extend<char> for SmartString<N> {
    #[inline]
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        for ch in iter {
            self.push(ch);
        }
    }
}

impl<'a, const N: usize> Extend<&'a str> for SmartString<N> {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        for s in iter {
            self.push_str(s);
        }
    }
}

impl<'a, const N: usize> Extend<&'a char> for SmartString<N> {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a char>>(&mut self, iter: T) {
        for ch in iter {
            self.push(*ch);
        }
    }
}

impl<const N: usize> Extend<String> for SmartString<N> {
    #[inline]
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        for s in iter {
            self.push_str(&s);
        }
    }
}

impl<'a, const N: usize> Extend<&'a String> for SmartString<N> {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a String>>(&mut self, iter: T) {
        for s in iter {
            self.push_str(s.as_str());
        }
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

impl<const N: usize> From<SmartString<N>> for Box<str> {
    #[inline]
    fn from(s: SmartString<N>) -> Self {
        s.into_boxed_str()
    }
}

impl<const N: usize> From<SmartString<N>> for Vec<u8> {
    #[inline]
    fn from(s: SmartString<N>) -> Self {
        s.into_bytes()
    }
}

impl<const N: usize> From<SmartString<N>> for Rc<str> {
    #[inline]
    fn from(s: SmartString<N>) -> Self {
        // NOTE: converting an owned string into Rc/Arc necessarily allocates an Rc/Arc-managed buffer.
        // We go through `String` here for correctness and std-like ergonomics; if this turns out hot,
        // we can evaluate alternative paths and document the cost model.
        Rc::from(s.into_string())
    }
}

impl<const N: usize> From<SmartString<N>> for Arc<str> {
    #[inline]
    fn from(s: SmartString<N>) -> Self {
        // See note on `Rc<str>` above.
        Arc::from(s.into_string())
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

impl<const N: usize, T: ops::Deref<Target = str>> ops::AddAssign<T> for SmartString<N> {
    #[inline]
    fn add_assign(&mut self, rhs: T) {
        self.push_str(rhs.deref());
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

        // NOTE: the enum layout for very small capacities depends on rustc version.
        // Newer compilers can represent these as 24 bytes on 64-bit platforms, while older compilers
        // (including our MSRV) may use 32 bytes. Both are acceptable; the important property is that
        // the default type stays small (32 bytes) and larger capacities grow in pointer-sized steps.
        let small_sizes = [
            mem::size_of::<SmartString<0>>(),
            mem::size_of::<SmartString<1>>(),
            mem::size_of::<SmartString<15>>(),
        ];
        for size in small_sizes {
            assert!(
                size == 24 || size == 32,
                "unexpected SmartString small size: {size}"
            );
        }

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

    #[test]
    fn test_try_reserve_exact_transitions_stack_to_heap() {
        let mut s = SmartString::<4>::from("ab");
        assert!(s.is_stack());

        // Exceeds remaining stack capacity => transition to heap.
        s.try_reserve_exact(3).unwrap();
        assert!(s.is_heap());
        assert_eq!(s.as_str(), "ab");
    }

    #[test]
    fn test_extend_str_transitions_stack_to_heap() {
        let mut s = SmartString::<4>::new();
        s.extend(["ab", "cd"]);
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "abcd");

        s.extend(["e"]);
        assert!(s.is_heap());
        assert_eq!(s.as_str(), "abcde");
    }

    #[test]
    fn test_extend_char_unicode_boundaries() {
        let mut s = SmartString::<4>::new();
        s.extend(['€', 'a']); // 3 + 1 bytes
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "€a");

        s.extend(['b']);
        assert!(s.is_heap());
        assert_eq!(s.as_str(), "€ab");
    }

    #[test]
    fn test_add_assign() {
        let mut s = SmartString::<4>::from("a");
        s += "bcd";
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "abcd");

        s += "e";
        assert!(s.is_heap());
        assert_eq!(s.as_str(), "abcde");
    }

    #[test]
    fn test_insert_and_remove_promotes_to_heap() {
        let mut s = SmartString::<8>::from("ab");
        assert!(s.is_stack());

        s.insert(1, '€');
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "a€b");

        let removed = s.remove(1);
        assert_eq!(removed, '€');
        assert_eq!(s.as_str(), "ab");
    }

    #[test]
    fn test_insert_promotes_to_heap_when_overflow() {
        let mut s = SmartString::<4>::from("ab");
        assert!(s.is_stack());

        // inserting "€" (3 bytes) into "ab" (2 bytes) => 5 bytes, overflows stack cap => heap
        s.insert(1, '€');
        assert!(s.is_heap());
        assert_eq!(s.as_str(), "a€b");
    }

    #[test]
    fn test_insert_str_truncated_on_stack() {
        let mut s = SmartString::<4>::from("ab");
        assert!(s.is_stack());

        // insert at idx=1: only 2 bytes available, so only "cd" fits (not "cde").
        let rem = s.insert_str_truncated(1, "cde");
        assert_eq!(s.as_str(), "acdb");
        assert_eq!(rem, "e");
        assert!(s.is_stack());
    }

    #[test]
    fn test_split_off_returns_stack_when_possible() {
        let mut s = SmartString::<8>::from("hello!");
        assert!(s.is_stack());

        let other = s.split_off(5);
        assert_eq!(s.as_str(), "hello");
        assert_eq!(other.as_str(), "!");
        assert!(other.is_stack());
        assert!(s.is_stack());
    }

    #[test]
    fn test_split_off_on_heap_avoids_alloc_when_tail_fits_stack() {
        let mut s = SmartString::<4>::from("abcde"); // heap (len 5 > 4)
        assert!(s.is_heap());

        let other = s.split_off(4); // tail is "e" (fits stack)
        assert_eq!(s.as_str(), "abcd");
        assert!(s.is_heap()); // no implicit demotion
        assert_eq!(other.as_str(), "e");
        assert!(other.is_stack());
    }

    #[test]
    fn test_retain_keeps_stack_when_possible() {
        let mut s = SmartString::<8>::from("a1b2c3");
        assert!(s.is_stack());
        s.retain(|ch| ch.is_ascii_alphabetic());
        assert_eq!(s.as_str(), "abc");
        assert!(s.is_stack());
    }

    #[test]
    fn test_replace_range() {
        let mut s = SmartString::<8>::from("ab");
        s.replace_range(1..1, "cd");
        assert_eq!(s.as_str(), "acdb");
        assert!(s.is_stack());
    }

    #[test]
    fn test_len_and_is_empty() {
        let s = SmartString::<4>::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);

        let s = SmartString::<4>::from("ab");
        assert!(!s.is_empty());
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn test_from_string_refs_and_smart_extend_refs() {
        let base = String::from("ab");
        let s = SmartString::<4>::from(&base);
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "ab");

        let mut s = SmartString::<4>::new();
        let euro = '€';
        let a = 'a';
        s.extend([&euro, &a]);
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "€a");

        let b = String::from("b");
        s.extend([&b]);
        assert!(s.is_heap());
        assert_eq!(s.as_str(), "€ab");
    }

    #[test]
    fn test_into_boxed_str() {
        let boxed = SmartString::<4>::from("ab").into_boxed_str();
        assert_eq!(&*boxed, "ab");
    }

    #[test]
    fn test_leak() {
        let leaked: &'static mut str = SmartString::<4>::from("ab").leak();
        leaked.make_ascii_uppercase();
        assert_eq!(leaked, "AB");
    }

    #[test]
    fn test_from_utf8_lossy() {
        let s = SmartString::<4>::from_utf8_lossy(&[0x66, 0x6f, 0x6f]);
        assert_eq!(s, "foo");

        let s = SmartString::<4>::from_utf8_lossy(&[0xff]);
        assert!(matches!(s, Cow::Owned(_)));
    }

    #[test]
    fn test_from_utf8_lossy_owned_picks_stack_when_possible() {
        let s = SmartString::<4>::from_utf8_lossy_owned(b"ab".to_vec());
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "ab");

        // U+FFFD is 3 bytes, so it fits into capacity=4.
        let s = SmartString::<4>::from_utf8_lossy_owned(vec![0xff]);
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "�");
    }

    #[test]
    fn test_try_with_capacity_picks_stack_or_heap() {
        let s = SmartString::<4>::try_with_capacity(3).unwrap();
        assert!(s.is_stack());
        assert_eq!(s.capacity(), 4);

        let s = SmartString::<4>::try_with_capacity(10).unwrap();
        assert!(s.is_heap());
        assert!(s.capacity() >= 10);
    }

    #[test]
    fn test_from_char_picks_stack_or_heap() {
        let s = SmartString::<4>::from('€'); // 3 bytes
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "€");

        let s = SmartString::<2>::from('€'); // won't fit (3 bytes)
        assert!(s.is_heap());
        assert_eq!(s.as_str(), "€");
    }

    #[test]
    fn test_from_ref_str_containers_and_into_box_str() {
        let b: Box<str> = "ab".into();
        let r: Rc<str> = Rc::from("ab");
        let a: Arc<str> = Arc::from("ab");

        assert_eq!(SmartString::<4>::from(&b).as_str(), "ab");
        assert_eq!(SmartString::<4>::from(&r).as_str(), "ab");
        assert_eq!(SmartString::<4>::from(&a).as_str(), "ab");

        let boxed: Box<str> = SmartString::<4>::from("ab").into();
        assert_eq!(&*boxed, "ab");
    }

    #[test]
    fn test_from_cow_ref() {
        let borrowed: Cow<'_, str> = Cow::Borrowed("ab");
        let owned: Cow<'_, str> = Cow::Owned(String::from("ab"));
        assert_eq!(SmartString::<4>::from(&borrowed).as_str(), "ab");
        assert_eq!(SmartString::<4>::from(&owned).as_str(), "ab");
    }

    #[test]
    fn test_into_vec_u8_rc_arc_str() {
        let bytes: Vec<u8> = SmartString::<4>::from("ab").into();
        assert_eq!(bytes, b"ab");

        let rc: Rc<str> = SmartString::<4>::from("ab").into();
        assert_eq!(&*rc, "ab");

        let arc: Arc<str> = SmartString::<4>::from("ab").into();
        assert_eq!(&*arc, "ab");
    }

    // -- from_utf16be / from_utf16le tests --------------------------------------------------------

    #[test]
    fn test_utf16be_ascii_hi_lands_on_stack() {
        // "Hi" in UTF-16BE: U+0048, U+0069
        let bytes = [0x00u8, 0x48, 0x00, 0x69];
        let s = SmartString::<30>::from_utf16be(&bytes).unwrap();
        assert_eq!(s.as_str(), "Hi");
        assert!(s.is_stack());
    }

    #[test]
    fn test_utf16be_long_string_lands_on_heap() {
        // 20 BMP characters encoded as UTF-16BE = 40 bytes; each is 1–3 UTF-8 bytes.
        // Use ASCII codepoints so the total UTF-8 size equals 20 bytes, which still fits on the
        // default 30-byte stack.  Instead use non-ASCII multi-byte chars to force heap:
        // U+4F60 ("你") = 3 UTF-8 bytes.  11 copies → 33 UTF-8 bytes > 30-byte stack.
        let code_unit: u16 = 0x4F60; // 你
        let mut bytes = Vec::new();
        for _ in 0..11 {
            bytes.extend_from_slice(&code_unit.to_be_bytes());
        }
        let s = SmartString::<30>::from_utf16be(&bytes).unwrap();
        assert_eq!(s.as_str(), "你你你你你你你你你你你");
        assert!(s.is_heap());
    }

    #[test]
    fn test_utf16be_surrogate_pair_musical_symbol() {
        // U+1D11E MUSICAL SYMBOL G CLEF encoded as UTF-16BE surrogate pair: D834 DD1E
        let bytes = [0xD8u8, 0x34, 0xDD, 0x1E];
        let s = SmartString::<30>::from_utf16be(&bytes).unwrap();
        assert_eq!(s.as_str(), "\u{1D11E}");
    }

    #[test]
    fn test_utf16be_unpaired_high_surrogate_is_err() {
        // High surrogate D800 with no following low surrogate.
        let bytes = [0xD8u8, 0x00, 0x00, 0x41]; // D800, then 'A'
        assert!(SmartString::<30>::from_utf16be(&bytes).is_err());
    }

    #[test]
    fn test_utf16be_unpaired_low_surrogate_is_err() {
        // Low surrogate DC00 with no preceding high surrogate.
        let bytes = [0xDCu8, 0x00, 0x00, 0x41]; // DC00, then 'A'
        assert!(SmartString::<30>::from_utf16be(&bytes).is_err());
    }

    #[test]
    fn test_utf16be_empty_input() {
        let s = SmartString::<30>::from_utf16be(&[]).unwrap();
        assert_eq!(s.as_str(), "");
        assert!(s.is_stack());
    }

    #[test]
    fn test_utf16be_odd_byte_count_is_err() {
        let bytes = [0x00u8, 0x48, 0x00]; // "H" + one trailing byte
        assert!(SmartString::<30>::from_utf16be(&bytes).is_err());
    }

    #[test]
    fn test_utf16be_bmp_chinese() {
        // "你好" in UTF-16BE: U+4F60, U+597D
        let bytes = [0x4Fu8, 0x60, 0x59, 0x7D];
        let s = SmartString::<30>::from_utf16be(&bytes).unwrap();
        assert_eq!(s.as_str(), "你好");
    }

    // -- from_utf16be_lossy -----------------------------------------------------------------------

    #[test]
    fn test_utf16be_lossy_valid_input_unchanged() {
        let bytes = [0x00u8, 0x48, 0x00, 0x69]; // "Hi"
        let s = SmartString::<30>::from_utf16be_lossy(&bytes);
        assert_eq!(s.as_str(), "Hi");
    }

    #[test]
    fn test_utf16be_lossy_unpaired_high_surrogate_replaced() {
        // High surrogate D800 followed by non-surrogate 'A'.
        let bytes = [0xD8u8, 0x00, 0x00, 0x41];
        let s = SmartString::<30>::from_utf16be_lossy(&bytes);
        assert_eq!(s.as_str(), "\u{FFFD}A");
    }

    #[test]
    fn test_utf16be_lossy_unpaired_low_surrogate_replaced() {
        let bytes = [0xDCu8, 0x00, 0x00, 0x41]; // DC00, 'A'
        let s = SmartString::<30>::from_utf16be_lossy(&bytes);
        assert_eq!(s.as_str(), "\u{FFFD}A");
    }

    #[test]
    fn test_utf16be_lossy_odd_trailing_byte_replaced() {
        let bytes = [0x00u8, 0x48, 0xFF]; // 'H' + one orphan byte
        let s = SmartString::<30>::from_utf16be_lossy(&bytes);
        assert_eq!(s.as_str(), "H\u{FFFD}");
    }

    // -- from_utf16le / from_utf16le_lossy -------------------------------------------------------

    #[test]
    fn test_utf16le_ascii_hi_lands_on_stack() {
        // "Hi" in UTF-16LE: U+0048, U+0069 → bytes reversed per code unit
        let bytes = [0x48u8, 0x00, 0x69, 0x00];
        let s = SmartString::<30>::from_utf16le(&bytes).unwrap();
        assert_eq!(s.as_str(), "Hi");
        assert!(s.is_stack());
    }

    #[test]
    fn test_utf16le_long_string_lands_on_heap() {
        // Same heap test as BE, but with LE byte order.
        let code_unit: u16 = 0x4F60; // 你
        let mut bytes = Vec::new();
        for _ in 0..11 {
            bytes.extend_from_slice(&code_unit.to_le_bytes());
        }
        let s = SmartString::<30>::from_utf16le(&bytes).unwrap();
        assert_eq!(s.as_str(), "你你你你你你你你你你你");
        assert!(s.is_heap());
    }

    #[test]
    fn test_utf16le_surrogate_pair_musical_symbol() {
        // U+1D11E in UTF-16LE: 34D8 1EDD
        let bytes = [0x34u8, 0xD8, 0x1E, 0xDD];
        let s = SmartString::<30>::from_utf16le(&bytes).unwrap();
        assert_eq!(s.as_str(), "\u{1D11E}");
    }

    #[test]
    fn test_utf16le_unpaired_high_surrogate_is_err() {
        let bytes = [0x00u8, 0xD8, 0x41, 0x00]; // D800 LE, then 'A' LE
        assert!(SmartString::<30>::from_utf16le(&bytes).is_err());
    }

    #[test]
    fn test_utf16le_unpaired_low_surrogate_is_err() {
        let bytes = [0x00u8, 0xDC, 0x41, 0x00]; // DC00 LE, then 'A' LE
        assert!(SmartString::<30>::from_utf16le(&bytes).is_err());
    }

    #[test]
    fn test_utf16le_empty_input() {
        let s = SmartString::<30>::from_utf16le(&[]).unwrap();
        assert_eq!(s.as_str(), "");
    }

    #[test]
    fn test_utf16le_odd_byte_count_is_err() {
        let bytes = [0x48u8, 0x00, 0x00]; // 'H' LE + one trailing byte
        assert!(SmartString::<30>::from_utf16le(&bytes).is_err());
    }

    #[test]
    fn test_utf16le_bmp_chinese() {
        // "你好" in UTF-16LE
        let bytes = [0x60u8, 0x4F, 0x7D, 0x59];
        let s = SmartString::<30>::from_utf16le(&bytes).unwrap();
        assert_eq!(s.as_str(), "你好");
    }

    #[test]
    fn test_utf16le_lossy_unpaired_surrogate_replaced() {
        let bytes = [0x00u8, 0xD8, 0x41, 0x00]; // D800 LE + 'A' LE
        let s = SmartString::<30>::from_utf16le_lossy(&bytes);
        assert_eq!(s.as_str(), "\u{FFFD}A");
    }

    #[test]
    fn test_utf16le_lossy_odd_trailing_byte_replaced() {
        let bytes = [0x48u8, 0x00, 0xFF]; // 'H' LE + orphan
        let s = SmartString::<30>::from_utf16le_lossy(&bytes);
        assert_eq!(s.as_str(), "H\u{FFFD}");
    }

    #[test]
    fn test_utf16_decode_error_display() {
        let err = Utf16DecodeError::new();
        assert_eq!(err.to_string(), "invalid utf-16: lone surrogate found");
    }

    #[test]
    fn test_utf16be_stack_vs_heap_awareness() {
        // 2 chars × 2 bytes/char = 4 bytes input → 2 UTF-8 bytes → fits on a 4-byte stack.
        let bytes = [0x00u8, 0x48, 0x00, 0x69]; // "Hi"
        let stack = SmartString::<4>::from_utf16be(&bytes).unwrap();
        assert!(stack.is_stack());

        // 3 × U+4F60 = 9 UTF-8 bytes > 4-byte stack → heap.
        let code_unit: u16 = 0x4F60;
        let mut long_bytes = Vec::new();
        for _ in 0..3 {
            long_bytes.extend_from_slice(&code_unit.to_be_bytes());
        }
        let heap = SmartString::<4>::from_utf16be(&long_bytes).unwrap();
        assert!(heap.is_heap());
    }

    // -- extend_from_within tests -------------------------------------------------------------------

    #[test]
    fn test_extend_from_within_stack() {
        let mut s = SmartString::<10>::from("abc");
        s.extend_from_within(0..2);
        assert_eq!(s.as_str(), "abcab");
        assert!(s.is_stack());
    }

    #[test]
    fn test_extend_from_within_promotes_to_heap() {
        let mut s = SmartString::<4>::from("abc");
        s.extend_from_within(..); // "abcabc" = 6 bytes > 4
        assert_eq!(s.as_str(), "abcabc");
        assert!(s.is_heap());
    }

    #[test]
    fn test_extend_from_within_heap() {
        let mut s = SmartString::<2>::from("hello");
        assert!(s.is_heap());
        s.extend_from_within(1..3);
        assert_eq!(s.as_str(), "helloel");
    }

    #[test]
    fn test_extend_from_within_empty_range() {
        let mut s = SmartString::<10>::from("abc");
        s.extend_from_within(1..1);
        assert_eq!(s.as_str(), "abc");
    }

    #[test]
    fn test_extend_from_within_full_range() {
        let mut s = SmartString::<10>::from("ab");
        s.extend_from_within(..);
        assert_eq!(s.as_str(), "abab");
    }

    #[test]
    #[should_panic]
    fn test_extend_from_within_non_char_boundary() {
        let mut s = SmartString::<10>::from("a€b"); // € is 3 bytes
        s.extend_from_within(1..3); // mid-€ boundaries
    }

    #[test]
    #[should_panic]
    fn test_extend_from_within_out_of_bounds() {
        let mut s = SmartString::<10>::from("abc");
        s.extend_from_within(0..10);
    }

    // -- remove_matches tests -----------------------------------------------------------------------

    #[test]
    fn test_remove_matches_stack_single() {
        let mut s = SmartString::<10>::from("abcabc");
        s.remove_matches("b");
        assert_eq!(s.as_str(), "acac");
    }

    #[test]
    fn test_remove_matches_stack_multiple() {
        let mut s = SmartString::<20>::from("aXbXcX");
        s.remove_matches("X");
        assert_eq!(s.as_str(), "abc");
    }

    #[test]
    fn test_remove_matches_no_match() {
        let mut s = SmartString::<10>::from("hello");
        s.remove_matches("xyz");
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_remove_matches_empty_pattern() {
        let mut s = SmartString::<10>::from("hello");
        s.remove_matches("");
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_remove_matches_entire_string() {
        let mut s = SmartString::<10>::from("aaa");
        s.remove_matches("aaa");
        assert_eq!(s.as_str(), "");
    }

    #[test]
    fn test_remove_matches_heap() {
        let mut s = SmartString::<2>::from("hello world");
        assert!(s.is_heap());
        s.remove_matches("o");
        assert_eq!(s.as_str(), "hell wrld");
    }

    #[test]
    fn test_remove_matches_multibyte() {
        let mut s = SmartString::<20>::from("a€b€c");
        s.remove_matches("€");
        assert_eq!(s.as_str(), "abc");
    }

    #[test]
    fn test_remove_matches_char_variant() {
        let mut s = SmartString::<10>::from("abcabc");
        s.remove_matches_char('b');
        assert_eq!(s.as_str(), "acac");
    }

    // -- replace_first / replace_last tests ---------------------------------------------------------

    #[test]
    fn test_replace_first_same_length() {
        let mut s = SmartString::<10>::from("abcabc");
        s.replace_first("b", "X");
        assert_eq!(s.as_str(), "aXcabc");
        assert!(s.is_stack());
    }

    #[test]
    fn test_replace_first_shorter() {
        let mut s = SmartString::<10>::from("abcabc");
        s.replace_first("bc", "Z");
        assert_eq!(s.as_str(), "aZabc");
    }

    #[test]
    fn test_replace_first_longer_fits_stack() {
        let mut s = SmartString::<20>::from("abcabc");
        s.replace_first("b", "XYZ");
        assert_eq!(s.as_str(), "aXYZcabc");
        assert!(s.is_stack());
    }

    #[test]
    fn test_replace_first_longer_promotes_to_heap() {
        let mut s = SmartString::<4>::from("abc");
        s.replace_first("b", "XXXX"); // "aXXXXc" = 6 > 4
        assert_eq!(s.as_str(), "aXXXXc");
        assert!(s.is_heap());
    }

    #[test]
    fn test_replace_first_no_match() {
        let mut s = SmartString::<10>::from("hello");
        s.replace_first("xyz", "!");
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_replace_last_gets_last() {
        let mut s = SmartString::<10>::from("abcabc");
        s.replace_last("b", "X");
        assert_eq!(s.as_str(), "abcaXc");
    }

    #[test]
    fn test_replace_first_vs_last() {
        let mut s1 = SmartString::<20>::from("aXbXcXd");
        let mut s2 = s1.clone();
        s1.replace_first("X", ".");
        s2.replace_last("X", ".");
        assert_eq!(s1.as_str(), "a.bXcXd");
        assert_eq!(s2.as_str(), "aXbXc.d");
    }

    #[test]
    fn test_replace_first_at_boundaries() {
        let mut s = SmartString::<10>::from("abc");
        s.replace_first("a", "X");
        assert_eq!(s.as_str(), "Xbc");

        let mut s = SmartString::<10>::from("abc");
        s.replace_first("c", "X");
        assert_eq!(s.as_str(), "abX");
    }

    #[test]
    fn test_replace_first_char_variant() {
        let mut s = SmartString::<10>::from("a€b€c");
        s.replace_first_char('€', "X");
        assert_eq!(s.as_str(), "aXb€c");
    }

    #[test]
    fn test_replace_last_char_variant() {
        let mut s = SmartString::<10>::from("a€b€c");
        s.replace_last_char('€', "X");
        assert_eq!(s.as_str(), "a€bXc");
    }

    #[test]
    fn test_from_utf16_stack_aware() {
        // "Hi" = 2 bytes UTF-8, should land on stack with capacity 4
        let s = SmartString::<4>::from_utf16(&[0x48u16, 0x69]).unwrap();
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "Hi");
    }

    #[test]
    fn test_from_utf16_lossy_stack_aware() {
        let s = SmartString::<4>::from_utf16_lossy(&[0x48u16, 0x69]);
        assert!(s.is_stack());
        assert_eq!(s.as_str(), "Hi");
    }
}
