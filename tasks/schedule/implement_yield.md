> NOTE!! This task set depends on completion of fix-stale-ready-ids.md

### `tasks/scheduler/implement_yield.md`

````md
# ✨ Task: Introduce `yield_now()` and Cooperative Stepping

## Context
PyOS8’s coroutines explicitly `yield` back to the scheduler every logical step.
We will mirror this by adding `TaskContext::yield_now()` and making
`Scheduler::run` use the ready queue rather than blocking on `recv_timeout`.

## Acceptance
* A new `yield_now()` call exists and is callable from inside tasks.
* Scheduler round-robins fairly: if a task calls `yield_now`, it is re-queued
  and the next ready task is executed.
* Existing tests still pass; add a new test `yield_order` verifying round-robin.

---

## Steps

- [x] **Extend `SystemCall`**

  ```rust
  SystemCall::Yield,
````

* [x] **Add helper on `TaskContext`**

  ```rust
  impl TaskContext {
      pub fn yield_now(&self) {
          self.syscall(SystemCall::Yield);
      }
  }
  ```

* [x] **Handle `Yield` in `Scheduler::run`**

  ```rust
  SystemCall::Yield => { /* no-op except requeue */ }
  ```

* [x] **Re-queue immediately without sleep**

  Ensure `requeue = true` for `Yield`.

* [x] **Add regression test**

  `crates/scheduler/tests/yield.rs`

  ```rust
  #[test]
  fn yield_order() {
      let mut s = Scheduler::new();
      let a = unsafe { s.spawn(|ctx| { ctx.yield_now(); ctx.syscall(SystemCall::Done); }) };
      let b = unsafe { s.spawn(|ctx| { ctx.syscall(SystemCall::Done); }) };
      let done = s.run();
      assert_eq!(done, vec![b, a]); // because a yielded once
  }
  ```

