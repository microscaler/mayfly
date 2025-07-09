## 3. `tasks/scheduler/cpu_budget.md`

# ⏱️ Task: CPU Budget / Time Slice Limiter

## Context
Prevent infinite loops from locking the VM; enforce fairness.
Infinite loops must yield; we enforce cooperative fairness by tracking
elapsed virtual time.

## Acceptance
* Each task receives a Default `quantum = 5ms` virtual time (configurable).
* `Task::consumed` ticks field.
* If a task exhausts quantum without yielding, scheduler re-queues it
  **after** others of equal priority. i.e consumed ≥ quantum without `SystemCall`, scheduler re-queues tid.
* Counter Metric `task_budget_throttle_total{tid}` increments on throttle..
* Test `budget_throttle.rs` ensures interleaving.
* Task loops 3× without yield; scheduler throttles; other task runs between.

## Steps
1. Add field `remaining_ticks` to `Task` struct.
   **Struct updates**
```rust
pub struct Task {
   tid: TaskId,
   handle: JoinHandle<()>,
   consumed: Duration,
}
```
2. On every loop iteration subtract elapsed (use `TickClock` diff).
   **`Scheduler::spawn*`**: set `consumed = Duration::ZERO`.
3. **During run loop**
   If <= 0:
    * reset to default,
    * push to ready queue end (same priority),
    * emit PAL event `Throttled`.

    * Measure `elapsed = now - last_tick`.
    * `task.consumed += elapsed`.
    * If `>= quantum` : reset to 0, emit `TaskEvent::Throttled`, push to queue.
4. Expose config: `Scheduler::set_default_quantum(Duration)`.
   **Config helper**
```rust
pub fn set_quantum(&mut self, q: Duration)
```

5. **Test**
* Task that loops 3× calls `ctx.yield_now()` only once—should not starve
  a low-priority sibling task.

---

## Notes

* Use `clock.now()` to derive durations (virtual time).
* Keep overhead minimal: simple `Duration` accumulation.
---

## 4. `tasks/scheduler/mem_guard.md`

# 🧩 Task: Memory Guard & Slab Allocator

## Context
Spawn storms can OOM the microVM. Provide predictable limits.
Even within a microVM, we want graceful degradation when spawn storm occurs.

## Acceptance
* Feature flag `mem-guard`.
* `Scheduler::with_slab(capacity: usize)` returns configured instance.
* `spawn()` fails with `SpawnError::OutOfMemory` when slab exhausted.
* PAL event `SpawnDenied { reason: OutOfMemory }`.
* Configurable slab size via `Scheduler::with_slab(capacity)`.
* Test `mem_guard.rs` – capacity = 1, attempt 2 spawns, second returns Err.

## Steps
1. Add `Slab<Task>` dependency (`slab` crate) behind feature flag.
1. **Add `slab = { version = "0.4", optional = true }` to Cargo.toml**
2. Replace `HashMap<TaskId, Task>` with `Slab`.
2. **Compile-time gating**
```rust
#[cfg(feature="mem-guard")]
use slab::Slab;
```
3. Track used vs capacity; return `Result<TaskId, SpawnError>`. Replace `HashMap<TaskId, Task>` with (feature-gated) `Slab<Task>`.
4. Update existing tests to unwrap or expect `Ok`. When slab full, return error + `pal::emit(SpawnDenied)`.
5. Provide convenience `spawn_or_panic` in tests when feature disabled.


### Edge Cases

* Keep `TaskId` deterministic: use slab index + offset so IDs never repeat
  in a session.
* In `shutdown`, iterate slab to drop remaining handles.

