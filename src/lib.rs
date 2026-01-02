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
mod display_ext;
pub mod pascal_string;
pub mod smart_string;
pub mod str_stack;

pub use display_ext::DisplayExt;
pub use pascal_string::PascalString;
pub use smart_string::SmartString;
pub use str_stack::StrStack;
pub use str_stack::StrStackIter;
