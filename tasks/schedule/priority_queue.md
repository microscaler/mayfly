
# ⏫ Task: Priority-Aware Ready Queue

## Goal
Implement fair scheduling & high-priority tasks (WAL flusher, metrics).
Replace FIFO `ReadyQueue` with `BinaryHeap<(priority, seq, tid)>`.

## Acceptance
* New API:

  ```rust
  unsafe fn spawn_with_priority<F>(&mut self, pri: u8, f: F) -> TaskId;
  ```

* `0 = highest`, `10 = default`, `255 = lowest`.

- Existing `spawn()` delegates to priority 10.

- Add test `priority.rs` verifying order: high priority task finishes before low.

---

### Steps

1. **Replace `VecDeque` with `BinaryHeap<ReadyEntry>`** where:

   ```rust
   struct ReadyEntry { pri: u8, seq: u64, tid: TaskId }
   impl Ord/Eq for reverse ordering
   ```

2. **Sequence counter** to maintain FIFO within same priority.

3. Adapt pop/push helpers.
