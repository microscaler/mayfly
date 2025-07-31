# 🛡️ Task: Panic Isolation & Error Propagation

## Goal
A coroutine panic must not crash the whole VM; instead:
* mark task `Failed`
* wake any joiners with failure
* write PAL entry

## Acceptance
* Wrap `task.handle.join()` in `match` on `Result<_, Box<dyn Any>>`.
* Add test `panic_isolation.rs` – child panics, parent join receives failure.

---
### Steps
1. Store task exit status in a new `TaskState` enum.
2. Update `WaitMap::complete` to return status.
3. Log to `PAL` via placeholder `pal::emit(TaskEvent::Failed(tid, msg))`.

