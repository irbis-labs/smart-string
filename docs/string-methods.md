# `std::string::String` methods

Extracted from the local Rust toolchain source (`alloc/src/string.rs`) on:

- Rust: `rustc 1.94.0-nightly (2025-12-31)`

Scope:

- This document lists **inherent public methods** on `String` (including `pub const fn` and `pub unsafe fn`).
- `String` also exposes *all* `str` methods via `Deref<Target = str>`; those are not listed here.

## Method list (alphabetical)

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

## Method properties table

Columns:

- **const**: the method is `pub const fn` (cell contains the word `const`).
- **unsafe**: the method is `pub unsafe fn` (cell contains the word `unsafe`).
- **self**: receiver kind: `&self`, `&mut self`, `self`, or `—` for associated functions.
- **alloc**: short note about allocations/reallocations (coarse; optimized for parity work).
- **complexity**: short hint (only where it matters; coarse).
- **fallible**: empty / `Option` / `Result`.
- **panics**: short description of panic conditions (or “unsafe: UB …” where the contract is unsafe rather than panicking).

| String method | const | unsafe | self | alloc | complexity | fallible | panics |
|---|---|---|---|---|---|---|---|
| `as_bytes` | const |  | `&self` |  |  |  |  |
| `as_mut_str` | const |  | `&mut self` |  |  |  |  |
| `as_mut_vec` | const | unsafe | `&mut self` |  |  |  | unsafe: must preserve UTF-8 |
| `as_str` | const |  | `&self` |  |  |  |  |
| `capacity` | const |  | `&self` |  |  |  |  |
| `clear` |  |  | `&mut self` |  |  |  |  |
| `drain` |  |  | `&mut self` |  |  |  | Panics if the range has `start_bound > end_bound`, or, if the range is bounded on either end and does not lie on a [`char`] boundary. |
| `extend_from_within` |  |  | `&mut self` | may realloc |  |  | Panics if the range has `start_bound > end_bound`, if the range is bounded on either end and does not lie on a [`char`] boundary, or if the new capacity exceeds `isize::MAX` bytes. |
| `from_raw_parts` |  | unsafe | — | no (takes Vec parts) |  |  | unsafe: UB if invariants not met |
| `from_utf16` |  |  | — | alloc |  | Result |  |
| `from_utf16_lossy` |  |  | — | alloc |  |  |  |
| `from_utf16be` |  |  | — | alloc |  | Result |  |
| `from_utf16be_lossy` |  |  | — | alloc |  |  |  |
| `from_utf16le` |  |  | — | alloc |  | Result |  |
| `from_utf16le_lossy` |  |  | — | alloc |  |  |  |
| `from_utf8` |  |  | — | no (takes Vec) |  | Result |  |
| `from_utf8_lossy` |  |  | — | may alloc |  |  |  |
| `from_utf8_lossy_owned` |  |  | — | may alloc |  |  |  |
| `from_utf8_unchecked` |  | unsafe | — | no (takes Vec) |  |  | unsafe: UB if invalid UTF-8 |
| `insert` |  |  | `&mut self` | may realloc | Note that calling this in a loop can result in quadratic behavior. |  | Panics if `idx` is larger than the `String`'s length, or if it does not lie on a [`char`] boundary. |
| `insert_str` |  |  | `&mut self` | may realloc | Note that calling this in a loop can result in quadratic behavior. |  | Panics if `idx` is larger than the `String`'s length, or if it does not lie on a [`char`] boundary. |
| `into_boxed_str` |  |  | `self` | may realloc |  |  |  |
| `into_bytes` | const |  | `self` | no |  |  |  |
| `into_chars` |  |  | `self` |  |  |  |  |
| `into_raw_parts` |  |  | `self` | no |  |  |  |
| `is_empty` | const |  | `&self` |  |  |  |  |
| `leak` |  |  | `self` | no (leaks) |  |  |  |
| `len` | const |  | `&self` |  |  |  |  |
| `new` | const |  | — | no |  |  |  |
| `pop` |  |  | `&mut self` |  |  | Option |  |
| `push` |  |  | `&mut self` | may realloc |  |  | Panics if the new capacity exceeds `isize::MAX` _bytes_. |
| `push_str` |  |  | `&mut self` | may realloc |  |  | Panics if the new capacity exceeds `isize::MAX` _bytes_. |
| `remove` |  |  | `&mut self` |  | Note that calling this in a loop can result in quadratic behavior. |  | Panics if `idx` is larger than or equal to the `String`'s length, or if it does not lie on a [`char`] boundary. |
| `remove_matches` |  |  | `&mut self` |  |  |  |  |
| `replace_first` |  |  | `&mut self` | may realloc |  |  |  |
| `replace_last` |  |  | `&mut self` | may realloc |  |  |  |
| `replace_range` |  |  | `&mut self` | may realloc |  |  | Panics if the range has `start_bound > end_bound`, or, if the range is bounded on either end and does not lie on a [`char`] boundary. |
| `reserve` |  |  | `&mut self` | may realloc |  |  | Panics if the new capacity exceeds `isize::MAX` _bytes_. |
| `reserve_exact` |  |  | `&mut self` | may realloc |  |  | Panics if the new capacity exceeds `isize::MAX` _bytes_. |
| `retain` |  |  | `&mut self` |  |  |  |  |
| `shrink_to` |  |  | `&mut self` | may dealloc |  |  |  |
| `shrink_to_fit` |  |  | `&mut self` | may dealloc |  |  |  |
| `split_off` |  |  | `&mut self` | alloc |  |  | Panics if `at` is not on a `UTF-8` code point boundary, or if it is beyond the last code point of the string. |
| `truncate` |  |  | `&mut self` |  |  |  | Panics if `new_len` does not lie on a [`char`] boundary. |
| `try_reserve` |  |  | `&mut self` | may realloc |  | Result |  |
| `try_reserve_exact` |  |  | `&mut self` | may realloc |  | Result |  |
| `try_with_capacity` |  |  | — | alloc |  | Result |  |
| `with_capacity` |  |  | — | alloc |  |  | Panics if the capacity exceeds `isize::MAX` _bytes_. |


