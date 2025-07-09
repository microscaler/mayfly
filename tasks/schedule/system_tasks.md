## 2. `tasks/scheduler/system_tasks.md`


# ⚙️ Task: System (Supervisor) Tasks

## Context
Internal runtime jobs (e.g., WAL flusher, metrics pusher) need the highest priority and must never starve.
WAL flusher, metrics pusher, and GC sweeper must pre-empt user tasks.


## Acceptance
* `Scheduler::spawn_system<F>(f) -> TaskId` spawns with priority `0`.
* Existing `spawn` delegates to priority `10`.
* New test `system_priority.rs` confirms system task completes before others.

## Steps
1. Ensure Priority Queue (Phase 4D) is merged.
* Priority queue implementation (already added) dispatches priority 0 first.
2. Implement `spawn_system` using `spawn_with_priority(0, f)`.
* `tests/system_priority.rs`:
    * Implement `spawn_system` using `spawn_with_priority(0, f)`.
    * Existing `spawn` delegates to priority `10`.
    * New test `system_priority.rs` confirms system task completes before others.
    * Spawn sys task logging `"sys"`, user task logging `"user"`.
    * Assert done order `[sys, user]`.
3. Add doc update to crate README.

* API:

```rust
pub unsafe fn spawn_system<F>(&mut self, f: F) -> TaskId
where F: FnOnce(TaskContext)+Send+'static;
```

---

## Implementation Steps

1. **Priority Constant**

```rust
pub const PRI_SYSTEM: u8 = 0;
pub const PRI_DEFAULT: u8 = 10;
```
2. **`spawn_system` Implementation**

```rust
pub unsafe fn spawn_system<F>(&mut self, f: F) -> TaskId
where F: FnOnce(TaskContext)+Send+'static {
   self.spawn_with_priority(PRI_SYSTEM, f)
}
```
3. **Boot tasks**
   In runtime init (daemon crate) add:

    * WAL flush loop (`looptask_flush_wal`)
    * Metrics push loop (`looptask_metrics`)
4. **Test** – use `serial_test::serial` to isolate logs.

---

## Edge Cases

* Ensure budget-throttling still applies to system tasks (maybe higher limit).
* If system task panics, scheduler should restart or fail fast (PAL event).


