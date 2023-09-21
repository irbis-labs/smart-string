# `DisplayExt`

This trait offers a suite of methods for streamlined string formatting. While every type supporting `Display` also
implements this trait, it's unlikely you'll need to implement it for your custom types.

- `is_empty` - Checks if the output from `.to_string()` will be empty, without making allocations or copies. Be aware of
  potential computational costs if the `Display` implementation performs calculations before its first output. It's
  unusual for `Display` implementations to be computationally heavy, as the convention is to keep them lightweight.

- `write_to_fmt<W: fmt::Write>`, `write_to_bytes<W: io::Write>` - Writes output directly to the provided formatter,
  removing the need for extraneous code.

- `to_fmt<T>`, `to_bytes<T>`, where `T: Write + Default` - Constructs a fresh instance of the specified type and writes
  the result to it.

- `try_to_*` - Functions akin to the ones above but return a `Result` instead of panicking.

- `format_with(cb)` - A callback for easy processing of the output from the underlying formatter. Useful for when you
  want to modify the formatter's output before it's written.

These methods prove invaluable when formatting a string into a buffer or writer, especially if you're looking to
minimize repetitive boilerplate.

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

A common task with `Display` combinators is to process the output of the underlying formatter. For instance, you might
want to unescape a string. Ultimately, you'd need to flush the temporary buffer that might have contained a potential
escaped sequence, which didn't turn out to be an actual one upon reaching the text's end.

```rust
struct UnescapeFormatter<T: fmt::Display>(pub T);

impl<T: fmt::Display> fmt::Display for UnescapeFormatter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Buffer to process character sequences at the edges of &str 
        // from the underlying formatter.
        let mut buf = PascalString::<5>::new();
        self.0.format_with(&mut buf, |optional_str| {
            if let Some(s) = optional_str {
                // Process the stream of characters in s, including the handling of any unfinished escape sequences.
                // If s ends with an incomplete escaped sequence, store it in the buffer and delay writing it.
            } else {
                // We've reached the end of the stream; time to flush the buffer.
                f.write_str(&buf)?;
            }
            Ok(())
        })
    }
}
```
