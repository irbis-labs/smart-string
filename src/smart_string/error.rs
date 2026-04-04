use core::fmt;

/// Error returned when decoding invalid UTF-16 data.
///
/// This is a crate-local equivalent of `std::string::FromUtf16Error`, which cannot be
/// constructed outside of std.
///
/// Returned by [`super::SmartString::from_utf16be`], [`super::SmartString::from_utf16le`], and their variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utf16DecodeError {
    _private: (),
}

impl Utf16DecodeError {
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

impl fmt::Display for Utf16DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid utf-16: lone surrogate found")
    }
}
