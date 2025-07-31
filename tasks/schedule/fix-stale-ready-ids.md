### `tasks/scheduler/fix-stale-ready-ids.md`

````md
# 🐞 Fix: Stale Task IDs in ReadyQueue cause panic (`task not found`)

## Context
`Scheduler::run` pops a `tid` from `ready`, then immediately looks it up in
`self.tasks`.  
If the task finished earlier (after a `SystemCall::Done`) **and** an older copy of
its `tid` is still in the queue, the lookup panics.

## Acceptance
* All scheduler unit-tests and integration tests pass (no `task not found` panic).
* `cargo nextest run --workspace` returns success.

---

## Tasks

- [x] **Guard lookup in `Scheduler::run`**

  *File*: `crates/scheduler/src/scheduler.rs`  
  *Locate* the first `_task = self.tasks.get_mut(&tid)…` line (≈ l. 96).  
  *Insert* guard **before** it:

  ```rust
  if self.tasks.get(&tid).is_none() {
      // Stale ID—task already removed after Done; skip.
      continue;
  }
````

* [x] **(Optional) Deduplicate pushes**

  In every spot that calls `self.ready.push(tid)` add:

  ```rust
  if !self.ready.contains(tid) {
      self.ready.push(tid);
  }
  ```

  > Requires adding a simple `contains(&self, tid: TaskId) -> bool` helper to
  > `ReadyQueue`.

* [x] **Add regression test**

  *File*: `crates/scheduler/tests/stale_ready.rs`

  ```rust
  use scheduler::{Scheduler, SystemCall, task::TaskContext};

  #[test]
  fn stale_ready_id_is_ignored() {
      let mut sched = Scheduler::new();
      let child = unsafe {
          sched.spawn(|ctx: TaskContext| {
              ctx.syscall(SystemCall::Done);
          })
      };
      // enqueue the same tid twice to mimic duplicate push
      sched.ready_push_duplicate_for_test(child);
      let order = sched.run();
      assert_eq!(order, vec![child]); // scheduler should not panic
  }
  ```

  You may add a `#[cfg(test)]` helper in `scheduler.rs`:

  ```rust
  #[cfg(test)]
  impl Scheduler {
      pub fn ready_push_duplicate_for_test(&mut self, tid: TaskId) {
          self.ready.push(tid);
      }
  }
  ```

* [x] **Run full test suite**

  ```bash
  cargo nextest run --workspace
  ```

* [x] **Commit**

  ```
  feat(scheduler): guard against stale task-ids in ReadyQueue
  ```

---

## Notes / Stretch

* Investigate replacing raw `VecDeque` with a small set-aware queue to avoid duplicates wholesale.
* Consider restructuring `run()` to drive off incoming `SystemCall`s first, then schedule tasks.
