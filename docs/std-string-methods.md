# `std::string::String` methods (bullet list)

Extracted from the local Rust toolchain source (`alloc/src/string.rs`) on:

- Rust: `rustc 1.94.0-nightly (2025-12-31)`

This list contains **inherent public methods** on `String` (including `pub const fn` and `pub unsafe fn`), not trait
methods.
Note: `String` also exposes *all* `str` methods via `Deref<Target = str>`; those are not listed here.

 - `as_bytes`
 - `as_mut_str`
 - `as_mut_vec`
 - `as_str`
 - `capacity`
 - `clear`
 - `drain`
 - `extend_from_within`
 - `from_raw_parts`
 - `from_utf16`
 - `from_utf16_lossy`
 - `from_utf16be`
 - `from_utf16be_lossy`
 - `from_utf16le`
 - `from_utf16le_lossy`
 - `from_utf8`
 - `from_utf8_lossy`
 - `from_utf8_lossy_owned`
 - `from_utf8_unchecked`
 - `insert`
 - `insert_str`
 - `into_boxed_str`
 - `into_bytes`
 - `into_chars`
 - `into_raw_parts`
 - `is_empty`
 - `leak`
 - `len`
 - `new`
 - `pop`
 - `push`
 - `push_str`
 - `remove`
 - `remove_matches`
 - `replace_first`
 - `replace_last`
 - `replace_range`
 - `reserve`
 - `reserve_exact`
 - `retain`
 - `shrink_to`
 - `shrink_to_fit`
 - `split_off`
 - `truncate`
 - `try_reserve`
 - `try_reserve_exact`
 - `try_with_capacity`
 - `with_capacity`


