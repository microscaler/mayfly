# ADR-0002: Documentation Standards and Docbook Policy

## Status
Accepted

## Context
Tiffany is an autonomous agentic runtime composed of multiple crates, task orchestration layers, plugins, and memory systems. As the project scales and contributors increase, keeping documentation complete, consistent, and accessible is critical.

This ADR defines our approach to documentation, centered around:
- Generating and maintaining a **Rust Docbook** as a version-controlled, browsable reference.
- Enforcing **comprehensive inline documentation** at all public API boundaries.
- Integrating `mdbook`-based module-level design notes into the same output tree.

This policy ensures Tiffany is a self-documenting system where new contributors, auditors, and maintainers can explore and understand internals without guesswork.

---

## Decision

We adopt the following standards:

### 1. **Docbook Output**
- The unified system documentation will be generated via the `docsbookgen` crate.
- Output lives in `docsbook/` and includes:
    - `docsbook/api/` – `cargo doc` generated documentation for all public crates.
    - `docsbook/md/` – rendered `mdbook` pages from `docs/mdbook/`.

### 2. **Inline Documentation Standards**
- All **public functions, structs, enums, traits** must have a meaningful `///` doc comment.
- All **crate-level modules** must include a `//!` overview explaining design intent.
- Documentation must include usage examples where applicable.

### 3. **Good vs. Bad Examples**

#### ❌ Bad Documentation
```rust
/// Do stuff
do_stuff(x: i32) -> i32 {
    // ???
}
```

- Lacks purpose, parameter explanation, or return context.
- Cannot be indexed or searched effectively.

#### ✅ Good Documentation
```rust
/// Converts an input value to its mirrored form.
///
/// # Arguments
/// * `x` - The input value to transform
///
/// # Returns
/// A mirrored integer that inverts digits and preserves sign.
///
/// # Example
/// ```
/// let mirrored = mirror(123);
/// assert_eq!(mirrored, 321);
/// ```
fn mirror(x: i32) -> i32 {
    // implementation
}
```

- Clearly explains purpose
- Uses Markdown headers
- Provides examples and rationale

### 4. **Doc Generation Policy**
- `cargo doc --workspace --no-deps` is run as part of CI.
- `./scripts/gen_docsbook.sh` should be run before every tagged release.
- Contributors are expected to review their `cargo doc` output locally.

---

## Rationale

### 📚 Documentation = Accessibility
Comprehensive inline docs make the system easier to:
- Explore for new contributors
- Audit for correctness and security
- Extend or replace components with confidence

### 🧠 Reduces Tribal Knowledge
Many agent runtimes and infra projects degrade over time due to undocumented internals. By requiring inline and rendered docs at every level, we prevent that rot.

### 🔎 Searchable, Indexed APIs
`cargo doc` output can be hosted, indexed, and browsed offline. This allows all contributors to quickly understand type signatures, data models, and module boundaries.

### 🧪 Better Testing Support
Well-documented examples often double as tests. Using `///` examples that compile and run (`doc-tests`) ensures correctness and up-to-date documentation.

### 🤝 Contributor Enablement
With a living docbook, we avoid contributor gatekeeping. Anyone can onboard from the docs and find:
- What a module does
- How to use a crate
- Which tools are registered where

---

## Consequences

- PRs without adequate documentation will be blocked.
- CI will enforce `cargo doc` and docbook generation in future stages.
- Markdown-based `mdbook` diagrams, design notes, and architecture files will live beside the API docbook.

---

## Alternatives Considered

- **Minimal documentation, rely on code** – Faster in the short term, but fails at scale and disables contributor autonomy.
- **Only Markdown (`mdbook`) docs** – Good for high-level guides, but misses APIs and crate-level types. Docbook unifies both.

---

## Related Documents
- [scripts/gen_docsbook.sh](../../scripts/gen_docsbook.sh)
- [crates/docsbookgen/](../../crates/docsbookgen/)
- [docs/mdbook/](../../docs/mdbook/)

---

## Adopted
This ADR is accepted as of June 2025. All contributors are required to document public APIs, crate-level modules, and submit their design docs alongside code. Releases must include an up-to-date generated `docsbook/` output.

Maintainers: `@casibbald`, `@microscaler-team`
