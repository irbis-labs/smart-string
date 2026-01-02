# Contributing

Thanks for your interest in contributing to `smart-string`!

This crate contains low-level, performance-oriented string primitives and includes some `unsafe` code, so contributions
are expected to come with boundary-focused tests and clear invariants.

## Development workflow

From the crate root:

```bash
cargo +stable test
```

## Quality gates

This repository uses a `rustfmt.toml` that enables unstable options, so formatting checks should use nightly:

```bash
cargo +nightly fmt --all -- --check
cargo +stable check --all-targets
cargo +stable test
cargo +stable clippy --all-targets -- -D warnings
```

## MSRV check

This crate declares MSRV in `Cargo.toml` (`rust-version`). To verify it locally:

```bash
rustup toolchain install 1.59.0 --profile minimal
cargo +1.59.0 test
```

## Required local test matrix

Before opening a PR, validate on both:

- **Latest stable**: `cargo +stable test`
- **MSRV**: `cargo +1.59.0 test`

Note: `Cargo.lock` is intentionally not committed for this library crate. If you want a more CI-like MSRV check
(newer Cargo resolver + older compiler), you can run:

```bash
rustup toolchain install 1.59.0 --profile minimal
RUSTC="$(rustc +1.59.0 --print sysroot)/bin/rustc" \
RUSTDOC="$(rustc +1.59.0 --print sysroot)/bin/rustdoc" \
cargo +stable test
```

## What to test

When changing behavior around storage, UTF-8 boundaries, or unsafe code paths, please include tests for:

- empty inputs
- maximum capacity boundaries (`CAPACITY-1`, `CAPACITY`, `CAPACITY+1`)
- UTF-8 boundary conditions (multi-byte characters; truncation should never split a codepoint)
- serde roundtrips (when the `serde` feature is enabled)


