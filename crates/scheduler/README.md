# Tiffany Scheduler

The `scheduler` crate implements Tiffany’s coroutine-based cooperative task scheduler.

> ✅ This crate is a **minimal MVP** that focuses on building and testing the fundamental task management loop — before any agentic cognition is added.

---

## Channel model

- **Syscall channel:** Tasks (may coroutines) send `(TaskId, SystemCall)` on an unbounded crossbeam channel; a single **run loop** receives and updates scheduler state. This is **MPSC** (multi-producer, single-consumer).
- **Same-thread rule:** May coroutines run only on the thread that called `spawn`. The run loop and all task spawning must therefore run on the **same thread**, or use the **spawn-request API** so that a dedicated scheduler thread owns spawning (see [Scheduler-thread-owned spawning](#scheduler-thread-owned-spawning) below).
- **MPMC option:** Multiple consumer threads (cloning the syscall `Receiver`) are possible with shared or partitioned scheduler state; see the [design doc](../../../docs/design-mayfly-channels-mpmc.md) for options and trade-offs. Current implementation is single-consumer.

### Scheduler-thread-owned spawning

To run the scheduler on a **dedicated thread** without deadlock, that thread must be the one that calls `spawn` (so coroutines run on it). Use the **spawn-request API**:

1. Create a spawn-request channel: `let (spawn_tx, spawn_rx) = unbounded()`.
2. Start a thread that owns the `Scheduler` and calls `sched.run_with_spawn_requests(spawn_rx)`.
3. From other threads, send `SpawnRequest` values via `spawn_tx`; each request runs a closure that receives `&mut Scheduler` and may call `spawn`, `spawn_with_priority`, etc. Optionally include a reply sender to get the new `TaskId` back.

See `Scheduler::run_with_spawn_requests` and `SpawnRequest` in the API docs.

---

## 🎯 MVP Objective

The scheduler should:

- Support creating and running **multiple tasks** concurrently
- Support yielding, blocking, and resuming tasks cooperatively
- Manage tasks via:
  - Priority-based ready queue
  - Per-task LIFO call stack (for nested coroutines)
- Support blocking system calls like `Sleep`, `WaitTask`, `ReadWait`
- Be fully testable via deterministic simulations
- Implement a strict test harness that verifies lifecycle behavior, blocking/wakeup, and call-stack trampolining

---

## 🌀 Core Concepts

| Concept         | Description                                                   |
|-----------------|---------------------------------------------------------------|
| `Task`          | A generator-style coroutine that can yield system calls       |
| `Scheduler`     | Manages task queue, blocking map, and the main loop           |
| `SystemCall`    | An abstract yield, e.g., `Sleep`, `Spawn`, `Join`, `Log`, `Yield` |
| `ReadyQueue`    | Priority queue of runnable tasks built on `BinaryHeap<ReadyEntry>` |
| `CallStack`     | LIFO per-task stack for nested coroutine trampolining |
| `WaitMap`       | Tracks join/wait conditions for resumption |
| `TaskState`     | Lifecycle status: `Running`, `Finished`, or `Failed` |
| `ready_len()`   | Inspect number of tasks currently queued |

---

## 🔍 Example Flow

```rust
let mut sched = Scheduler::new();
let tid = sched.spawn(echo_loop());
sched.run();
````

Higher priority tasks can be spawned with `spawn_with_priority`:

```rust
let tid = unsafe { sched.spawn_with_priority(5, my_task) };
```

Inside `echo_loop`, you might yield:

```rust
yield SystemCall::Sleep(Duration::from_secs(1));
```

The task is then moved to a timed wait queue, and resumed later by the scheduler.
When no tasks are ready, a virtual `TickClock` advances instantly so that
`Sleep` durations never block the thread.

Tasks automatically yield back to the scheduler after each call to `TaskContext::syscall`,
ensuring cooperative execution across all running tasks. When a task simply
wants to give up the CPU without performing a specific syscall it can call
`TaskContext::yield_now()`, which requeues the task behind others.

---

### Scheduler Loop

`Scheduler::run` processes events in a fixed order each iteration:

1. **Drain system calls** – all pending `SystemCall`s are handled first. This may
   wake tasks waiting on joins or I/O and records any completed tasks.
2. **Apply I/O completions** – any ready I/O events are drained and their waiting
   tasks queued.
3. **Pull next task** – a task ID is popped from the ready queue. If the queue is
   empty the scheduler waits up to five seconds for an I/O event before
   returning.

This guarantees that tasks unblocked by system calls resume promptly before the
next ready task is polled.

---

### Scheduler Loop

`Scheduler::run` processes events in a fixed order each iteration:

1. **Drain system calls** – all pending `SystemCall`s are handled first. This may
   wake tasks waiting on joins or I/O and records any completed tasks.
2. **Apply I/O completions** – any ready I/O events are drained and their waiting
   tasks queued.
3. **Pull next task** – a task ID is popped from the ready queue. If the queue is
   empty the scheduler waits up to five seconds for an I/O event before
   returning.

This guarantees that tasks unblocked by system calls resume promptly before the
next ready task is polled.

---

### 🔮 Coroutine Implementation Guidance

While the initial MVP of the scheduler may use a simple `Box<dyn Generator<Yield = SystemCall, Return = ()>>` model for tasks, contributors are encouraged to evaluate **long-term strategies** based on two possible coroutine models in Rust:

#### 1. ✅ [May](https://github.com/Xudong-Huang/may) Coroutine Crate

- Mature userland stackful coroutine runtime
- No `Send + Sync` required
- Deterministic, good for local multitasking
- Pros: lightweight, proven, battle-tested
- Cons: Requires external runtime, less idiomatic with `async` ecosystems

#### 2. 🧪 Native Rust 2024 `generators`

- Stabilizing `Generator` trait in `std` as part of coroutine roadmap
- Uses built-in `yield` keyword
- Integrates well with Rust’s future `gen` support and async schedulers
- Pros: native, no external deps, future-proof
- Cons: still under stabilization in parts of the ecosystem

---

### 💡 Guiding Principle

The scheduler crate must:

- Be **agnostic to coroutine engine** (via trait or `dyn Task` abstraction)
- Document internal coroutine usage clearly
- Provide a toggle or feature flag (e.g., `--features native-gen`) if dual support emerges


---

## 🧪 Testing Framework

All tests run **deterministically**. Features under test:


* Task resumption and execution order
* Join dependencies
* Stack-based subcoroutines
* Time-based blocking (`Sleep`)
* I/O wait (`IoWait`)
* Manual signal triggering to unblock tasks

Use:

```bash
cargo test -p scheduler
cargo test -p scheduler --features async-io
```

---

## ✅ MVP Checklist

* [x] Task spawning and ID tracking
* [x] FIFO ready queue
* [x] LIFO call stack with trampolining
* [x] Join/wait and requeueing
* [x] Sleep system call (with tick clock or mock timer)
* [x] Test suite for task blocking/resume logic
* [x] I/O wait
* [ ] Signal/interrupt handling

---

## 🔧 Directory Layout

```txt
crates/scheduler/
├── src/
│   ├── lib.rs             # Scheduler entry point
│   ├── task.rs            # Task struct + state
│   ├── syscall.rs         # SystemCall enum and types
│   ├── ready_queue.rs     # FIFO queue
│   ├── call_stack.rs      # LIFO stack for sub-coroutines
│   ├── wait_map.rs        # Task wait conditions (join, sleep)
│   └── tests/             # Full lifecycle + system call tests
```

---

## 🧱 Non-Functional Requirements

| Characteristic | Implementation Strategy                       |
| -------------- | --------------------------------------------- |
| Deterministic  | Tasks yield values; no threads, no preemption |
| Observable     | All transitions may be traced to logs/PAL     |
| Isolated       | Tasks are logically sandboxed                 |
| Portable       | No OS dependencies; purely Rust coroutines    |
| Replayable     | Future: WAL hook for recording yields         |

---
### 🛣️ Phase 3 - Scheduler Roadmap 

*(moving from “it works” ⇒ “production-ready, PyOS8-inspired”)*

| Phase | Focus Area                    | Why It Matters                                                                                                                        | Concrete Next Tasks                                                                                                                                                                    |
| ----- | ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **1** | **True cooperative stepping** | Right now we drive purely from blocking `recv_timeout`. PyOS8 yields back to the scheduler **every time the coroutine uses `yield`**. | • Refactor `TaskContext` to expose an explicit `yield_now()`<br>• Replace `std::thread::sleep` in `SystemCall::Sleep` with inserting a `(wake_at, tid)` into a timed wheel / min-heap. |
| **2** | **TickClock & virtual time**  | Deterministic tests + future timeouts without wall-clock sleeps.                                                                      | • Wire `TickClock` into `Scheduler` (inject during tests).<br>• Create a `sleep_heap: BinaryHeap<(Instant, TaskId)>`.<br>• Add `tick()` path to pop ready sleepers.                    |
| **3** | **I/O poll abstraction**      | Real async runtime needs readiness events (Fd / network).                                                                             | • Introduce trait `IoSource { fn raw_fd(&self) -> RawFd; fn id(&self) -> u64 }`.<br>• Map `SystemCall::IoWait` to epoll/kqueue (behind feature).                                                               |
| **4** | **Cancellation & timeouts**   | PyOS8 supports cancelling tasks via “kill task id”.                                                                                   | • Add `SystemCall::Cancel(target)`. <br>• Store `cancelled: HashSet<TaskId>` and drop tasks gracefully.                                                                                |
| **5** | **Priority & fairness**       | Some agents (e.g., WAL flusher) must pre-empt heavy compute.                                                                          | • Replace FIFO `ReadyQueue` with (`priority, tid`) binary-heap.<br>• Expose `spawn_with_priority(priority, f)` API.                                                                    |
| **6** | **Error & panic isolation**   | Panicking inside a coroutine shouldn’t crash scheduler.                                                                               | • Wrap `handle.join()` in `catch_unwind`; emit PAL entry.<br>• Bubble failure to any `Join` waiters with an error code.                                                                |
| **7** | **Supervisor tasks**          | PyOS8 supervises children; Tiffany needs same for WAL/PAL.                                                                            | • Add “system” tasks started at boot (metrics flush, GC).                                                                                                                              |
| **8** | **Instrumentation**           | We’ll plug metric spans + event hooks into PAL & Prometheus.                                                                          | • Emit `tracing` span per `run()` cycle.<br>• Counter: `scheduler_ready_queue_depth`.<br>• Histogram: task run-to-completion latency.                                                  |

---

## 🛣️ Scheduler Roadmap — **Phase 4 & 5**

*(“production-hard” + multi-agent scale)*

> You now have PyOS8-level parity: cooperative yield, virtual clock, real I/O, cancellation, priority, panic isolation.
> Next we harden for Tiffany’s real workload: dozens of in-VM agents, thousands of tasks, strict observability, and resource guarantees.

| Phase  | Theme                                 | Why It Matters                                                                                                                      | Key Work-Items (Codex file names)                                                                                                                                                                                                                                                                                     |
| ------ | ------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **4A** | **Instrumentation & PAL/WAL hooks**   | Tiffany’s Process-Activity-Log and metrics dashboards depend on per-task spans and outcome events.                                  | `tasks/scheduler/telemetry_hooks.md`<br>• Insert `tracing::span!(scheduler.loop)` around each step.<br>• Emit `TaskEvent::{Yield, Wake, Sleep, IoWait, Done, Panic}` to PAL via `pal::emit()`.<br>• Expose Prometheus gauges: `scheduler_ready_depth`, `tasks_running_total`, histograms for run latency & wait time. |
| **4B** | **Supervisor / system tasks**         | WAL flusher, metrics pusher, GC sweeper must always run even under load.                                                            | `tasks/scheduler/system_tasks.md`<br>• Reserve priority 0 for “system”.<br>• Add `spawn_system()` used by runtime boot.                                                                                                                                                                                               |
| **4C** | **CPU quota & runtime budgets**       | Prevent runaway compute loops inside a VM; enforce fairness.                                                                        | `tasks/scheduler/cpu_budget.md`<br>• Track “ticks consumed” per task.<br>• If budget exceeded, re-queue and decrement remaining quantum.<br>• Metric `task_budget_throttle_total`.                                                                                                                                    |
| **4D** | **Memory guard-rails**                | Even within a microVM we don’t want OOM to kill the whole agent.                                                                    | `tasks/scheduler/mem_guard.md`<br>• Optional compile-time feature: allocate coroutines from a bounded slab; fail `spawn` if exhausted.<br>• PAL event on `SpawnDenied`.                                                                                                                                               |
| **4E** | **Graceful shutdown & snapshot**      | FAR VM gets `SIGTERM` when orchestrator suspends it. Scheduler must drain tasks, flush WAL, and checkpoint memory for resurrection. | `tasks/scheduler/graceful_shutdown.md`<br>• `Scheduler::shutdown()` sets `terminating=true`, rejects new tasks, waits for drain or timeout.<br>• Emit `PAL::ShutdownBegin`, `PAL::ShutdownComplete`.                                                                                                                  |
| **4F** | **Cross-VM back-pressure (optional)** | When many VMs push work to shared storage, need a flow-control hook.                                                                | `tasks/scheduler/backpressure_hook.md`<br>• Expose callback trait `OnBackpressure` that WAL layer can trigger; scheduler pauses IO-heavy tasks.                                                                                                                                                                       |
| **5**  | **Multi-core / sharded sched**        | Later we’ll run multi-threaded inside VM or across NUMA cores. Stage the API now.                                                   | `tasks/scheduler/sharded_api.md`<br>• Abstract `Scheduler` behind trait `Executor`.<br>• Prototype `ShardPool { schedulers: Vec<Scheduler> }` with work-stealing ready-queues.                                                                                                                                        |

---


1. **Telemetry & PAL Hooks**
   `tasks/scheduler/telemetry_hooks.md` (span wiring + PAL events + Prom metrics)
2. **System Tasks**
   `tasks/scheduler/system_tasks.md` (priority 0, boot tasks, tests)
3. **CPU Budgeting**
   `tasks/scheduler/cpu_budget.md` (tick accounting + throttle test)
4. **Memory Guard**
   `tasks/scheduler/mem_guard.md` (slab allocator wrapper, failure path)
5. **Graceful Shutdown**
   `tasks/scheduler/graceful_shutdown.md` (SIGTERM handler → drain)
6. (Optional) Back-pressure hooks
7. (Optional) Sharded executor prototype

> **Reality check**: Phases 4A-4D are “must-have” for a stable pilot.
> 4E becomes critical once FAR orchestration goes live.
> 4F/5 can trail slightly; they’re optimization/scale work.

No sugar-coating: this is the grind-phase where correctness, telemetry, and safety drown out flashy features — but it’s what turns a PoC into a production micro-kernel for autonomous agents.


---

### 📚 Reference to David Beazley’s PyOS8 Features to Mirror

| PyOS8 Concept                                           | Tiffany Parity               |
| ------------------------------------------------------- | ---------------------------- |
| `yield` returns a syscall tuple                         | our `ctx.syscall(...)`       |
| Scheduler maintains *ready*, *sleeping*, *waiting* maps | already present, will expand |
| `select` loop for I/O readiness                         | planned in Phase 3           |
| Task cancellation via exception injection               | Phase 4                      |
| Timers advancing virtual clock                          | Phase 2                      |

---

With these phases complete, Tiffany’s scheduler moves from **PoC** to a **tiny cooperative micro-kernel** capable of orchestrating thousands of lightweight in-VM tasks deterministically and observably.


---
## 🧩 Future Integrations

Once the scheduler is hardened and verified:

* Add `ReasonAct` agent loop on top
* Connect `executor` for skill invocation
* Begin streaming WAL/PAL logs for tracing

---

## Related Crates

* [`core`](../core) – runtime primitives and ID generators
* [`wal`](../wal) – planned replay integration
* [`daemon`](../daemon) – starts and owns the scheduler lifecycle

---

## 🚀 Goal

This MVP exists to **prove the core scheduler model** is robust and composable.

Until this crate has 100% test coverage on all coroutine lifecycles and blocking behaviors, no agent cognition (`reasonact`) will be integrated.

