
# 🤝 Contributing to Tiffany

Welcome! We're excited you're interested in contributing to **Tiffany**, an autonomous agentic runtime built in Rust.

This document outlines our contribution philosophy, coding standards, and the processes we use to maintain project quality — especially our emphasis on **Test-Driven Development**, **comprehensive documentation**, and **system-level reproducibility**.

---

## 🧪 Test-Driven Development (TDD)

We follow a **strict test-first workflow**.

### ✅ Requirements:
- **Write a failing test before implementing logic**
- **Unit tests are mandatory for all new features**
- **Behavioral tests must cover all public APIs**
- **Pact tests are required for all external service interactions**
- **Integration tests are expected for component interaction**
- **No untested code is accepted into `main`**

### 📊 Code Coverage:
- Aim for **70% coverage** on all new code
- Ideally achieve **90%+ coverage** on components

You may use `#[ignore]` for tests involving networked tools or MCP dependencies, but local unit coverage must always be high.

### 📂 Test locations:
- `tests/` — full integration pipelines
- `crates/<name>/tests/` — crate-level integration tests
- `crates/<name>/src/lib.rs` — unit and behavioral tests

> Run tests locally with:
```bash
cargo test --workspace
````

---

## 📚 Project Documentation

We maintain a **living Docbook** in the `docs/` folder that is always in sync with the source tree.

### 📖 Requirements:

* All **public functions**, **structs**, **traits**, and **enums** must be documented using Rustdoc comments (`///`)
* Any **crate-level or module-level design logic** must be described using `//!` inner docs
* All **new crates** must include a `README.md` and `examples/` folder if relevant

> Generate the local Docbook with:

```bash
cargo doc --workspace --no-deps --open
```

This will open a full system-level HTML documentation site locally.

---

## 📘 `docs/whitepapers` and Diagrams

If you're implementing a major feature (new tool, agent skill, MCP server, etc.), submit a short technical write-up in `docs/whitepapers/` **before or alongside the PR**. Use markdown and diagrams (preferably Mermaid or SVG).

### ✍ Example:

* `docs/whitepapers/my_module_design.md`
* `docs/diagrams/my_module_flow.mmd`

---

## 🧱 Coding Style & Linting

We follow idiomatic Rust standards and enforce them via:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features
```

* Avoid `.unwrap()` and `.expect()` in library code
* Prefer `?` and structured error types
* Use `Result<T, E>` for all fallible operations

---

## 📦 Commit & Branching Guidelines

* **Use conventional commits**, e.g.:

  ```
  feat(core): add scheduler job ID support
  fix(executor): handle early cancellation edge case
  doc(readme): add whitepaper index
  ```
* **Branch from `main`** and submit PRs into `main`
* PRs must pass all tests and include docs for all public APIs

---

## 🚨 Breaking Changes

If your change:

* alters a public API signature
* modifies WAL schema
* changes any REST/gRPC interface

Then mark the PR title with `BREAKING:` and ping a maintainer for review.

---

## 🧙 Maintainers & Decision Process

* This project is maintained by [Microscaler](https://github.com/microscaler)
* Major architecture changes are discussed via [GitHub Discussions](https://github.com/microscaler/tinkerbell/discussions)
* All code should ship behind tests and with updated documentation

---

Thank you for helping build **Tiffany** into a powerful, safe, and self-aware runtime! ✨

