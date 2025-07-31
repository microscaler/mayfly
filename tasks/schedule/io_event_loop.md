# 🌀 Task: Event Loop Integration (MIO)

## Goal
Replace the simple `io_tx` channel with real event polling so that
`Scheduler::run` can sleep until either:
1. next sleep timer expires
2. an I/O event arrives
3. a task is ready

## Acceptance
* `--features async-io` builds and all tests pass.
* `Scheduler::run` no longer uses `io_rx.recv_timeout`.
* New integration test `pipe_echo.rs`:
  - Reader task waits on FD
  - Writer thread writes after 20 ms
  - Scheduler wakes exactly once, order correct.

---
### Steps
1. **Remove** `io_tx/io_rx` fields & helper.
2. **Top of event loop**:
   * Determine smallest `sleepers` deadline.
   * Call `poll.poll(&mut events, timeout)` using that.
3. **Process events**: For each `event.token()` convert back to `u64`
   and call `wait_map.complete_io`.
4. **Feature-gate**:
   * When `async-io` disabled, keep current channel stub (tests rely on it).

