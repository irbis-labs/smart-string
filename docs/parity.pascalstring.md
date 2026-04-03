# `PascalString` ↔ `std::string::String` parity table

Legend:

- **Match**:
  - ✅ = full correspondence is possible (same semantics are achievable for a fixed-capacity string)
  - 🚫 = full correspondence is not possible (due to fixed capacity, lack of owned buffer, or fundamentally different safety model)
- **Implemented**:
  - ✅ = exists
  - 🟡 = partial (some equivalent exists, but semantics/API differ)
  - ❌ = not implemented

Notes:

- `PascalString` is **fixed capacity**, so many `String` “infallible growth” methods cannot have true parity.
- For those, we prefer **non-panicking `try_*` APIs** plus **`*_truncated`** (returning remainder) and explicit panicking
  **`*_expect_capacity`** variants.

| `String` method | Match | PascalString method (full match) | Additional / alternative methods | Implemented |
|---|---:|---|---|---:|
| `as_bytes` | ✅ | `as_bytes` | `as_ref::<[u8]>()` | ✅ |
| `as_mut_str` | ✅ | `as_mut_str` | *(also `DerefMut<Target=str>` provides `&mut str`)* | ✅ |
| `as_mut_vec` | 🚫 | — | *(would expose raw bytes and break UTF‑8 invariant unless heavily constrained / unsafe)* | ❌ |
| `as_str` | ✅ | `as_str` | *(also `Deref<Target=str>`)* | ✅ |
| `capacity` | ✅ | `capacity` | `CAPACITY` const (via type parameter) | ✅ |
| `clear` | ✅ | `clear` | — | ✅ |
| `drain` | 🚫 | — | *(could be emulated by copying into a new buffer; return type/lifetime makes parity awkward)* | ❌ |
| `extend_from_within` | 🚫 | — | *(could be provided as `try_extend_from_within` / `*_truncated` / `*_expect_capacity`)* | ❌ |
| `from_raw_parts` | 🚫 | — | `into_inner` (different) | ❌ |
| `from_utf16` | 🚫 | — | *(could be `try_from_utf16` → `Result<PascalString, TooLong/DecodeError>`)* | ❌ |
| `from_utf16_lossy` | 🚫 | — | *(could be `from_utf16_lossy_truncated` / `try_from_utf16_lossy`)* | ❌ |
| `from_utf16be` | 🚫 | — | — | ❌ |
| `from_utf16be_lossy` | 🚫 | — | — | ❌ |
| `from_utf16le` | 🚫 | — | — | ❌ |
| `from_utf16le_lossy` | 🚫 | — | — | ❌ |
| `from_utf8` | 🚫 | — | `TryFrom<&[u8]> for PascalString` (validates UTF‑8 + capacity) | 🟡 |
| `from_utf8_lossy` | 🚫 | — | `from_str_truncated` + caller-side `String::from_utf8_lossy` | ❌ |
| `from_utf8_lossy_owned` | 🚫 | — | — | ❌ |
| `from_utf8_unchecked` | 🚫 | — | *(not applicable without owned heap buffer; `PascalString` maintains UTF‑8 invariant by construction)* | ❌ |
| `insert` | 🚫 | — | `try_insert`, `insert_expect_capacity` | 🟡 |
| `insert_str` | 🚫 | — | `try_insert_str`, `insert_str_expect_capacity` | 🟡 |
| `into_boxed_str` | 🚫 | — | `to_string().into_boxed_str()` | ❌ |
| `into_bytes` | 🚫 | — | `as_bytes().to_vec()`; `into_inner()` (bytes+len) | 🟡 |
| `into_chars` | 🚫 | — | *(iterate chars via `chars()` on `&str`)* | ❌ |
| `into_raw_parts` | 🚫 | — | `into_inner` (len + inline array) | 🟡 |
| `is_empty` | ✅ | `is_empty` | — | ✅ |
| `leak` | 🚫 | — | — | ❌ |
| `len` | ✅ | `len` | — | ✅ |
| `new` | ✅ | `new` | — | ✅ |
| `pop` | ✅ | `pop` | — | ✅ |
| `push` | 🚫 | — | `try_push`, `push_expect_capacity`, `push_str_truncated` (for strings) | 🟡 |
| `push_str` | 🚫 | — | `try_push_str`, `push_str_truncated`, `push_str_expect_capacity` | 🟡 |
| `remove` | ✅ | `remove` *(panicking on invalid idx/boundary like `String`)* | `try_remove` (non-panicking) | 🟡 |
| `remove_matches` | 🚫 | — | `remove_matches(&str)` *(in-place scan+compact; `retain` covers char-level filtering)* | 🟡 |
| `replace_first` | 🚫 | — | *(could be implemented on `&str` output; may need truncation APIs)* | ❌ |
| `replace_last` | 🚫 | — | — | ❌ |
| `replace_range` | 🚫 | — | `try_replace_range_bounds`, `try_replace_range_bounds_truncated`, `replace_range_bounds_expect_capacity` | 🟡 |
| `reserve` | 🚫 | — | *(no-op if within capacity; otherwise cannot)* | ❌ |
| `reserve_exact` | 🚫 | — | — | ❌ |
| `retain` | ✅ | `retain` | — | ✅ |
| `shrink_to` | 🚫 | — | *(no-op for fixed-capacity)* | ❌ |
| `shrink_to_fit` | 🚫 | — | *(no-op for fixed-capacity)* | ❌ |
| `split_off` | 🚫 | — | `split_off(at)` panicking + `try_split_off(at)` non-panicking | 🟡 |
| `truncate` | ✅ | `truncate` *(requires char boundary, panics like `String` on invalid boundary)* | — | ✅ |
| `try_reserve` | 🚫 | — | *(could return Ok if fits, Err otherwise; not present)* | ❌ |
| `try_reserve_exact` | 🚫 | — | — | ❌ |
| `try_with_capacity` | 🚫 | — | `new` (capacity is const), `try_from_str_const` (const ctx) | ❌ |
| `with_capacity` | 🚫 | — | `new` (capacity is const) | ❌ |


