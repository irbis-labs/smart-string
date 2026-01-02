use core::fmt;
use core::str::Utf8Error;

/// An error returned when a conversion from a `&str` to a `PascalString` fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryFromStrError {
    /// The string is too long to fit into a `PascalString`.
    TooLong,
}

/// An error returned by insertion operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertError {
    /// The index is outside the string bounds.
    OutOfBounds { idx: usize, len: usize },
    /// The index is not on a UTF-8 character boundary.
    NotCharBoundary { idx: usize },
    /// The result would exceed the fixed capacity.
    TooLong,
}

/// An error returned by removal operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoveError {
    /// The index is outside the string bounds.
    OutOfBounds { idx: usize, len: usize },
    /// The index is not on a UTF-8 character boundary.
    NotCharBoundary { idx: usize },
}

/// An error returned when a conversion from a `&[u8]` to a `PascalString` fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryFromBytesError {
    /// The string is too long to fit into a `PascalString`.
    TooLong,
    /// The string is not valid UTF-8.
    Utf8Error(Utf8Error),
}

impl From<Utf8Error> for TryFromBytesError {
    fn from(e: Utf8Error) -> Self {
        TryFromBytesError::Utf8Error(e)
    }
}

impl From<TryFromStrError> for TryFromBytesError {
    fn from(e: TryFromStrError) -> Self {
        match e {
            TryFromStrError::TooLong => TryFromBytesError::TooLong,
        }
    }
}

impl fmt::Display for TryFromStrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TryFromStrError::TooLong => f.write_str("string too long"),
        }
    }
}

impl fmt::Display for InsertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InsertError::OutOfBounds { idx, len } => {
                write!(f, "index out of bounds: idx={idx}, len={len}")
            }
            InsertError::NotCharBoundary { idx } => write!(f, "index is not a char boundary: idx={idx}"),
            InsertError::TooLong => f.write_str("string too long"),
        }
    }
}

impl fmt::Display for RemoveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RemoveError::OutOfBounds { idx, len } => {
                write!(f, "index out of bounds: idx={idx}, len={len}")
            }
            RemoveError::NotCharBoundary { idx } => write!(f, "index is not a char boundary: idx={idx}"),
        }
    }
}

impl fmt::Display for TryFromBytesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TryFromBytesError::TooLong => f.write_str("string too long"),
            TryFromBytesError::Utf8Error(e) => e.fmt(f),
        }
    }
}
