# Contributing

Thanks for your interest in contributing to `smart-string`!

This crate contains low-level, performance-oriented string primitives and includes some `unsafe` code, so contributions
are expected to come with boundary-focused tests and clear invariants.

## Development workflow

From the crate root:

```bash
cargo test
```

## Quality gates

This repository uses a `rustfmt.toml` that enables unstable options, so formatting checks should use nightly:

```bash
cargo +nightly fmt --all -- --check
cargo check --all-targets
cargo test
cargo +stable clippy --all-targets -- -D warnings
```

## MSRV check

This crate declares MSRV in `Cargo.toml` (`rust-version`). To verify it locally:

```bash
rustup toolchain install 1.59.0 --profile minimal
cargo +1.59.0 test --locked
```

## What to test

When changing behavior around storage, UTF-8 boundaries, or unsafe code paths, please include tests for:

- empty inputs
- maximum capacity boundaries (`CAPACITY-1`, `CAPACITY`, `CAPACITY+1`)
- UTF-8 boundary conditions (multi-byte characters; truncation should never split a codepoint)
- serde roundtrips (when the `serde` feature is enabled)


