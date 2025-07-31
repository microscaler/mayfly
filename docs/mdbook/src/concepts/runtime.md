# Tiffany Runtime Model

Tiffany is a coroutine-first agentic runtime built around the idea of deterministic, traceable, yield-driven execution for autonomous system tasks.

---

## Execution Model

- **Coroutine Scheduler**: Inspired by David Beazley’s `pyos8`, all tasks are cooperative generators. There are no threads, only scheduled yields.
- **System Calls**: Tasks yield events like `ReadWait`, `Sleep`, or `SpawnTask` to the scheduler.
- **Trampolining**: Nested coroutines are managed via a LIFO stack to support function-like composition.
- **Replayability**: Tasks emit `WAL` logs with every yield, enabling complete deterministic replay.
- **Telemetry**: Progress is simultaneously emitted to the `PAL` stream for real-time observability.

---

## Agents and Isolation

Each agent:

- Operates in a self-contained environment (microVM or Apple Container)
- Loads semantic memory (GraphDB + VectorDB)
- Executes plans via the ReAct loop
- Coordinates with orchestrator over mTLS/gRPC

---

## Diagrams

You can find system-level sequence diagrams in the [Diagrams](../diagrams/index.md) section.
