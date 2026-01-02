![](https://img.shields.io/crates/l/smart-string.svg)
[![crates.io](https://img.shields.io/crates/v/smart-string.svg)](https://crates.io/crates/smart-string)

[//]: # ([![Build Status]&#40;https://travis-ci.org/irbis-labs/smart-string.svg&#41;]&#40;https://travis-ci.org/irbis-labs/smart-string&#41;)

[//]: # ([![Coverage Status]&#40;https://coveralls.io/repos/github/irbis-labs/smart-string/badge.svg?branch=main&#41;]&#40;https://coveralls.io/github/irbis-labs/smart-string?branch=main&#41;)
![Minimal rust version 1.56](https://img.shields.io/badge/rustc-1.56+-green.svg) (sorry, not checked yet)

# Smart String Library

This library is a collection of string types and traits designed for enhanced string manipulation. It's born out of a
need to centralize and avoid code repetition, particularly unsafe operations, from the author's previous projects. While
the tools and methods here reflect certain patterns frequently observed in those projects, it's worth noting that the
library itself is in its early stages of development.

## Status

Currently, Smart String is in active development, and its API might undergo changes. Although it encapsulates
tried-and-true patterns from earlier works, the library as a standalone entity is relatively new. Hence, it's advised to
use it with caution and feel free to provide feedback, report issues, or suggest improvements.

Some core behavior is covered by unit tests, but coverage is incomplete.

## Features

- [x] `serde` - Enables serde support.

## What's in the box

- [`PascalString<N>`](https://github.com/irbis-labs/smart-string/tree/main/src/pascal_string): A string with a fixed
  capacity, either stored on the stack or in-place within larger structures and arrays.
- [`DisplayExt`](https://github.com/irbis-labs/smart-string/tree/main/src/display_ext): A suite of methods to
  streamline string formatting.
- [`SmartString`](https://github.com/irbis-labs/smart-string/tree/main/src/smart_string): A string that dynamically
  decides its storage location (stack or heap) based on its length.

## Roadmap

### Primary Goals

- `StringsStack`: A dedicated storage solution for multiple strings, allowing them to be housed within a single
  allocation.
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
