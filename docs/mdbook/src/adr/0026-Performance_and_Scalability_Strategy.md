# ADR-0026: Performance and Scalability Strategy

## Status
Accepted

## Context
Tiffany's architecture must support:

- Concurrent multi-agent task execution (within and across nodes)
- Real-time skill invocation and coordination
- Memory graph lookups, WAL replay, and vector retrieval
- High-throughput CI-like workloads and autonomous ReAct loops

Without a clearly defined performance strategy, system bottlenecks could emerge in coroutine scheduling, graph storage, vector DB lookup, and task orchestration.

---

## Decision

### 📈 Target Performance Benchmarks

| Subsystem              | Metric                           | Target                                  |
|------------------------|----------------------------------|-----------------------------------------|
| Task scheduler         | Tasks resumed/sec                | 10,000+ reschedules/sec per agent       |
| WAL write latency      | Avg WAL append latency           | < 2ms                                   |
| Graph memory           | Semantic query (depth ≤ 2)       | < 10ms                                  |
| Vector search          | Cosine similarity lookup (top 5) | < 20ms per query                        |
| ReAct cycle            | Plan→Execute→Observe cycle time  | < 1.5s avg (local), < 3s (with LLM)     |
| Agent startup          | Cold boot to ready               | < 1s including WAL replay               |

---

### 🧱 Performance Foundations

#### 🌀 Coroutine-First Scheduling

- Zero-cost, turn-based scheduler with optional static analysis of yield points
- No thread-pool or preemption overhead; all scheduling is cooperative
- Blocking calls (I/O, joins, sleeps) are explicitly marked with `yield` system calls

#### 🧠 Memory Access Optimizations

- Semantic graph queries precompiled into fast traversal plans
- Vector DB sharded by namespace; warm caches and pinned LRU contexts per agent
- WAL segments memory-mapped and optionally compressed for low-latency replay

#### 💾 WAL + PAL I/O

- WAL uses append-only segment ring buffer with rotational flushing
- PAL logs streamed to async telemetry sink (Kafka, Loki) for non-blocking observability
- WAL replayer supports jump-to-checkpoint + range replay for partial recovery

---

### ⚖️ Scalability Strategy

#### Horizontal

- FAR agents are lightweight and container-isolated (e.g. via Firecracker, Apple Containers)
- Orchestrator distributes load across blades via topology-aware task placement
- VectorDB and GraphDB are horizontally scalable with sharded memory contexts

#### Vertical

- Each agent supports multi-tenant coroutine runtime (N tasks, M skills, K pipelines)
- In-memory task queuing and snapshot compression reduces memory churn

#### Auto-Tuning

- Per-agent schedulers can emit runtime feedback: task starvation, overcommit risk, I/O wait heat
- Orchestrator adjusts backpressure and agent warm pool allocation accordingly

---

### 📉 Proactive Optimization Strategy

- All major system calls (I/O, memory, skill) instrumented with OpenTelemetry spans
- Flamegraph generation in CI for long-running skills
- Periodic benchmark suite: `just perf-test` exercises hot paths
- Slack alerts for 99th percentile tail latencies exceeding thresholds

---

### 🛠️ Reactive Capacity Strategies

- Dynamic task throttling when:
  - WAL queue exceeds burst watermark
  - Memory store rejects graph writes
  - LLM response times exceed circuit breaker thresholds

- Task preemption policies can suspend low-priority plans during overload

---

## Consequences

- Requires standard performance benchmarks per subsystem
- Engineering effort for fine-grained tracing and instrumentation
- Enables high confidence in agent coordination at scale, multi-core
- Design encourages cache-warm, low-latency agents by default

---

## Related

- ADR-0006: WAL Schema and Replay
- ADR-0010: Task Plan Timeline and Execution Metadata
- ADR-0015: Metrics and Instrumentation Plan
- ADR-0030: Cost Management and Budgeting
