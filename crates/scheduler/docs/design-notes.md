# Scheduler Design Notes: Coroutine Strategy

Tiffany's coroutine scheduler is based on the pyos8 model — cooperative, deterministic, and traceable.

This document explores candidate coroutine engines and outlines the strategy for evolving coroutine support within the scheduler crate.

---

## Candidate Engines

### 1. May Coroutine Crate

May is a stackful coroutine runtime for Rust, suitable for green-threaded cooperative task switching.

- ✅ Fully supported today
- ⚙️ Implements its own scheduler + stack allocation
- ❗ External dependency

### 2. Native Generators (Rust 2024)

Rust 2024 introduces first-class support for generators via the `Generator` trait in `std`, and paves the way for future coroutine ergonomics.

- ✅ Zero-dependency
- 🧪 Experimental (but progressing toward stabilization)
- 📈 Future-compatible with `gen`, `async`, and structured concurrency

---

## Current MVP Plan

- Use boxed `Generator` trait objects
- Abstract over coroutine impls with a trait (e.g., `CoroutineTask`)
- Provide pluggable dispatch engine later

---

## Goals

- ✅ Deterministic scheduling
- ✅ Replayable yield/resume traces
- ✅ Strong test coverage

---

## Open Questions

- Should we support both engines side-by-side via feature flags?
- What tradeoffs exist for state snapshotting?
- Do generators offer enough for yield chaining + resumption graphs?

---

> Contributors should annotate all coroutine yield points clearly with `#[instrument]` or similar, and help evolve this crate to support one or both coroutine models.
