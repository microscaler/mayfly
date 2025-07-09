### `tasks/scheduler/task_state_expansion.md`

# 🗺️ Task: Expand `TaskState` & Wire Transitions

> **Goal:** Replace the current minimal `TaskState` with a full-fledged
> lifecycle enum (`Scheduled`, `Running`, `Sleeping`, `Waiting`, `Throttled`,
> `Finished`, `Failed`) and make `Scheduler` update it on every transition.
>
> This MUST land **before** Phase-4 instrumentation tasks so spans & PAL events
> reflect accurate states.

---

## Acceptance Criteria

| Check | Requirement |
|-------|-------------|
| ✅ Enum `TaskState` contains **7** variants with required payloads (`wake_at`) |
| ✅ `Task` struct updated (`state`, `consumed`) |
| ✅ All spawn paths set `Scheduled` |
| ✅ `Scheduler::run` transitions state on every `SystemCall` branch |
| ✅ Ready queue only re-queues `Scheduled` or `Running` (after quantum) |
| ✅ All existing tests pass;  **new tests** prove each state transition |
| ✅ `PAL::TaskEvent` carries enum variant; tracing span shows `state=?` |
| ✅ Metric `tasks_state_total{state="sleeping"}` increments correctly |

---

## Implementation Steps

1. **Define new enum**

   ```rust
   #[derive(Copy, Clone, Debug, PartialEq, Eq)]
   pub enum TaskState {
       Scheduled,
       Running,
       Sleeping { wake_at: Instant },
       Waiting,                // I/O or Join
       Throttled,              // CPU / back-pressure pause
       Finished,
       Failed,
   }
```

2. **Extend `Task` struct**

   ```rust
   pub struct Task {
       pub tid: TaskId,
       pub pri: u8,
       pub handle: JoinHandle<()>,
       pub state: TaskState,
       pub consumed: Duration,   // CPU ticks in current quantum
   }
   ```

3. **Spawn paths**

    * In `spawn*`, build `Task { state: TaskState::Scheduled, consumed: 0 }`
    * Emit `pal::emit(TaskEvent::Spawned(tid))`.

4. **Scheduler dispatch**

Before executing coroutine code:

```rust
{
    let t = self.tasks.get_mut(&tid).unwrap();
    t.state = TaskState::Running;
    pal::emit(TaskEvent::Running(tid));
}
```

5. **SystemCall handling**

   | Variant                   | Action                                                         |
         | ------------------------- | -------------------------------------------------------------- |
   | `Sleep(dur)`              | `task.state = Sleeping{wake_at}` and push into `sleepers` heap |
   | `Join(..)` / `IoWait(..)` | `task.state = Waiting`, add to `wait_map`, don’t re-queue      |
   | `Yield`                   | `task.state = Scheduled`, re-queue                             |
   | `Done`                    | `task.state = Finished`, remove from `tasks`, notify waiters   |
   | `Panic` (join error)      | `task.state = Failed`, notify waiters                          |

6. **CPU throttle path**

   When consumed ≥ quantum:

   ```rust
   task.state = TaskState::Throttled;
   self.ready.push(tid); // still same priority
   pal::emit(TaskEvent::Throttled(tid));
   metrics::counter!("task_budget_throttle_total", 1);
   ```

7. **Waiters wake-up**

   When `sleepers` fires or `io_wait` ready:

   ```rust
   let t = self.tasks.get_mut(&tid).unwrap();
   t.state = TaskState::Scheduled;
   self.ready.push(tid);
   ```

8. **Telemetry update**

    * `PAL::TaskEvent` struct gets `state: TaskState`.
    * Metrics: use `metrics::counter!("tasks_state_total", 1, "state" => variant)`.

9. **Tests**

    * `state_sleep_wait.rs` – spawn task that sleeps then done; assert sequence:
      `Scheduled → Running → Sleeping → Scheduled → Finished`.
    * `state_join_wait.rs` – joiner should see `Waiting` → `Scheduled`.
    * Update `yield_order`, `budget_throttle` tests to assert `Throttled`.

10. **Docs**

    * Update `scheduler/README.md` with state-diagram Mermaid:

      ```mermaid
      stateDiagram-v2
        [*] --> Scheduled
        Scheduled --> Running : pop from ready
        Running --> Sleeping : Sleep
        Running --> Waiting : IoWait/Join
        Running --> Throttled : CPU budget
        Throttled --> Scheduled : next tick
        Sleeping --> Scheduled : wake_at reached
        Waiting --> Scheduled : event ready
        Running --> Finished : Done
        Running --> Failed : panic
      ```

---

## Edge-Case Notes & Hints

* **Double push**: When a task yields & then emits `Sleep` quickly, ensure you
  don’t leave duplicate IDs in ready queue (use `ReadyQueue::contains` helper).
* **Cancelled tasks** (Phase-3 cancel) should terminate with `Failed` state.
* **Metrics cardinality** - limit labels to fixed set of state names.
* Minimal overhead: state update is a simple enum write (no heap allocation).

---

After merging, Phase-4 Telemetry and Budget tasks can assume accurate state
labels and richer PAL events.
