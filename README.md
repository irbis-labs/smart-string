![](https://img.shields.io/crates/l/smart-string.svg)
[![crates.io](https://img.shields.io/crates/v/smart-string.svg)](https://crates.io/crates/smart-string)

[//]: # ([![Build Status]&#40;https://travis-ci.org/irbis-labs/smart-string.svg&#41;]&#40;https://travis-ci.org/irbis-labs/smart-string&#41;)

[//]: # ([![Coverage Status]&#40;https://coveralls.io/repos/github/irbis-labs/smart-string/badge.svg?branch=main&#41;]&#40;https://coveralls.io/github/irbis-labs/smart-string?branch=main&#41;)
![MSRV 1.59](https://img.shields.io/badge/rustc-1.59+-green.svg)

# Smart String Library

This library is a collection of string types and traits designed for enhanced string manipulation. It's born out of a
need to centralize and avoid code repetition, particularly unsafe operations, from the author's previous projects. While
the tools and methods here reflect certain patterns frequently observed in those projects, it's worth noting that the
library itself is in its early stages of development.

## Status

Currently, Smart String is in active development, and its API might undergo changes. Although it encapsulates
tried-and-true patterns from earlier works, the library as a standalone entity is relatively new. Hence, it's advised to
use it with caution and feel free to provide feedback, report issues, or suggest improvements.

The library has 247 tests covering core behavior, edge cases, and UTF-8/UTF-16 boundary conditions, including property-based tests (`proptest`) for UTF-16 decode and Miri CI for undefined behavior detection.

## Features

- [x] `serde` - Enables serde support.

## MSRV (Minimum Supported Rust Version)

MSRV is **Rust 1.59.0** (with default features).

Motivation:

- This crate uses the 2021 edition and relies on language features needed by the public API (notably a **default const
  generic parameter** in `SmartString<const N: usize = DEFAULT_CAPACITY>`).
- Keeping MSRV relatively low matters for library users; if we ever need to bump it, we should document it in release
  notes.

What this guarantees (and what it doesn't):

- **Guaranteed**: `smart-string` itself builds and tests with `rustc 1.59.0`.
- **Not guaranteed**: that `cargo +1.59.0` will always be able to resolve and build the *latest* dependency graph from
  crates.io in the future without pinning, because transitive dependencies may raise their own MSRV over time.

In CI we run an MSRV job that compiles with `rustc 1.59.0`. If MSRV breaks due to dependency drift, we either:

- pin direct dependency versions / add upper bounds (policy decision), or
- bump MSRV (and document it).

## What's in the box

- [`PascalString<N>`](https://github.com/irbis-labs/smart-string/tree/main/src/pascal_string): A string with a fixed
  capacity, either stored on the stack or in-place within larger structures and arrays.
- [`DisplayExt`](https://github.com/irbis-labs/smart-string/tree/main/src/display_ext): A suite of methods to
  streamline string formatting.
- [`SmartString`](https://github.com/irbis-labs/smart-string/tree/main/src/smart_string): A string that dynamically
  decides its storage location (stack or heap) based on its length.

## Roadmap

### Shipped

- ~~`StringsStack`~~: Shipped in v0.3.0 as `StrStack` (mutable builder) + `StrList` (frozen) + `StrListRef` (borrowed view). Includes checkpoint/rollback for speculative parsing.

### Primary Goals

- `StringsSet`: A storage medium designed for strings, facilitating both consolidated allocation and utilization
  as a hash set.

### Additional Goals

- `PascalStringLong<N>`: An enhanced variant of `PascalString<N>` offering support for capacities up to 2^32-1
  bytes, catering to scenarios where a 255-byte limit falls short.
- Compatibility with `no_std` environments.
- Integration support for [ufmt](https://crates.io/crates/ufmt).

Open to more suggestions!

## SmartString storage semantics (explicit conversions)

`SmartString` may **promote** from stack to heap during mutating operations (e.g. `push_str`, `reserve`) when the stack
capacity is exceeded.

It does **not** automatically demote from heap to stack when the content becomes shorter (including during
in-place deserialization). This is intentional: implicit demotion can cause surprising realloc/dealloc churn in
real workloads (e.g. shorten → re-grow).

If you want to attempt a demotion, call `try_into_stack`. If you want to force heap storage, call `into_heap`.

## Safety & invariants (unsafe code)

This crate uses `unsafe` in a few carefully-scoped places to avoid repeated UTF‑8 validation and bounds checks when
projecting internal byte buffers as `&str` / `&mut str`.

The key invariants are:

- **`PascalString`**: `len <= CAPACITY` and `data[..len]` is always valid UTF‑8.
- **`StrStack`**: `data` is always valid UTF‑8 and `ends` entries are valid segment boundaries within `data`.

Policy:

- `#![deny(unsafe_op_in_unsafe_fn)]` enforced at the crate root.
- Every `unsafe { ... }` block must have a local `// SAFETY:` comment explaining what invariant makes it sound.
- Tests must cover UTF‑8 boundary and capacity edge cases.
- CI runs [Miri](https://github.com/rust-lang/miri) on every push to detect undefined behavior.

## Compatibility with `std::String`

`SmartString` aims to be a pragmatic, mostly drop-in alternative to `String`:

- It supports common `String`-like APIs and traits.
- Some mutation APIs currently **promote to heap and delegate** (trading stack retention for simpler, correct semantics).

For a living checklist of what’s implemented vs planned, see `docs/parity.api.md`.

Note on `PascalString`: it is fixed-capacity; for `String`-like “infallible” ergonomics it provides `push` / `push_str`
which **panic on overflow** (use `try_push` / `try_push_str` for fallible behavior).

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
  at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.

## Development

Recommended (enable repo hooks once per clone):

```bash
git config core.hooksPath .githooks
```

Quality gates:

```bash
cargo +nightly fmt --all -- --check
cargo check --all-targets
cargo test
cargo +stable clippy --all-targets -- -D warnings
```

See also: `CONTRIBUTING.md`.
