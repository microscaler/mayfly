# ADR-0003: Task Scheduler Model

## Status
Accepted

## Context
Tiffany operates as a coroutine-oriented, agentic runtime designed for reasoning, acting, and tool orchestration. Core to this architecture is a **cooperative task scheduler** responsible for managing:

- Long-running agent workflows
- Step-wise ReAct loops
- Concurrent tool invocations
- Cancelable task execution

As of Rust 2024, first-class coroutines provide an ideal primitive for implementing these requirements with clarity and zero-cost abstraction.

This ADR defines our **task scheduling model**, the task lifecycle states, the yield/resume semantics, and how the system will structure and manage cooperative multitasking.

---

## Decision

We will implement a coroutine-first, cooperative **task scheduler** with the following characteristics:

### 🧵 Task Type
- All tasks will be represented by a unified trait:

```rust
trait AgentTask {
    fn poll(&mut self, ctx: &mut TaskContext) -> TaskState;
}
```

### 📐 Task States
Tasks can exist in the following explicit states:
- `Ready` – enqueued for execution
- `Running` – currently executing
- `Waiting` – yielded for tool response or LLM
- `Completed` – finished with result or error
- `Canceled` – forcibly stopped

### 🔄 Yield/Resume Semantics
Tasks may yield cooperatively during:
- LLM calls
- Tool executions
- User confirmations
- Awaiting subprocess completion

### 🧰 Runtime Loop
The scheduler will:
- Poll each `Ready` task
- Route yielded work (e.g. to LLM executor or tool manager)
- Queue task back when dependency resolves

### 🧭 Goals
- Deterministic, testable behavior
- Serializable/resumable task state
- Decoupled from `tokio::spawn` or native threads
- Pluggable scheduling policy (FIFO, priority, dependency-aware)

---

## Rationale

### 🔁 Cooperative vs Preemptive
Tiffany requires transparent control over task transitions. Preemptive systems (e.g. thread pools) make it difficult to audit agent state or model planning steps. Cooperative coroutines, by contrast, allow us to:
- Yield at semantic boundaries
- Inject logs and metrics at every step
- Serialize/resume entire task graphs

### 🧠 Agent Design Requires Suspended Thought
An agent might:
```rust
let plan = yield plan_with_llm("Build test harness");
for step in plan.steps {
    yield apply_code_diff(step);
    yield confirm_with_user(step);
}
```
This structure is naturally represented with a coroutine and state machine — not an async future.

### 🧪 Testability
We can model agent execution using deterministic stepping:
```rust
let mut scheduler = TestScheduler::new();
scheduler.inject_mock_tool("ls", "result");
scheduler.step_until_idle();
assert_eq!(scheduler.task_state(task_id), TaskState::Completed);
```
This level of control is difficult in actor or spawn-based models.

### 📦 Integration Simplicity
The scheduler serves as glue between:
- LLM router
- Tool executor
- WAL
- Canvas

Having a single poll-loop mediator makes integration simpler and easier to visualize.

---

## Consequences

- Adds internal coroutine scheduler as a first-class subsystem
- Task implementations will need to support resumable `poll()` style execution
- `Executor`, `LLM`, and `Tool` interfaces will interact through message passing / callbacks with the scheduler
- CI tests will include end-to-end scheduling tests using mocked yield points

---

## Alternatives Considered

- **Tokio TaskPool**: Too opaque for agentic step control; no built-in yield
- **Actor Model (e.g., `actix`)**: Good for I/O, but overkill for structured flows
- **`futures`-based step machines**: Verbose, brittle, not coroutine-native

---

## Related Documents
- [Tiffany System Architecture](../whitepapers/Tiffany%20System%20Architecture%20and%20Design%20Overview.md)
- [pyos8.py (Coroutine OS)](http://www.dabeaz.com/coroutines/pyos8.py)
- [ADR-0001: Rust 2024 Coroutine Adoption](adr_0001_rust_version.md)

---

## Adopted
This ADR is accepted as of June 2025. All internal workflows that require suspendable agent behavior will be modeled as `AgentTask`s and scheduled cooperatively.

Maintainers: `@casibbald`, `@microscaler-team`
