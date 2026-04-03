# Changelog

All notable changes to this project will be documented in this file.

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
