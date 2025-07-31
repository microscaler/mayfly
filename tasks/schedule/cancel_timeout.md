# ✂️ Task: Task Cancellation & Timeout (PyOS8 parity)

## Goal
Allow a task (or supervisor) to cancel another task (`SystemCall::Cancel`)
and allow `Join` with timeout.

## Acceptance
* New enum variant:

  ```rust
  SystemCall::Cancel(TaskId)
  SystemCall::JoinTimeout { target: TaskId, dur: Duration }
  ```

* If a task is cancelled it receives a panic-safe drop (do not unwind
  into scheduler). Waiters on that task get woken with an **error** state.
* New tests:

    * `cancel.rs` – parent cancels child, parent completes.
    * `join_timeout.rs` – JoinTimeout returns after virtual clock tick.

---

### Hints

* Keep `cancelled: HashSet<TaskId>` in `Scheduler`.
* For join-timeout store `(wake_at, waiter_tid, target_tid)`.
* Return an enum `JoinResult::{Success, Timeout, Cancelled}` via
  `ctx.syscall(SystemCall::Return(ResultVariant))` (or a simpler log for now).

