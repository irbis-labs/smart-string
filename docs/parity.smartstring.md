# `SmartString` ↔ `std::string::String` parity table

Legend:

- **Match**:
  - ✅ = full correspondence is possible (same semantics are achievable for a stack-or-heap string)
  - 🚫 = full correspondence is not possible (or would require exposing representation/unsafe contracts we intentionally avoid)
- **Implemented**:
  - ✅ = exists
  - 🟡 = partial (some equivalent exists, but semantics/API differ)
  - ❌ = not implemented

Notes:

- `SmartString` can promote from stack to heap, so most `String` APIs can have full semantic parity.
- Some APIs are **unsafe / representation-exposing** (`*_raw_parts`, `as_mut_vec`, `from_utf8_unchecked`). They are
  technically possible (by forcing heap) but are treated as “out of scope” for now; see the “Additional/alternative”
  column.

| `String` method | Match | SmartString method (full match) | Additional / alternative methods | Implemented |
|---|---:|---|---|---:|
| `as_bytes` | ✅ | `as_bytes` | `as_ref::<[u8]>()` | ✅ |
| `as_mut_str` | ✅ | `as_mut_str` | *(also `DerefMut<Target=str>`)* | ✅ |
| `as_mut_vec` | ✅ | — | *(possible by forcing heap: `s.into_heap()` then `String::as_mut_vec` (unsafe))* | ❌ |
| `as_str` | ✅ | `as_str` | *(also `Deref<Target=str>`)* | ✅ |
| `capacity` | ✅ | `capacity` | — | ✅ |
| `clear` | ✅ | `clear` | — | ✅ |
| `drain` | ✅ | `drain` | *(currently promotes to heap and delegates)* | ✅ |
| `extend_from_within` | ✅ | — | *(could be implemented by delegating on heap, or by stack-aware copy with promotion on overflow)* | ❌ |
| `from_raw_parts` | 🚫 | — | *(representation-exposing/unsafe; could exist only for heap variant)* | ❌ |
| `from_utf16` | ✅ | `from_utf16` | — | ✅ |
| `from_utf16_lossy` | ✅ | `from_utf16_lossy` | — | ✅ |
| `from_utf16be` | ✅ | — | *(can be implemented by delegating to `String::from_utf16be`)* | ❌ |
| `from_utf16be_lossy` | ✅ | — | *(can be implemented by delegating to `String::from_utf16be_lossy`)* | ❌ |
| `from_utf16le` | ✅ | — | *(can be implemented by delegating to `String::from_utf16le`)* | ❌ |
| `from_utf16le_lossy` | ✅ | — | *(can be implemented by delegating to `String::from_utf16le_lossy`)* | ❌ |
| `from_utf8` | ✅ | `from_utf8` | — | ✅ |
| `from_utf8_lossy` | ✅ | `from_utf8_lossy` | — | ✅ |
| `from_utf8_lossy_owned` | ✅ | `from_utf8_lossy_owned` | — | ✅ |
| `from_utf8_unchecked` | ✅ | — | *(possible by forcing heap and delegating; unsafe API)* | ❌ |
| `insert` | ✅ | `insert` | `insert_str_truncated` (SmartString-only helper for stack paths) | ✅ |
| `insert_str` | ✅ | `insert_str` | `insert_str_truncated`, `try_insert_str_truncated` | ✅ |
| `into_boxed_str` | ✅ | `into_boxed_str` | — | ✅ |
| `into_bytes` | ✅ | `into_bytes` | `From<SmartString> for Vec<u8>` | ✅ |
| `into_chars` | ✅ | — | *(can be implemented by delegating to `String::into_chars` on heap)* | ❌ |
| `into_raw_parts` | 🚫 | — | *(representation-exposing/unsafe; could exist only for heap variant)* | ❌ |
| `is_empty` | ✅ | `is_empty` | — | ✅ |
| `leak` | ✅ | `leak` | — | ✅ |
| `len` | ✅ | `len` | *(also `Deref<Target=str>`)* | ✅ |
| `new` | ✅ | `new` | — | ✅ |
| `pop` | ✅ | `pop` | — | ✅ |
| `push` | ✅ | `push` | — | ✅ |
| `push_str` | ✅ | `push_str` | — | ✅ |
| `remove` | ✅ | `remove` | — | ✅ |
| `remove_matches` | ✅ | — | *(can be implemented by delegating to `String::remove_matches` on heap; stack path possible too)* | ❌ |
| `replace_first` | ✅ | — | *(can be implemented by delegating to `String::replace_first` on heap)* | ❌ |
| `replace_last` | ✅ | — | *(can be implemented by delegating to `String::replace_last` on heap)* | ❌ |
| `replace_range` | ✅ | `replace_range` | — | ✅ |
| `reserve` | ✅ | `reserve` | — | ✅ |
| `reserve_exact` | ✅ | `reserve_exact` | — | ✅ |
| `retain` | ✅ | `retain` | — | ✅ |
| `shrink_to` | ✅ | `shrink_to` | — | ✅ |
| `shrink_to_fit` | ✅ | `shrink_to_fit` | — | ✅ |
| `split_off` | ✅ | `split_off` | *(optimized to avoid allocation when tail fits stack)* | ✅ |
| `truncate` | ✅ | `truncate` | — | ✅ |
| `try_reserve` | ✅ | `try_reserve` | — | ✅ |
| `try_reserve_exact` | ✅ | `try_reserve_exact` | — | ✅ |
| `try_with_capacity` | ✅ | `try_with_capacity` | — | ✅ |
| `with_capacity` | ✅ | `with_capacity` | — | ✅ |


