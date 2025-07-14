# ADR-0028: Compliance, Auditing, and Regulatory Considerations

## Status
Accepted

## Context
Tiffany may be deployed into regulated environments—enterprises, public sector, and AI-driven developer platforms—requiring strong guarantees around:

- Data handling
- Auditability
- Retention policies
- Regulatory standards (e.g., GDPR, CCPA, SOC 2, ISO 27001)

Compliance must be supported without compromising core runtime autonomy, task scheduling, or memory architecture. Logging and retention infrastructure must also not degrade system performance or introduce state leakage across boundaries.

---

## Decision

### 🛡️ Compliance Frameworks Supported

| Standard       | Applicability                         |
|----------------|----------------------------------------|
| **GDPR**       | Data handling, retention, right to forget |
| **CCPA**       | Consumer data access / erasure rights |
| **SOC 2 Type II** | Auditing, traceability, access control |
| **ISO 27001**  | Information security policy alignment |
| **AI Act / NIST RMF** | (Forward-compatible) AI safety & risk mgmt |

---

### 🔍 Auditing Architecture

- **PAL (Process Activity Log)**: real-time, tamper-evident stream of agent and skill activity
- **WAL (Write-Ahead Log)**: deterministic, append-only task ledger
- **Access Logs**: all CLI, UI, and orchestrator interactions are journaled (auth, IP, user ID, method)
- **Agent Memory Traces**: sensitive memory entries tagged for masking or restricted retrieval

All logs include:

- `timestamp (ISO 8601)`
- `actor_id` (user, agent, service)
- `action` (e.g., `task.create`, `skill.invoke`, `agent.shutdown`)
- `resource` (task_id, file, repo, etc.)
- `result` (success/fail, error code, duration)

Logs are streamed via OTEL to:
- Cloud logging providers (Loki, Elasticsearch, CloudWatch)
- Encrypted S3 archive buckets (for regulatory cold storage)
- Optional compliance dashboard interface

---

### ⏳ Retention and Erasure Policy

| Data Type         | Default Retention | Erasure Capability         |
|-------------------|-------------------|----------------------------|
| PAL               | 7 days            | Selective stream deletion  |
| WAL               | 30 days (checkpointed) | Erased via agent scope GC |
| Vector Embeddings | 14 days           | Partial wipe per task ID   |
| Graph Memory      | Persistent        | Versioned + redaction flag |

Deletion is authorized via elevated control plane privilege (`admin.data` scope).

---

### 🧪 Audit Reports and Hooks

- Audit export via `/__compliance/auditlog?from=…&to=…`
- Reconciliation with external SIEM or data lake
- Real-time alerting on:
  - Role violations (e.g. unauthorized `edit:secrets`)
  - Data from unexpected georegions
  - Excessive access patterns (rate anomaly)

---

### ✳️ Trust Boundary Definitions

- Each agent is sandboxed per identity and task
- Memory, vector, and filesystem isolation enforced via namespaces
- No secret or memory entry is ever written to WAL
- All telemetry redacted for PII unless explicitly scoped

---

## Consequences

- Requires hard boundaries between system logs (WAL) and observability logs (PAL)
- Agents must expose structured audit output over gRPC or push to stream
- Teams adopting Tiffany in regulated environments must configure:
  - Retention duration
  - Audit log sink
  - Regional data sovereignty requirements

---

## Related

- ADR-0022: Backup and Disaster Recovery
- ADR-0023: Secrets and Credentials Management
- ADR-0029: Data Retention and Privacy
- ADR-0015: Metrics and Instrumentation
