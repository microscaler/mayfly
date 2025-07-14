# ADR-0001: Choice of Rust Version

## Status
Accepted

## Context
The Tiffany project is a long-running agentic runtime that coordinates reasoning, code generation, tool usage, and durable memory across asynchronous and cooperative tasks. As we design and implement a sophisticated internal scheduler, coroutine-style agents, and multi-modal tool pipelines, the choice of Rust version is foundational.

Rust 2024 introduces first-class **coroutines**, a long-requested language feature that enables cooperative multitasking and stateful, yieldable computations — precisely the kind of behavior required by intelligent agent runtimes like Tiffany.

Choosing a stable, long-term edition early is essential for architectural consistency and contributor alignment. This ADR proposes that we standardize the entire codebase on **Rust 2024 edition** and adopt its coroutine model as the preferred approach to all internal cooperative task flow and agent loop execution.

---

## Decision

We will use:
- `edition = "2024"` in all crates in the Tiffany workspace.
- Coroutine support as the primary concurrency model for:
    - Task scheduling
    - Agent step planning and reasoning
    - Middleware pipelines
    - ReAct loops (reason/act/yield)

We will gradually phase out any simulated generator patterns or excessive async-future machinery in favor of the idiomatic 2024 coroutine model.

All crates should be compatible with the 2024 edition compiler. Contributors should use the latest stable or nightly toolchains as required to access coroutine syntax and tooling.

---

## Rationale

### ✨ Coroutines Are Native to Tiffany's Design
Tiffany is built around the idea of long-lived agents that yield control during:
- LLM interactions (awaiting model output)
- Tool execution (blocking shell commands or file I/O)
- Multi-turn planning and execution

Coroutines allow these behaviors to be modeled as **linear, stateful programs** that yield control naturally, rather than forcing users to re-encode intent across futures, callbacks, or deeply nested `async fn` chains.

### 🧠 Clearer Agent Logic
Coroutines allow agent plans and task loops to resemble natural, readable sequences:
```rust
coroutine! {
  log("Starting task");
  let plan = yield plan_with_llm("Refactor code");
  for step in plan.steps {
    yield execute_tool(step);
  }
  log("Task completed");
}
```
This model is easier to test, simulate, debug, and understand than a tangled web of futures.

### ⚙️ Aligns with Agentic Runtime Goals
Coroutines are the best tool to model the behavior of an autonomous agent runtime:
- Long-lived processes
- Internal suspend/resume
- Cooperative scheduling
- Parallel agents, each with their own state machine

Tiffany already leans heavily toward micro-OS patterns (see `pyos8.py` inspiration). Coroutines complete the loop.

### 🛠️ Improves Maintainability
By standardizing early on 2024 edition and coroutines:
- All contributors learn one mental model
- Tooling (docs, examples, error messages) is consistent
- Reduces churn from future refactors (async-to-coroutine migrations)
- Better ergonomics when building nested workflows (LLM + tools + commits + retry loops)

### 📈 Forward-Compatible
The Rust ecosystem will increasingly shift to coroutine-based runtimes (especially for agents, actors, and workflows). This choice aligns with where the community is headed.

---

## Consequences

- Requires contributors to use Rust 1.77+ or whichever version first stably supports `edition = "2024"` with coroutine support.
- Initial ramp-up as we define coroutine idioms and patterns for our use case.
- Enables future coroutine-native extensions like:
    - Coroutine-aware task scheduler
    - Coroutine-first agent APIs
    - Nested or multi-agent flows

---

## Alternatives Considered

- **Stick to async/await**: Works for I/O, but is verbose and inflexible for agentic control flow. Harder to compose and introspect than coroutines.
- **Use external coroutine libraries (`may`, `generator`)**: Adds friction, diverges from language-native syntax, and requires users to learn nonstandard constructs.
- **Simulate state with futures or actors**: Complex, harder to test, and introduces boilerplate that Rust 2024 makes obsolete.

---

## Related Documents
- [Tiffany System Architecture and Design Overview](../whitepapers/Tiffany%20System%20Architecture%20and%20Design%20Overview.md)
- [Task Scheduler Design Notes](../whitepapers/task_scheduler.md)
- [pyos8.py – Micro-OS inspiration](http://www.dabeaz.com/coroutines/pyos8.py)

---

## Adopted
This ADR is accepted as of June 2025. All new code contributions should conform to `edition = "2024"` and use coroutine models when implementing long-running tasks or agent loops.

Maintainers: `@casibbald`, `@microscaler-team