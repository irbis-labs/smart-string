# TODOS

## Pattern<'_> trait support for pattern-based methods

**What:** Research and potentially implement a Pattern-like trait or accept broader pattern types for `remove_matches`, `replace_first`, `replace_last`.

**Why:** Current `&str` + `char` subset covers ~95% of usage, but full `Pattern` support would allow closures, `&[char]`, regex-like patterns, matching String's nightly API.

**Pros:** True API parity for pattern-based methods. Forward-compatible with eventual Pattern stabilization.

**Cons:** Pattern has been unstable for years. Implementing an equivalent is complex. May need to track upstream changes.

**Context:** Raised by Codex during plan review (2026-04-02). Current plan ships `&str` + `char`, which is pragmatic but incomplete.

**Depends on:** Safe API Parity release (the `&str` + `char` implementations must exist first).

---

## Unsafe API parity via feature flag

**What:** Design and implement unsafe/representation-exposing String methods (`as_mut_vec`, `from_utf8_unchecked`, `from_raw_parts`, `into_raw_parts`) behind a feature flag like `fragile` or `unsafe_parity`.

**Why:** These 4 methods are the remaining gap between "safe parity" and true full parity. Some users (FFI, zero-copy serialization) genuinely need them.

**Pros:** Closes the last parity gap. Enables advanced use cases.

**Cons:** Requires careful design since SmartString has dual representation (stack/heap). Weakens UTF-8 invariants. Needs Miri coverage.

**Context:** Identified during eng review (2026-04-02). Codex pointed out that "Full Parity" is misleading without these. Deserves its own architectural analysis session.

**Depends on:** Safe API Parity release + Miri CI (for testing unsafe code).

---

## Benchmark safe vs unsafe paths for extend_from_within

**What:** Create a micro-benchmark comparing safe (extract `&str` + `push_str`) vs unsafe (`ptr::copy`) for `extend_from_within` on PascalString, across various buffer sizes and range lengths.

**Why:** Safe-first was chosen, but compiler optimization behavior is unknown. Only a benchmark resolves whether `ptr::copy` adds measurable value.

**Pros:** Data-driven decision. Avoids premature optimization AND premature de-optimization.

**Cons:** Micro-benchmarks can be misleading without real workload context.

**Context:** Codex challenged `ptr::copy` as premature optimization (2026-04-02). Owner noted compiler might optimize either path. Deferred to post-implementation benchmark.

**Depends on:** Safe `extend_from_within` implementation exists.

---

## Evaluate PascalString::try_reserve

**What:** Evaluate whether `PascalString::try_reserve` (capacity check, not allocation) is useful enough to implement or should be skipped permanently.

**Why:** Codex called it "fake parity" on a fixed-capacity type. `len() + capacity()` already covers the use case. But some users might want a standardized "will this fit?" predicate.

**Pros:** If useful, 5-line implementation. If not, saved unnecessary API surface.

**Cons:** Deferring means re-evaluating later.

**Context:** Parity table marks `try_reserve` as 🚫 for PascalString. Raised during eng review (2026-04-02).

**Depends on:** Nothing specific.

---

## Research: cargo audit value for library crates

**What:** Investigate whether `cargo audit` in CI provides meaningful value for library crates with minimal dependency surfaces (2 runtime deps).

**Why:** Currently skipped — `serde` and `rustversion` vulnerabilities would be global news, not caught by our CI first. But the question deserves a proper answer: at what dependency count / risk profile does it become worth the CI time?

**Context:** Raised during v0.3.0 pre-release audit (2026-04-04). Deferred as low priority.
