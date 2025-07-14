# ADR-0022: Backup, Disaster Recovery, and Failover Strategy

## Status
Accepted

## Context
Tiffany’s autonomous runtime operates as a long-lived coroutine-based execution platform with agent-local state, skill graphs, and persistent semantic memory. Ensuring the reliability and durability of this system across restarts, hardware failures, or service interruptions is critical. This ADR defines the system-wide approach for backup, disaster recovery, and failover—including coverage for WAL (Write-Ahead Log), PAL (Process Activity Log), vector memory embeddings, and graph state.

## Decision

### 🔐 Backup Domains

We define the following subsystems as backup-critical:

- **WAL (Write-Ahead Log)**
  - Immutable, append-only logs of task execution, state transitions, yields, and system calls
  - Required for deterministic replay and recovery

- **PAL (Process Activity Log)**
  - Real-time telemetry of progress events and logs
  - Not durable by default, but critical for observability; streamed to persistent pub/sub sink

- **MemoryStore**
  - Semantic Graph (Neo4j or embedded store)
  - Vector embeddings (e.g. Qdrant, Faiss, or SQLite-based)
  - Required to preserve agent context and avoid memory loss on reboot

- **Agent Skill & Scheduler State**
  - Active task queue snapshot
  - Join/Wait dependencies
  - Sleep/IO wait maps

### 💽 Backup and Retention Policies

| Component         | Frequency   | Retention | Backup Medium       |
|------------------|-------------|-----------|----------------------|
| WAL              | Continuous  | 30 days   | S3 (or MinIO)        |
| PAL Sink         | Streamed    | 7 days    | Kafka + cold archive |
| Graph Store      | Daily       | 14 days   | Dump to S3 as file   |
| Vector Store     | Daily       | 7 days    | Export to S3 bucket  |
| Scheduler Snap   | Hourly      | 3 days    | Snapshot to S3       |

### 🔁 Recovery Flow

```mermaid
sequenceDiagram
    participant Operator
    participant WAL
    participant GraphStore
    participant Scheduler
    participant Agent

    Operator->>Scheduler: Trigger recovery
    Scheduler->>WAL: Load WAL segment
    Scheduler->>GraphStore: Restore graph dump
    Scheduler->>Agent: Restore task queues, state
    Agent-->>Scheduler: Ready
````

### 🧱 Failover Strategy

* **Stateless FAR agents** always resume from latest WAL and graph snapshot
* WAL segments are replayed from last completed checkpoint
* If WAL is corrupted or missing, system will fail fast and request operator intervention
* PAL is not required for recovery but is useful for verifying replay consistency

### 🎯 RTO / RPO Targets

| Metric               | Target       |
| -------------------- | ------------ |
| Recovery Time (RTO)  | < 5 minutes  |
| Recovery Point (RPO) | < 10 seconds |

* WAL checkpoints will be taken every 10s for hot recovery
* Graph dumps are checkpointed and hash-verified for consistency

---

## Consequences

* Introduces need for WAL checkpoint indexing and segment management
* Requires agent restart process to support cold boot and WAL replay
* Graph and vector stores must expose export/import APIs
* PAL must be optionally sinked to a persistent stream processor

---

## Related Documents

* ADR-0006: WAL Schema and Replay Policy
* ADR-0007: Process Activity Log (PAL)
* ADR-0008: Persistent Agent Memory Strategy
