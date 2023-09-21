# `PascalString<N>`

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
