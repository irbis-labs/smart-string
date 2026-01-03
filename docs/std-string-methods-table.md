# `std::string::String` methods — properties table

Extracted from the local Rust toolchain source (`alloc/src/string.rs`) on:

- Rust: `rustc 1.94.0-nightly (2025-12-31)`

This table covers **inherent public methods** on `String` (including `pub const fn` and `pub unsafe fn`).

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
| `as_bytes` | const |  | `&self` | no | O(1) |  |  |
| `as_mut_str` | const |  | `&mut self` | no | O(1) |  |  |
| `as_mut_vec` | const | unsafe | `&mut self` | no | O(1) (unsafe) |  | — (unsafe: must preserve UTF-8) |
| `as_str` | const |  | `&self` | no | O(1) |  |  |
| `capacity` | const |  | `&self` | no | O(1) |  |  |
| `clear` |  |  | `&mut self` | no | O(1) |  |  |
| `drain` |  |  | `&mut self` | no | O(n) |  | range invalid/out of bounds or not char boundary |
| `extend_from_within` |  |  | `&mut self` | may realloc | O(k) |  |  |
| `from_raw_parts` |  | unsafe | — | no (takes Vec parts) | O(1) (unsafe) |  | — (unsafe: UB if invariants not met) |
| `from_utf16` |  |  | — | alloc | O(n) | Result |  |
| `from_utf16_lossy` |  |  | — | alloc | O(n) |  |  |
| `from_utf16be` |  |  | — | alloc | O(n) | Result |  |
| `from_utf16be_lossy` |  |  | — | alloc | O(n) |  |  |
| `from_utf16le` |  |  | — | alloc | O(n) | Result |  |
| `from_utf16le_lossy` |  |  | — | alloc | O(n) |  |  |
| `from_utf8` |  |  | — | no (takes Vec) | O(n) | Result |  |
| `from_utf8_lossy` |  |  | — | may alloc | O(n) |  |  |
| `from_utf8_lossy_owned` |  |  | — | may alloc | O(n) |  |  |
| `from_utf8_unchecked` |  | unsafe | — | no (takes Vec) | O(1) (unsafe) |  | — (unsafe: UB if invalid UTF-8) |
| `insert` |  |  | `&mut self` | may realloc | O(n) |  | idx out of bounds or not char boundary |
| `insert_str` |  |  | `&mut self` | may realloc | O(n) |  | idx out of bounds or not char boundary |
| `into_boxed_str` |  |  | `self` | may realloc | O(1) to O(n) |  |  |
| `into_bytes` | const |  | `self` | no | O(1) |  |  |
| `into_chars` |  |  | `self` | no | O(1) |  |  |
| `into_raw_parts` |  |  | `self` | no | O(1) |  |  |
| `is_empty` | const |  | `&self` | no | O(1) |  |  |
| `leak` |  |  | `self` | no (leaks) | O(1) |  |  |
| `len` | const |  | `&self` | no | O(1) |  |  |
| `new` | const |  | — | no | O(1) |  |  |
| `pop` |  |  | `&mut self` | no | O(1) avg | Option |  |
| `push` |  |  | `&mut self` | may realloc | amortized |  |  |
| `push_str` |  |  | `&mut self` | may realloc | amortized |  |  |
| `remove` |  |  | `&mut self` | no | O(n) |  | idx out of bounds or not char boundary |
| `remove_matches` |  |  | `&mut self` | no | O(n) |  |  |
| `replace_first` |  |  | `&mut self` | may realloc | O(n) |  |  |
| `replace_last` |  |  | `&mut self` | may realloc | O(n) |  |  |
| `replace_range` |  |  | `&mut self` | may realloc | O(n) |  | range invalid/out of bounds or not char boundary |
| `reserve` |  |  | `&mut self` | may realloc | amortized |  |  |
| `reserve_exact` |  |  | `&mut self` | may realloc | O(n) worst |  |  |
| `retain` |  |  | `&mut self` | no | O(n) |  |  |
| `shrink_to` |  |  | `&mut self` | may dealloc | O(n) worst |  |  |
| `shrink_to_fit` |  |  | `&mut self` | may dealloc | O(n) worst |  |  |
| `split_off` |  |  | `&mut self` | alloc | O(n) |  | at out of bounds or not char boundary |
| `truncate` |  |  | `&mut self` | no | O(1) |  | not char boundary |
| `try_reserve` |  |  | `&mut self` | may realloc | amortized | Result |  |
| `try_reserve_exact` |  |  | `&mut self` | may realloc | O(n) worst | Result |  |
| `try_with_capacity` |  |  | — | alloc | O(1) | Result |  |
| `with_capacity` |  |  | — | alloc | O(1) |  |  |


