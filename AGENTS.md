# 🤖 Codex Agent Protocol: Mayfly

Welcome, Codex or autonomous contributor.

Tiffany is a coroutine-first autonomous agent runtime. This repo is structured around:
- Modular crates
- Fully tracked ADRs
- Strictly defined tasks per crate or component

## Agent Workflow Rules

1. **All changes must be tied to a task.**
    - Tasks are stored in `tasks/<crate>/tasks.md`

2. **Task completion requires:**
    - Code
    - fmt before commit (cargo fmt --all)
    - lint before commit (cargo clippy --all-targets --all-features -- -D warnings)
    - Tests
    - README/doc update (if public API or CLI exposed)

3. **You may generate new tasks** from:
    - `docs/mdbook/src/adrs/*.md`
    - `docs/mdbook/src/concepts/*.md`
    - `docs/mdbook/src/whitepapers/*.md`
    - `canvas/` (if reason-act context is active)

4. **Do not write to crates outside the current task scope.**
    - Use interfaces exposed by other crates.
    - If you need changes, create a dependency task.

5. **Tests must be deterministic and run under `just test`.**

## Crate Types

| Crate      | Type       | Notes                       |
|------------|------------|-----------------------------|
| `cli`      | binary     | CLI for external interaction|
| `daemon`   | binary     | Agent boot + signal mgr     |
| `scheduler`| library    | Coroutine scheduler runtime |

## Communication Interfaces

- IPC via UDS or vsock
- gRPC (async, tonic)

## Output Guidelines

- All structs, enums, traits must be documented.
- Use `tracing::instrument` for runtime behavior visibility.
- When in doubt, log to PAL-compatible format.

Happy contributing 🧚
