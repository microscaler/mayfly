Below are **enriched versions** of the seven Phase-4/5 scheduler task files, each with deeper implementation hints, edge-case notes, and sample snippets.
Copy them into `tasks/scheduler/…` to guide Codex or any contributor.

---

### 1️⃣ `tasks/scheduler/telemetry_hooks.md`

````md
# 📊 Task: Telemetry Hooks (Tracing, PAL events, Prometheus metrics)

## Context
Tiffany’s PAL (Process Activity Log), WAL, and metrics dashboards are only as
useful as the events emitted by the scheduler. We need *consistent* lifecycle
events and low-overhead metrics for every state transition.

---

## Acceptance Criteria
| Item | Requirement |
|------|-------------|
| **PAL events** | Emit `TaskEvent::{Spawned,Running,Yield,Sleep,IoWait,Done,Failed,Throttled}` |
| **Tracing spans** | Wrap each `Scheduler::run` loop iteration in a span<br/>`span!("sched.tick", tid=?tid, state=?state)` |
| **Prometheus** | <br/>• `scheduler_ready_queue_depth` (gauge)<br/>• `tasks_running_total` (counter · label `state=done|failed`)<br/>• `task_run_latency_seconds` (histogram `le` buckets 1ms..2s)<br/>• `task_wait_time_seconds` (histogram) |

* Every state transition (`Ready→Running`, `Sleep`, `IoWait`, `Done`, `Panic`)
  emits a `pal::TaskEvent`.
* Scheduler loop wrapped in `tracing::span!("scheduler.loop", tid=? )`.
* New Prometheus metrics exposed via `metrics` crate:
   * `scheduler_ready_queue_depth` (gauge)
   * `task_run_latency_seconds` (histogram)
   * `task_wait_time_seconds` (histogram)
   
All existing tests must remain green.

---

## Implementation Steps

1. **PAL stub (if not yet implemented)**
   ```rust
   // crates/pal/src/lib.rs
   #[derive(Debug, Clone)]
   pub enum TaskEvent { Spawned(TaskId), Running(TaskId), Yield(TaskId),
                        Sleep{tid:TaskId,dur:Duration}, IoWait{tid,io:u64},
                        Done(TaskId), Failed{tid,error:String}, Throttled(TaskId) }

   pub fn emit(evt: TaskEvent) { tracing::debug!(?evt, "PAL"); }
````

2. **Tracing integration**

    * Add `tracing` to `scheduler/Cargo.toml` if not present (already used).
    * In `Scheduler::spawn*`, emit `TaskEvent::Spawned` and `tracing::info!`.
    * In `run()`:

      ```rust
      let span = tracing::span!(tracing::Level::TRACE, "sched.tick", tid);
      let _enter = span.enter();
      pal::emit(TaskEvent::Running(tid));
      ```

3. **Metric registration**

   ```rust
   use metrics::{register_gauge, describe_gauge, gauge, ...};

   static READY_DEPTH: Lazy<Gauge> = Lazy::new(|| {
       describe_gauge!("scheduler_ready_queue_depth", "Tasks waiting to run");
       register_gauge!("scheduler_ready_queue_depth")
   });
   // update on every push/pop
   ```

   Histograms via `metrics::histogram!`.

4. **Compute latencies**

    * Record `Instant` at pop; on re-queue compute `run_dur`.
    * For wait histogram: when task first becomes not-ready (sleep/join/iowait)
      store `start_wait` in small hashmap; compute delta on resume.

5. **Tests**

    * Use `metrics_util::registry::Registry` to capture values.
    * Assert gauge depth rises/falls as expected.
    * Unit test logs: set `RUST_LOG=trace`, capture `tracing_subscriber::fmt`
      output and assert PAL events appear.

---

## Notes & Edge Cases

* Keep telemetry overhead < 5 µs per transition (no string allocs inside loop).
* Histogram buckets: `[0.001,0.005,0.010,0.050,0.100,0.250,0.5,1,2]` seconds.
* Consider compile-time feature `telemetry` to strip all when disabled.

````

---

### 2️⃣ `tasks/scheduler/system_tasks.md`

```md
# ⚙️ Task: System / Supervisor Tasks (Priority 0)

## Context
Internal runtime jobs (e.g., WAL flusher, metrics pusher) need the highest
priority and must never starve.
WAL flusher, metrics pusher, and GC sweeper must pre-empt user tasks.

---

## Acceptance
* API:  
  ```rust
  pub unsafe fn spawn_system<F>(&mut self, f: F) -> TaskId
  where F: FnOnce(TaskContext)+Send+'static;
````

* Priority queue implementation (already added) dispatches priority 0 first.
* `tests/system_priority.rs`:
    * Implement `spawn_system` using `spawn_with_priority(0, f)`.
    * Existing `spawn` delegates to priority `10`.
    * New test `system_priority.rs` confirms system task completes before others.
    * Spawn sys task logging `"sys"`, user task logging `"user"`.
    * Assert done order `[sys, user]`.
* Add doc update to crate README.

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

````

---

### 3️⃣ `tasks/scheduler/cpu_budget.md`

```md
# ⏱️ Task: CPU Budget / Time Slice Limiter

## Context
Infinite loops must yield; we enforce cooperative fairness by tracking
elapsed virtual time.

---

## Acceptance
* Each task receives a Default `quantum = 5ms` virtual time (configurable).
* `Task::consumed` ticks field.
* If a task exhausts quantum without yielding, scheduler re-queues it
  **after** others of equal priority. i.e consumed ≥ quantum without `SystemCall`, scheduler re-queues tid.
* Counter Metric `task_budget_throttle_total{tid}` increments.
* Test `budget_throttle.rs` ensures interleaving.

---

## Steps
1. **Struct updates**
   ```rust
   pub struct Task {
       tid: TaskId,
       handle: JoinHandle<()>,
       consumed: Duration,
   }
````

2. **`Scheduler::spawn*`**: set `consumed = Duration::ZERO`.
3. **During run loop**

    * Measure `elapsed = now - last_tick`.
    * `task.consumed += elapsed`.
    * If `>= quantum` : reset to 0, emit `TaskEvent::Throttled`, push to queue.
4. **Config helper**

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

````

---

### 4️⃣ `tasks/scheduler/mem_guard.md`

```md
# 🧩 Task: Memory Guard & Slab Allocator

## Context
Spawn storms can OOM the microVM. Provide predictable limits.

---

## Acceptance
* Feature flag `mem-guard`.
* `Scheduler::with_slab(capacity: usize)` returns configured instance.
* `spawn` returns `Result<TaskId, SpawnError::OutOfMemory>`.
* Test `mem_guard.rs` – capacity = 1, attempt 2 spawns, second returns Err.

---

## Steps
1. **Add `slab = { version = "0.4", optional = true }` to Cargo.toml**
2. **Compile-time gating**
   ```rust
   #[cfg(feature="mem-guard")]
   use slab::Slab;
````

3. Replace `HashMap<TaskId, Task>` with (feature-gated) `Slab<Task>`.
4. When slab full, return error + `pal::emit(SpawnDenied)`.
5. Provide convenience `spawn_or_panic` in tests when feature disabled.

---

### Edge Cases

* Keep `TaskId` deterministic: use slab index + offset so IDs never repeat
  in a session.
* In `shutdown`, iterate slab to drop remaining handles.

````

---

### 5️⃣ `tasks/scheduler/graceful_shutdown.md`

```md
# 📴 Task: Graceful Shutdown & Drain

## Context
When orchestrator sends SIGTERM, Tiffany needs to:
1. Stop accepting new work.
2. Let running tasks finish (bounded).
3. Flush WAL / PAL.
4. Exit cleanly.

---

## Acceptance
* `Scheduler::shutdown(timeout)` returns `ShutdownResult::{Clean,Timeout}`.
* New test `shutdown.rs`:
  * Spawn long sleeper (100ms), call `shutdown(30ms)`.
  * Expect Timeout variant, task cancelled.
* PAL events emitted:
  * `ShutdownBegin`, `ShutdownComplete{clean:bool}`

---

## Steps
1. **`terminating` bool + `deadline: Option<Instant>`**
2. `spawn_*` functions check `terminating`; return `Err(SpawnError::ShuttingDown)`.
3. In `run` loop, if terminating and tasks empty → break.
4. After deadline, force-cancel remaining tasks.
5. Provide helper:
   ```rust
   impl Scheduler {
       pub fn initiate_shutdown(&mut self, timeout: Duration) { ... }
   }
````

6. **Daemon crate**: register `ctrlc::set_handler` calling `initiate_shutdown`.

---

## Edge Cases

* Cancelled tasks must have `Failed(Cancelled)` PAL event so waiters wake.
* WAL flush hook must run **after** all tasks done but **before** return.

````

---

### 6️⃣ `tasks/scheduler/backpressure_hook.md` (optional)

```md
# 📡 Task: Back-Pressure Hook

## Context
Shared storage saturation should slow IO-heavy tasks to avoid global stalls.

---

## Acceptance
* Trait `BackPressureListener` with `fn pressure(&self) -> f32` (0-1).
* Scheduler consults listener every N ticks (configurable).
* If `>= 0.8`, pause tasks in IoWait by not re-queueing them until pressure < 0.5.
* Metric `backpressure_pauses_total`.

---

## Implementation Hints
* Keep a `paused_io: HashSet<TaskId>`; when pressure drops, move back to ready.
* Provide stub listener returning `0.0` when feature disabled.

````

---

### 7️⃣ `tasks/scheduler/sharded_api.md` (optional prototype)

````md
# 🧵 Task: Sharded Executor Prototype

## Goal
Demonstrate multi-threaded scaling with minimal work-stealing.

---

## Acceptance
* Trait:
  ```rust
  pub trait Executor {
      unsafe fn spawn<F>(&self, f: F) -> TaskId
      where F: FnOnce(TaskContext)+Send+'static;
      fn run(&self);
  }
````

* `ShardPool::new(cores)` spawns `cores` independent `Scheduler`s.
* Work-stealing: if local ready empty, pop from next scheduler in ring.
* Bench (`benches/shard.rs`) shows linear speed-up on >1 core.

---

## Implementation Hints

* Use `crossbeam::deque::{Injector, Steal, Worker}` for global & local queues.
* Use `scope` from `crossbeam` to spawn OS threads.
* Keep telemetry per-shard; aggregate via Prom labels (`shard=X`).

```

---

These enriched tasks give contributors the details needed to harden Tiffany’s scheduler and push it toward production-grade runtime behavior.
```
