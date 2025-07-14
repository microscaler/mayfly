# ADR-0030: Cost Management and Budgeting

## Status
Accepted

## Context
Tiffany is designed to run as a horizontally scalable, distributed agentic system. It may operate across:

- On-premises compute nodes (e.g. bare metal blades)
- Cloud environments (GCP, AWS, Azure)
- Mixed clusters with varying workloads

Without explicit cost controls, long-lived agents, large memory graphs, and LLM interactions could cause unpredictable compute and storage costs. This ADR establishes policies for real-time cost monitoring, budgeting limits, alerting, and long-term resource planning.

---

## Decision

### 💰 Cost Dimensions

| Category             | Description                                 | Metered By                    |
|----------------------|---------------------------------------------|--------------------------------|
| Compute (FAR agents) | CPU, RAM, container time                    | Node metrics / k8s limits     |
| Storage (WAL, Graph) | WAL segments, graph db, vector db          | S3, volume usage              |
| Network              | Inter-agent comms, LLM calls, syncs        | Outbound GB                   |
| LLM / SaaS usage     | API calls to OpenAI, Anthropic, etc.       | Per-token or per-call pricing|
| Logging / Observability | PAL/WAL streams, Grafana, Prometheus    | Log volume, query freq        |

---

### 🧭 Budget Enforcement Strategy

- **Per-namespace quotas** for:
  - Number of concurrently running FAR agents
  - Max WAL size
  - Memory graph object count
  - Vector DB footprint

- **Quota policy backend** at orchestrator layer
  - Rejects task plan execution if limits exceeded
  - Annotates tasks with `budget_violation = true` if soft threshold breached

---

### 📊 Monitoring and Tooling

- Prometheus metrics exposed per-agent:
  - `tinkerbell_agent_cpu_seconds_total`
  - `tinkerbell_agent_mem_bytes`
  - `tinkerbell_wal_bytes_written_total`
  - `tinkerbell_vector_index_size_bytes`
  - `tinkerbell_llm_token_usage_total`

- Central cost dashboard:
  - Displays per-agent and per-namespace cost
  - Forecast burn rate based on current usage
  - Alerts on:
    - WAL growth beyond historical baseline
    - LLM token spikes (>2x hourly average)
    - Idle agents over CPU/mem thresholds

---

### 🔔 Alerting Policy

| Alert Condition                 | Action                                  |
|--------------------------------|------------------------------------------|
| WAL exceeds 80% quota          | Annotate task, emit warning to PAL      |
| LLM spend exceeds budget cap   | Throttle future calls, alert operator   |
| Idle agent > 10m at 100% CPU   | Suggest scale-down or forced suspend    |
| PAL output > 5GB/hr            | Recommend retention reduction           |

Alerts are sent to:
- Slack / Discord webhooks
- CLI (`just budget-status`)
- UI budget panel

---

### 💵 Cost Control Features

- `cost_scope` annotation on tasks and plans
- LLM usage budgeting per skill or per namespace
- Graph memory size capping via pruning policies
- WAL compression + offload to cold storage after X days
- PAL TTL auto-GC enabled by default

---

## Consequences

- Adds enforcement logic at orchestrator level
- Increases agent responsibility for exposing real-time metrics
- Enables proactive cost forecasting and caps for cloud-hosted installations
- Supports transparent multi-tenant environments

---

## Related

- ADR-0026: Performance and Scalability Strategy
- ADR-0022: WAL and Backup Storage Policy
- ADR-0015: Metrics and Instrumentation
- ADR-0029: Data Retention and Privacy
