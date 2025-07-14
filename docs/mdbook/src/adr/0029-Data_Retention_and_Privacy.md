# ADR-0029: Data Retention and Privacy

## Status
Proposed

## Context
Tiffany’s architecture involves persistent agent memory (graph + vector), detailed execution logs (WAL, PAL), and potential handling of sensitive user data (e.g., source code, commit authors, system metadata).

To maintain trust, legal compliance, and operational hygiene, we must clearly define:

- What data is retained
- For how long
- Under what controls
- And how data deletion, redaction, or masking are handled in practice

This ADR formalizes retention tiers and provides guarantees that Tiffany can operate in privacy-conscious or regulated environments.

---

## Decision

### 🗃️ Data Types and Retention Classes

| Data Type            | Category         | Default Retention | Erasable | Notes                                       |
|----------------------|------------------|-------------------|----------|---------------------------------------------|
| WAL segments         | Operational log  | 30 days           | ✅        | Checkpointed, GC’d after task completion     |
| PAL streams          | Observability    | 7 days            | ✅        | Ephemeral; redacted on export               |
| Memory graph (Neo4j) | Agent memory     | Persistent        | ⚠️ Partial | Versioned; requires explicit redaction API  |
| Vector embeddings    | Semantic memory  | 14 days           | ✅        | Per-task TTL + ID-based erasure             |
| Config files         | Static config    | Infinite          | ✅        | Controlled via Git, traceable to commits    |
| Task plans/comments  | Runtime metadata | 14 days           | ✅        | Redacted upon request                       |
| Secrets              | Ephemeral        | ≤ 1 hour          | ✅        | Enforced via SecretProvider TTL             |

---

### 🔐 Personally Identifiable Information (PII)

All potential PII must be:

- Tagged at ingestion
- Redacted or masked at rest (if configured)
- Excluded from WAL and PAL by default unless explicitly authorized

**Examples of PII-like data:**
- Developer usernames, emails (from Git commits)
- API keys, tokens, URIs
- Repository paths or SSH hosts
- Stack traces with hostnames or kernel logs

---

### 🧪 Erasure API

A standard gRPC interface will be exposed:

```protobuf
rpc EraseData(EraseRequest) returns (EraseResponse)
message EraseRequest {
  string task_id
  string agent_id
  repeated string data_types = ["vector", "graph", "pal"]
  bool full_wipe = false
}
````

Tasks or memory entries can be erased individually or collectively. Redacted entries are retained as tombstones for observability unless `full_wipe = true`.

All erasures are:

* Logged to PAL as a `data.erased` event
* Recorded in a WAL checkpoint tombstone marker

---

### 🧭 Governance and Policy Controls

* Default retention values are configurable per deployment

* Redaction can be triggered by:

    * User request (via CLI/UI)
    * Admin policy script (e.g. nightly TTL GC)
    * Compliance webhook (e.g. `delete_user_data` API from UI)

* Data will not be replicated across regions unless explicitly marked as public

* Multi-tenant deployments must enforce namespace-scoped storage

---

## Consequences

* Requires memory and vector stores to support TTL, deletion, and redaction natively
* Adds overhead for audit logging of data lifecycle events
* Enables deployers to satisfy GDPR/CCPA “right to forget” and internal cleanup policies

---

## Related

* ADR-0028: Compliance and Auditing
* ADR-0008: Persistent Agent Memory Strategy
* ADR-0023: Secrets Management

