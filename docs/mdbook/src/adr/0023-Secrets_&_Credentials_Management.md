# ADR-0023: Secrets and Credentials Management

## Status
Accepted

## Context
Tiffany's distributed runtime interacts with source control (e.g. GitHub), container registries, external APIs, and internal tools — all of which require secure credential handling. Furthermore, FAR agents may dynamically receive, inject, or rotate secrets, especially when performing system calls, provisioning resources, or accessing private tools.

Security principles such as least privilege, explicit scoping, and strong auditing are essential to avoid lateral escalation or leakage of secrets within the runtime.

## Decision

### 🔐 Secret Types and Scope

| Secret Type            | Use Case                                    | Scope          |
|------------------------|---------------------------------------------|----------------|
| GitHub Token           | Cloning repos, PR creation                  | Task           |
| Container Registry Key | Pulling/pushing images                      | Agent / Tool   |
| API Key (3rd Party)    | External service calls (e.g. Slack)         | Skill / Task   |
| FAR Agent JWT / Cert   | Agent authentication to orchestrator        | Node / Agent   |
| Internal Tool Tokens   | Secure tool usage (e.g. SecretsManager)     | Task / Skill   |

### 🔐 Secrets Management Components

- **Secret Abstraction Layer**
  - Standard Rust trait: `SecretProvider` with methods like `get_secret`, `list_secrets`, `refresh_secret`
  - Allows pluggable backends (Vault, Kubernetes Secrets, GCP Secret Manager, local `.env`, etc.)

- **Supported Backends (pluggable):**
  - HashiCorp Vault (primary reference)
  - Kubernetes Secrets (agent-native)
  - AWS Secrets Manager
  - GCP Secret Manager
  - Local `.env` fallback for development

- **Agent-Side Caching and Injection**
  - FARs never persist secrets to disk
  - Secrets are injected per-task into coroutine context and cleared after use
  - Secure memory wipe policy for expired or completed task secrets

### 🔐 Secret Access Flow

```mermaid
sequenceDiagram
    participant FAR
    participant Scheduler
    participant Vault as SecretProvider
    participant Skill

    Scheduler->>FAR: Dispatch task with secret ref
    FAR->>Vault: get_secret("github-token")
    Vault-->>FAR: secret value (scoped TTL)
    FAR->>Skill: inject secret into task context
    Skill-->>FAR: uses secret in API call
    FAR->>Vault: revoke or expire secret (optional)
````

### 🔐 Rotation and Expiry

* All secrets are versioned
* Secrets must support TTL and rotation (with schedule-based or on-demand triggers)
* Expired secrets are removed from memory context and revoked in the backing provider if supported

### 🔍 Audit and Traceability

* All secret fetch/injection events logged to PAL
* Secrets are *not* written to WAL
* Each access has:

    * `task_id`
    * `agent_id`
    * `secret_name`
    * timestamp + duration

### ⚙️ Runtime Controls

* Secrets are injected via coroutine `TaskContext`
* Accessing secrets outside allowed scope (e.g. wrong task or agent) results in panic and termination
* Secrets are never exposed via logs, UI, or PAL payloads (only references)

---

## Consequences

* Requires `SecretProvider` abstraction to be implemented early in runtime setup
* Secret rotation and TTL enforcement must be centrally coordinated (either via orchestrator or via scheduler hooks)
* Adds memory management responsibilities for ephemeral secrets in agents
* All contributors must follow secret redaction in logs and test fixtures

---

## Related

* ADR-0024: Authentication and Authorization
* ADR-0007: PAL (Process Activity Log)
* ADR-0005: Virtual Canvas Git Strategy


