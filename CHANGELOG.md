# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2026-04-03

### Added

- **StrStack ergonomics**: `with_capacity`, `bytes_len`, `reserve_items`, `reserve_bytes`, `last`, `truncate`, `try_push`, `clear` (was private, now public).
- **Checkpoint/rollback**: `Checkpoint` type, `StrStack::checkpoint()` and `StrStack::reset()` for speculative parsing with cheap rollback.
- **StrList**: frozen (immutable) string list backed by `Box<[u8]>` + `Box<[u32]>`. Constructed via `From<StrStack>` or `FromIterator<&str>`. Serde support (serialize/deserialize as sequence of strings).
- **StrListRef**: borrowed read-only view over `&[u8]` + `&[u32]` with validated construction (`StrListRef::new`). Zero-copy views over external buffers (e.g., memory-mapped files).
- **StrListValidationError**: error type for `StrListRef::new` with variants for boundary and UTF-8 violations.
- **StrStackOverflow**: error type for `try_push` when total byte length would exceed `u32::MAX`.
- **StrListIter**: iterator for `StrList`/`StrListRef` with `DoubleEndedIterator`, `ExactSizeIterator`, `FusedIterator`.
- `DoubleEndedIterator` and `FusedIterator` for `StrStackIter`.
- Module-level rustdoc for `str_stack`: representation model, invariants, complexity table, usage example.

### Changed

- **Breaking**: `StrStack` boundary table migrated from `Vec<usize>` to `Vec<u32>`. Halves boundary memory on 64-bit platforms. Total content limited to ~4 GB.
- **Breaking**: `StrStack::get_bounds()` now returns `Option<(u32, u32)>` instead of `Option<(usize, usize)>`.
- `StrStackIter` refactored from cursor-based to index-based to support `DoubleEndedIterator`.

## [0.2.3] - 2026-04-03

### Added

- 37 new tests closing coverage gaps: `SmartString::drain`, `into_bytes`, `into_string`, `from_utf8`, `shrink_to`, `shrink_to_fit`, `StrStack` mutation sequences (push/remove/push), empty string segments, `DisplayExt::format_with`, `write_to_bytes`.
- Property-based tests (`proptest`) for UTF-16 decode: round-trip, reference match against `String::from_utf16`, lossy-never-panics, stack-awareness.
- `proptest` added as dev-dependency.

## [0.2.2] - 2026-04-03

### Added

- Miri CI job for automated UB detection (nightly, strict-provenance).
- `#![deny(unsafe_op_in_unsafe_fn)]` at crate root.
- `debug_assert!` guard in UTF-16 BMP decode branch.

### Changed

- `SmartString::leak()` now gated behind `rustversion::since(1.72)` — available automatically on Rust 1.72+, absent on older compilers. Preserves MSRV 1.59 without feature flags.
- `StrStack::get_unchecked` deprecated in favor of crate-internal `get_unchecked_internal`. Reduces public unsafe surface.
- `PascalString::retain` and `remove_matches` now share a `snapshot_as_str!` macro, centralizing the unsafe `from_utf8_unchecked` call.

### Fixed

- `extend_from_within`: potential `usize` overflow on `Bound::Excluded(usize::MAX)` / `Bound::Included(usize::MAX)` — now uses `checked_add`.
- MSRV CI: use `cargo +1.59.0 check --lib` to avoid `split-debuginfo` incompatibility and dev-dependency MSRV drift (`ryu` 1.0.23 requires 1.71).
- Pre-commit hook now checks formatting and clippy before allowing commits.

## [0.2.0] - 2026-04-03

### Added

- `SmartString::extend_from_within`: copy a byte range and append it to the end.
- `SmartString::remove_matches` / `remove_matches_char`: remove all occurrences of a pattern in-place.
- `SmartString::replace_first` / `replace_first_char`: replace the first occurrence of a pattern.
- `SmartString::replace_last` / `replace_last_char`: replace the last occurrence of a pattern.
- `SmartString::from_utf16be` / `from_utf16be_lossy`: decode UTF-16 big-endian from raw bytes.
- `SmartString::from_utf16le` / `from_utf16le_lossy`: decode UTF-16 little-endian from raw bytes.
- `SmartString::into_chars`: consuming char iterator with full trait surface (Iterator, DoubleEndedIterator, FusedIterator, ExactSizeIterator, Clone, Debug, Display, `as_str()`, `into_string()`).
- `IntoChars<N>` public iterator type.
- `Utf16DecodeError` crate-local error type for UTF-16 decode failures.
- `PascalString::split_off` / `try_split_off`: split a PascalString at a byte index.
- `PascalString::remove_matches`: remove all occurrences of a pattern in-place.
- `SplitOffError` error type.
- `TODOS.md` tracking deferred work from the eng review.

### Changed

- `from_utf16` / `from_utf16_lossy` now pick the stack variant when the decoded string fits inline capacity (previously always forced heap allocation).

### Fixed

- Stale `drain` entry in `docs/parity.api.md` (was marked unimplemented, but it exists).
- Removed dead commented-out `from_utf8_lossy` TBD code.
