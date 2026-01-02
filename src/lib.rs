//! `smart-string` is a collection of small string primitives:
//!
//! - [`PascalString`]: fixed-capacity UTF-8 string stored inline (stack / in-place).
//! - [`SmartString`]: stack-or-heap string that promotes to heap when needed.
//! - [`StrStack`]: a compact “stack” of string slices backed by a single byte buffer.
//!
//! ## Notes
//!
//! - `SmartString` promotion (stack → heap) can happen implicitly during mutation when capacity is exceeded.
//! - Demotion (heap → stack) is **explicit** and must be requested via [`SmartString::try_into_stack`].
//! - MSRV (default features): **Rust 1.59.0**.
//!   - Motivation: `SmartString`'s public API uses a default const generic parameter
//!     (`SmartString<const N: usize = DEFAULT_CAPACITY>`), which requires newer compilers.
//!   - Note: MSRV is a `rustc` guarantee for this crate. Without a committed `Cargo.lock`, transitive dependency MSRVs
//!     can drift over time; our CI runs an MSRV job to detect such drift.
mod display_ext;
pub mod pascal_string;
pub mod smart_string;
pub mod str_stack;

pub use crate::display_ext::DisplayExt;
pub use crate::pascal_string::PascalString;
pub use crate::smart_string::SmartString;
pub use crate::str_stack::StrStack;
pub use crate::str_stack::StrStackIter;
