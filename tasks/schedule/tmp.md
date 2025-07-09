Below are **seven Codex-ready task files** (copy each into `tasks/scheduler/…`) that move Tiffany’s scheduler into Phases 4 & 5.

---

## 1. `tasks/scheduler/telemetry_hooks.md`

```md
# 📊 Task: Telemetry Hooks (Tracing, PAL events, Prometheus metrics)

## Context
Tiffany’s PAL/WAL and observability stack require per-task lifecycle events and
scheduler-level metrics.

## Acceptance
* Every state transition (`Ready→Running`, `Sleep`, `IoWait`, `Done`, `Panic`)
  emits a `pal::TaskEvent`.
* Scheduler loop wrapped in `tracing::span!("scheduler.loop", tid=? )`.
* New Prometheus metrics exposed via `metrics` crate:
  * `scheduler_ready_queue_depth` (gauge)
  * `task_run_latency_seconds` (histogram)
  * `task_wait_time_seconds` (histogram)

## Steps
1. **Add PAL stub if absent**  
   `pub fn emit(event: TaskEvent) { /* placeholder */ }`
2. **Define `TaskEvent`** in `scheduler::telemetry`.
3. In `Scheduler::run`:
   * Create span before processing each syscall.
   * Call `pal::emit(TaskEvent::State { tid, state })`.
4. Update metrics on:  
   * `ready.push()` / `pop()`  → gauge  
   * Before/after task execution → histograms
5. **Add unit-test** (`telemetry.rs`) verifying:
   * PAL receives `Done` event.
   * Gauge depth behaves as expected (use test registry).

```

---

## 2. `tasks/scheduler/system_tasks.md`

```md
# ⚙️ Task: System (Supervisor) Tasks

## Context
Internal runtime jobs (e.g., WAL flusher, metrics pusher) need the highest
priority and must never starve.

## Acceptance
* `Scheduler::spawn_system<F>(f) -> TaskId` spawns with priority `0`.
* Existing `spawn` delegates to priority `10`.
* New test `system_priority.rs` confirms system task completes before others.

## Steps
1. Ensure Priority Queue (Phase 4D) is merged.
2. Implement `spawn_system` using `spawn_with_priority(0, f)`.
3. Add doc update to crate README.

```

---

## 3. `tasks/scheduler/cpu_budget.md`

```md
# ⏱️ Task: CPU Budget / Time Slice Limiter

## Context
Prevent infinite loops from locking the VM; enforce fairness.

## Acceptance
* Each task receives a default quantum (e.g. 5 ms virtual time).
* If a task exhausts quantum without yielding, scheduler re-queues it
  **after** others of equal priority.
* Counter metric `task_budget_throttle_total` increments on throttle.
* Test `budget_throttle.rs`:
  * Task loops 3× without yield; scheduler throttles; other task runs between.

## Steps
1. Add field `remaining_ticks` to `Task` struct.
2. On every loop iteration subtract elapsed (use `TickClock` diff).
3. If <= 0:
   * reset to default,
   * push to ready queue end (same priority),
   * emit PAL event `Throttled`.
4. Expose config: `Scheduler::set_default_quantum(Duration)`.

```

---

## 4. `tasks/scheduler/mem_guard.md`

```md
# 🧩 Task: Memory Guard & Slab Allocator

## Context
Even within a microVM, we want graceful degradation when spawn storm occurs.

## Acceptance
* Optional feature `mem-guard`.
* `spawn()` fails with `SpawnError::OutOfMemory` when slab exhausted.
* PAL event `SpawnDenied { reason: OutOfMemory }`.
* Configurable slab size via `Scheduler::with_slab(capacity)`.

## Steps
1. Add `Slab<Task>` dependency (`slab` crate) behind feature flag.
2. Replace `HashMap<TaskId, Task>` with `Slab`.
3. Track used vs capacity; return `Result<TaskId, SpawnError>`.
4. Update existing tests to unwrap or expect `Ok`.

```

---

## 5. `tasks/scheduler/graceful_shutdown.md`

```md
# 📴 Task: Graceful Shutdown & Drain

## Context
FAR VM receives SIGTERM when orchestrator suspends; scheduler must flush state.

## Acceptance
* `Scheduler::shutdown(timeout: Duration)`:
  * sets `terminating = true`
  * rejects new spawns (`SpawnError::Terminating`)
  * waits for tasks to finish up to timeout
  * forces cancel afterwards.
* PAL emits `ShutdownBegin` and `ShutdownComplete`.
* Integration test `shutdown.rs` uses thread to send `shutdown()` mid-run,
  asserts WAL flush hook called.

## Steps
1. Add `terminating` boolean.
2. Wrap spawn paths with check.
3. Provide public `shutdown()`; call from daemon signal handler (future).
4. Hook WAL flush via trait object `ShutdownHook`.

```

---

## 6. `tasks/scheduler/backpressure_hook.md` *(optional)*

```md
# 📡 Task: Back-Pressure Hook for Shared Storage

## Goal
Allow WAL layer to signal saturation; scheduler backs off IO-heavy tasks.

## Acceptance
* Define trait `BackPressureListener { fn on_backpressure(level: f32) }`
  (0.0 = fine, 1.0 = critical).
* Scheduler pauses tasks currently in `SystemCall::IoWait` if level ≥ 0.8.
* Metric `backpressure_pauses_total`.

## Steps
1. Inject `listener: Arc<dyn BackPressureListener>` field (optional).
2. In IO event loop check `listener.on_backpressure()`.
3. Demonstrate with mock listener in test.

```

---

## 7. `tasks/scheduler/sharded_api.md` *(optional prototype)*

```md
# 🧵 Task: Sharded Executor Prototype

## Context
Future multi-core VMs will need multiple cooperative schedulers with
work-stealing ready queues.

## Acceptance
* New trait `Executor` with `spawn`, `spawn_with_priority`, `run`.
* Struct `ShardPool { schedulers: Vec<Scheduler> }` implements `Executor`.
* Simple round-robin spawn; naive steal on empty ready queue.
* Benchmark placeholder in `benches/shard.rs`.

## Steps
1. Move `Scheduler` behind `trait Executor`.
2. Implement `ShardPool::run()` launching each scheduler on its own thread.
3. Work-stealing: if local ready empty, lock another scheduler’s ready pop.

```

---

**Tip:** Land items 1-5 first for production readiness; 6-7 bring advanced scaling but can trail.
