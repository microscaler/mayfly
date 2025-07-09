## 7. `tasks/scheduler/graceful_shutdown.md`

# 📴 Task: Graceful Shutdown & Drain

## Context
FAR VM receives SIGTERM when orchestrator suspends; scheduler must flush state.
When orchestrator sends SIGTERM, Tiffany needs to:
1. Stop accepting new work.
2. Let running tasks finish (bounded).
3. Flush WAL / PAL.
4. Exit cleanly.

## Acceptance
* `Scheduler::shutdown(timeout: Duration)`:
* `Scheduler::shutdown(timeout: Duration)` returns `ShutdownResult::{Clean,Timeout}`.
    * sets `terminating = true`
    * rejects new spawns (`SpawnError::Terminating`)
    * waits for tasks to finish up to timeout
    * forces cancel afterwards.
* PAL events emitted:
    * `ShutdownBegin`, `ShutdownComplete{clean:bool}`
* Integration test `shutdown.rs` uses thread to send `shutdown()` mid-run,
  asserts WAL flush hook called.
* New test `shutdown.rs`:
    * Spawn long sleeper (100ms), call `shutdown(30ms)`.
    * Expect Timeout variant, task cancelled.

## Steps
1. Add `terminating` boolean + `deadline: Option<Instant>`**
2. Wrap spawn paths with check. `spawn_*` functions check `terminating`; return `Err(SpawnError::ShuttingDown)`.
3. Provide public `shutdown()`; call from daemon signal handler (future). In `run` loop, if terminating and tasks empty → break.
4. Hook WAL flush via trait object `ShutdownHook`. After deadline, force-cancel remaining tasks.
5. Provide helper:
```rust
impl Scheduler {
   pub fn initiate_shutdown(&mut self, timeout: Duration) { ... }
}
````
6. **Daemon crate**: register `ctrlc::set_handler` calling `initiate_shutdown`.


## Edge Cases

* Cancelled tasks must have `Failed(Cancelled)` PAL event so waiters wake.
* WAL flush hook must run **after** all tasks done but **before** return.

---