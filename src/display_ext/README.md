# `DisplayExt`

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

fn join(prefix: &str, stem: &str, suffix: &str) -> SmartString {
    format_args!("{}{}{}", prefix, stem, suffix).to_fmt()
}
```
