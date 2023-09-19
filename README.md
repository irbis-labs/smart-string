![](https://img.shields.io/crates/l/smart-string.svg)
[![crates.io](https://img.shields.io/crates/v/smart-string.svg)](https://crates.io/crates/smart-string)

[//]: # ([![Build Status]&#40;https://travis-ci.org/irbis-labs/smart-string.svg&#41;]&#40;https://travis-ci.org/irbis-labs/smart-string&#41;)

[//]: # ([![Coverage Status]&#40;https://coveralls.io/repos/github/irbis-labs/smart-string/badge.svg?branch=main&#41;]&#40;https://coveralls.io/github/irbis-labs/smart-string?branch=main&#41;)
![Minimal rust version 1.56](https://img.shields.io/badge/rustc-1.56+-green.svg) (sorry, not checked yet)

# Smart String

This library is a collection of string types and traits designed for enhanced string manipulation. It's born out of a
need to centralize and avoid code repetition, particularly unsafe operations, from the author's previous projects. While
the tools and methods here reflect certain patterns frequently observed in those projects, it's worth noting that the
library itself is in its early stages of development.

## Status

Currently, Smart String is in active development, and its API might undergo changes. Although it encapsulates
tried-and-true patterns from earlier works, the library as a standalone entity is relatively new. Hence, it's advised to
use it with caution and feel free to provide feedback, report issues, or suggest improvements.

## What's in the box

### `PascalString<N>`

This is a string of fixed capacity, stored on the stack or in-place within larger structures and arrays.

The `PascalString<N>` is pretty straightforward: it's a wrapper around a [u8; N] array with an extra byte
right up front for the string length. So, in total, it takes up `N + 1` bytes of memory.

The string is aligned to a 1-byte boundary, so it can fill the gaps between fields or be stored compactly
in arrays. However, it's advisable not to cross cache line boundaries without a valid reason.

PascalString is ideal for short strings where heap allocation is undesired, and the string's length
is predetermined. It's suitable for words from a constrained dictionary, keys in hash maps,
or as a temporary buffer for string manipulations, among others.

```rust
use smart_string::PascalString;

fn main() {
    let mut s = PascalString::<31>::try_from("Hello").unwrap();
    s.try_push_str(", world!").unwrap();
    assert_eq!(s, "Hello, world!");
    assert_eq!(s.len(), 13);
    assert_eq!(s.capacity(), 31);

    // You can use it as a buffer for string manipulations.
    let mut buf: PascalString<255> = Default::default();
    let mut remaining = "..... <Big large string> .....";
    while !remaining.is_empty() {
        buf.clear();
        // Take a chunk of the remaining string, respecting utf-8 boundaries.
        remaining = buf.push_str_truncated(remaining);
        // Do something with the buf.
        // ...
    }
}
```

### `DisplayExt`

This trait offers a suite of methods for streamlined string formatting. While it's implemented for all types
that support `Display`, you can use it seamlessly with any displayable type. However, you're unlikely
to implement it for your custom types.

- `is_empty` - Determines if the output of `.to_string()` will be empty. This is achieved without any allocations
  or copying. However, there might be computational costs if the `Display` implementation performs calculations
  before its first output. It's worth noting that such behavior is atypical, as there's a general preference
  to keep the `Display` implementations as lightweight as possible.

- `write_to_fmt<W: fmt::Write>`, `write_to_bytes<W: io::Write>` - Directly writes the output to the given formatter,
  eliminating the need for boilerplate code.

- `to_fmt<T>`, `to_bytes<T>`, where `T: Write + Default` - Constructs a fresh instance of the specified type and then
  writes the result into it.

- `try_to_*` - Similar functions, but they return a `Result` rather than panicking.

These methods are particularly handy when you have to format a string into a buffer or writer and wish to bypass
repetitive boilerplate.

```rust
use smart_string::DisplayExt;

fn main() {
    let mut s: PascalString<15> = "Hello,".to_fmt();
    " world!".write_to_fmt(&mut s).unwrap();
    assert_eq!(s, "Hello, world!");
}
```

## Roadmap

### Primary Goals

- [x] `PascalString<N>`: A string with a fixed capacity, either stored on the stack or in-place within larger
  structures and arrays.

- [x] `DisplayExt`: A suite of methods to streamline string formatting.

- [ ] `SmartString`: A string that dynamically decides its storage location (stack or heap) based on its length.

- [ ] `StringsStack`: A dedicated storage solution for multiple strings, allowing them to be housed within a single
  allocation.

- [ ] `StringsSet`: A storage medium designed for strings, facilitating both consolidated allocation and utilization
  as a hash set.

### Additional Goals

- [ ] `PascalStringLong<N>`: An enhanced variant of `PascalString<N>` offering support for capacities up to 2^32-1
  bytes, catering to scenarios where a 255-byte limit falls short.

- [ ] Compatibility with `no_std` environments.

- [ ] Integration support for [ufmt](https://crates.io/crates/ufmt).

- [ ] Open to more suggestions!

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
  at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
