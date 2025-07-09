### 6️⃣ `tasks/scheduler/backpressure_hook.md` (optional)


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
