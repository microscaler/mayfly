# 🤖 Codex Agent Protocol: Tinkerbell

Welcome, Codex or autonomous contributor.

Tinkerbell is a coroutine-first autonomous agent runtime. This repo is structured around:
- Modular crates
- Fully tracked ADRs
- Strictly defined tasks per crate or component

## Agent Workflow Rules

1. **All changes must be tied to a task.**
    - Tasks are stored in `tasks/<crate>/tasks.md`

2. **Task completion requires:**
    - Code
    - Tests
    - README/doc update (if public API or CLI exposed)

3. **You may generate new tasks** from:
    - `docs/mdbook/src/adr/*.md`
    - `docs/mdbook/src/concepts/*.md`
    - `canvas/` (if reason-act context is active)

4. **Do not write to crates outside the current task scope.**
    - Use interfaces exposed by other crates.
    - If you need changes, create a dependency task.

5. **Tests must be deterministic and run under `just test`.**

# Scheduler Task Refinement (MVP Fixes)

## Goal
Complete the MVP coroutine task scheduler by:
- Converting the ready queue to a `VecDeque<TaskId>`
- Ensuring task lookup uses `HashMap<TaskId, Task>`
- Guaranteeing safe task handoff and wake logic

## Tasks

- [x] Replace `VecDeque<Task>` with `VecDeque<TaskId>` in `scheduler.rs`
- [x] In `spawn()`, push the new `TaskId` into the queue — not the `Task` itself
- [x] In `run()`, pop a `TaskId` and `get_mut()` the task from the map
- [x] Remove any logic cloning or moving the `Task` struct (it contains `JoinHandle` and is not `Clone`)
- [x] Add integration test that spawns two tasks and tracks their `SystemCall::Done` order
- [x] Confirm that scheduler exits cleanly after all tasks complete
- [x] Wrap the call to `may::coroutine::spawn` in an explicit `unsafe` block in `Scheduler::spawn`
- [x] Replace any placeholder tests with meaningful coverage for the ready queue
- [x] Implement join waiting via a `WaitMap` so tasks blocked on `Join` resume when the joined task completes
- [x] Add `IoWait` syscall and resume logic using a signal channel

## Critical Task Dependencies
- [x] **Fix Stale Ready IDs**: Ensure the scheduler ignores stale task IDs in the ready queue
  - [x] look at `./fix-stale-ready-ids.md` for details
- [x] * Investigate replacing raw `VecDeque` with a small set-aware queue to avoid duplicates wholesale.
- [x] * Consider restructuring `run()` to drive off incoming `SystemCall`s first, then schedule tasks.
- [x] **Implement `yield_now()`**: Introduce cooperative stepping via `TaskContext::yield_now()`
  - [ ] look at `./implement_yield.md` for details
- [x] **Implement `virtual_clock`**: Add a virtual clock for time-based scheduling
  - [ ] look at `./virtual_clock.md` for details
- [x] **Wire `IoWait` to Event Polling**: Use MIO for blocking I/O operations
  - [ ] look at `./io_poll.md` for details
- [ ] **Implement `io_registry_trait` look at `./io_registry_trait.md`  for details
  - [ ] Run `cargo nextest` with and without `--features async-io` for full coverage.
- [ ] **Implement `io_event_loop` and  look at `./io_event_loop.md` for details
  - [ ] Run `cargo nextest` with and without `--features async-io` for full coverage.
- [ ] **Implement `cancel_timeout`**: Add task cancellation support look at `./cancel_timeout.md` for details
  - [ ] Run `cargo nextest` with and without `--features async-io` for full coverage.
- [ ] **Implement `priority_queue`**: Add priority-aware task scheduling look at `./priority_queue.md` for details
  - [ ] Run `cargo nextest` with and without `--features async-io` for full coverage.
- [ ] **Implement `panic_isolation`**: Ensure task panics do not crash the scheduler look at `./panic_isolation.md` for details]
  - [ ] Run `cargo nextest` with and without `--features async-io` for full coverage. 


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



