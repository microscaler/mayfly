## 1. `tasks/scheduler/telemetry_hooks.md`


# 📊 Task: Telemetry Hooks (Tracing, PAL events, Prometheus metrics)

## Context
Tiffany’s PAL/WAL and observability stack require per-task lifecycle events and
scheduler-level metrics. Tiffany’s PAL (Process Activity Log), WAL, and metrics dashboards are only as
useful as the events emitted by the scheduler. We need *consistent* lifecycle
events and low-overhead metrics for every state transition.

## Acceptance Criteria* Every state transition (`Ready→Running`, `Sleep`, `IoWait`, `Done`, `Panic`)
emits a `pal::TaskEvent`.
* Scheduler loop wrapped in `tracing::span!("scheduler.loop", tid=? )`.
* New Prometheus metrics exposed via `metrics` crate:
    * `scheduler_ready_queue_depth` (gauge)
    * `task_run_latency_seconds` (histogram)
    * `task_wait_time_seconds` (histogram)
* Every state transition (`Ready→Running`, `Sleep`, `IoWait`, `Done`, `Panic`)
  emits a `pal::TaskEvent`.

| Item              | Requirement                                                                                           |
|-------------------|-------------------------------------------------------------------------------------------------------|
| **PAL events**    | Emit `TaskEvent::{Spawned,Running,Yield,Sleep,IoWait,Done,Failed,Throttled}`                          |
| **Tracing spans** | Wrap each `Scheduler::run` loop iteration in a span<br/>`span!("sched.tick", tid=?tid, state=?state)` |
| **Prometheus**    | <br/>• `scheduler_ready_queue_depth` (gauge)<br/>• `tasks_running_total` (counter · label `state=done |failed`)<br/>• `task_run_latency_seconds` (histogram `le` buckets 1ms..2s)<br/>• `task_wait_time_seconds` (histogram) |

## Steps
1. **Add PAL stub if absent**
```rust
// crates/pal/src/lib.rs
#[derive(Debug, Clone)]
pub enum TaskEvent { Spawned(TaskId), Running(TaskId), Yield(TaskId),
                    Sleep{tid:TaskId,dur:Duration}, IoWait{tid,io:u64},
                    Done(TaskId), Failed{tid,error:String}, Throttled(TaskId) }

pub fn emit(evt: TaskEvent) { tracing::debug!(?evt, "PAL"); }
```
2. **Tracing integration** Define `TaskEvent`** in `scheduler::telemetry`.
    * Add `tracing` to `scheduler/Cargo.toml` if not present (already used).
    * In `Scheduler::spawn*`, emit `TaskEvent::Spawned` and `tracing::info!`.
* In `run()`:

```rust
let span = tracing::span!(tracing::Level::TRACE, "sched.tick", tid);
let _enter = span.enter();
pal::emit(TaskEvent::Running(tid));
```

3.**Metric registration** In `Scheduler::run`:
* Create span before processing each syscall.
* Call `pal::emit(TaskEvent::State { tid, state })`.

```rust
use metrics::{register_gauge, describe_gauge, gauge, ...};

static READY_DEPTH: Lazy<Gauge> = Lazy::new(|| {
   describe_gauge!("scheduler_ready_queue_depth", "Tasks waiting to run");
   register_gauge!("scheduler_ready_queue_depth")
});
// update on every push/pop
```

Histograms via `metrics::histogram!`.
4. **Compute latencies** Update metrics on:
    * `ready.push()` / `pop()`  → gauge
    * Before/after task execution → histograms

    * Record `Instant` at pop; on re-queue compute `run_dur`.
    * For wait histogram: when task first becomes not-ready (sleep/join/iowait)
      store `start_wait` in small hashmap; compute delta on resume.

5. **Add unit-test** (`telemetry.rs`) verifying:
    * PAL receives `Done` event.
    * Gauge depth behaves as expected (use test registry).

## Implementation Steps

5. **Tests**

    * Use `metrics_util::registry::Registry` to capture values.
    * Assert gauge depth rises/falls as expected.
    * Unit test logs: set `RUST_LOG=trace`, capture `tracing_subscriber::fmt`
      output and assert PAL events appear.


## Notes & Edge Cases

* Keep telemetry overhead < 5 µs per transition (no string allocs inside loop).
* Histogram buckets: `[0.001,0.005,0.010,0.050,0.100,0.250,0.5,1,2]` seconds.
* Consider compile-time feature `telemetry` to strip all when disabled.


All existing tests must remain green.
