## 7. `tasks/scheduler/sharded_api.md` *(optional prototype)*


# 🧵 Task: Sharded Executor Prototype

## Context
Future multi-core VMs will need multiple cooperative schedulers with
work-stealing ready queues.

## Acceptance
* New trait `Executor` with `spawn`, `spawn_with_priority`, `run`.
* `ShardPool::new(cores)` spawns `cores` independent `Scheduler`s.
* Struct `ShardPool { schedulers: Vec<Scheduler> }` implements `Executor`.
* Simple round-robin spawn; naive steal on empty ready queue.
* Benchmark placeholder in `benches/shard.rs`.

* Trait:
```rust
pub trait Executor {
  unsafe fn spawn<F>(&self, f: F) -> TaskId
  where F: FnOnce(TaskContext)+Send+'static;
  fn run(&self);
}
```

* Work-stealing: if local ready empty, pop from next scheduler in ring.
* Bench (`benches/shard.rs`) shows linear speed-up on >1 core.

## Steps
1. Move `Scheduler` behind `trait Executor`.
2. Implement `ShardPool::run()` launching each scheduler on its own thread.
3. Work-stealing: if local ready empty, lock another scheduler’s ready pop.


🧵 Task: Sharded Executor Prototype

## Goal
Demonstrate multi-threaded scaling with minimal work-stealing.

---

## Implementation Hints

* Use `crossbeam::deque::{Injector, Steal, Worker}` for global & local queues.
* Use `scope` from `crossbeam` to spawn OS threads.
* Keep telemetry per-shard; aggregate via Prom labels (`shard=X`).
