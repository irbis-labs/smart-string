# `smart-string` std compatibility / API parity checklist

Goal: incrementally improve “drop-in replacement” ergonomics for `SmartString` (and, where reasonable, `PascalString`)
toward `std::String` / `str`.

This is intentionally **step-by-step**: small, reviewable additions with tests, while keeping the crate’s core invariants
sound and its performance intent intact.

## Compatibility principles

- **`SmartString`**: should be able to behave like `String` for most APIs, by operating on the stack variant when it fits
  and **promoting to heap** when needed.
- **`PascalString`** (fixed capacity): cannot be fully compatible with `String` for “infallible growth” APIs.
  - We prefer to provide both:
    - **fallible** APIs (`try_push_str`, `try_push`) and
    - **infallible ergonomics** that may **panic on overflow** (`push_str`, `push`), mirroring how `String` may panic on
      OOM in practice.
- **Unsafe policy**: every `unsafe {}` must have a local `// SAFETY:` comment describing the invariant that makes it
  sound; tests should cover UTF‑8 boundaries and capacity edges.

## Parity checklist (high-level)

### Traits (priority: high)

- **SmartString**
  - [x] `Deref<Target=str>`, `DerefMut`
  - [x] `AsRef<str>`, `AsRef<[u8]>`, `Borrow<str>`
  - [x] `AsMut<str>`, `BorrowMut<str>`
  - [x] `From<&str>`, `From<String>`, `From<Cow<str>>`
  - [x] `From<&String>`, `From<Box<str>>`, `From<Rc<str>>`, `From<Arc<str>>`
  - [x] `FromStr` (infallible)
  - [x] `FromIterator<char>`, `FromIterator<&str>`
  - [x] `Extend<char>`, `Extend<&str>`, `Extend<String>`
  - [x] `fmt::Write`
  - [x] `Add`, `AddAssign`
  - [x] `Extend<&char>`, `Extend<&String>`
  - [ ] `IntoIterator` (over chars/bytes?) — decide ergonomics vs `Deref<str>` sufficiency

- **PascalString**
  - [x] `Deref<Target=str>`, `DerefMut`
  - [x] `AsRef<str>`, `AsRef<[u8]>`, `Borrow<str>`
  - [x] `AsMut<str>`, `BorrowMut<str>`
  - [x] `TryFrom<&str>`, `TryFrom<&[u8]>`, `TryFrom<char>`
  - [x] `FromStr` (fallible: `TooLong`)
  - [ ] `From<&str>` (would need to panic/truncate; decide explicitly)
  - [ ] `Extend` impls (would need panic-on-overflow semantics; decide explicitly)

### `String`-like inherent APIs (priority: high for SmartString)

- **SmartString**
  - [x] `new`, `with_capacity`, `capacity`
  - [x] `push`, `push_str`, `pop`, `truncate`, `clear`
  - [x] `reserve`, `reserve_exact`, `try_reserve*`, `shrink_to_fit`, `shrink_to`
  - [x] `len`, `is_empty` (explicit wrappers for std parity + rustdoc discoverability)
  - [x] `insert`, `insert_str` (currently promotes to heap and delegates)
  - [x] `remove`, `retain`, `drain`, `replace_range` (currently promotes to heap and delegates)
  - [x] `split_off` (promotes to heap and delegates; returned value may be stored on stack if it fits)
  - [x] `into_bytes`, `into_string` (consuming conversions)
  - [x] `into_boxed_str`, `leak`, `from_utf8_lossy`
  - [ ] `as_mut_vec` (likely **out of scope**; would expose raw bytes and complicate UTF‑8 invariants)

- **PascalString**
  - [x] `len`, `is_empty`, `capacity`
  - [x] `try_push*`, `push_str_truncated`, `truncate`, `pop`, `clear`
  - [x] `push_str`, `push` (panic on overflow)

## Next slice (suggested)

1) Add the most common missing `String`-like mutation methods to `SmartString` by:
   - operating on stack when feasible, otherwise promoting to heap and delegating to `String`.
2) Expand boundary tests:
   - UTF‑8 insertion/removal boundaries
   - capacity edges (`N-1`, `N`, `N+1`) across promote paths


