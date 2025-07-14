# ADR-0024: Authentication and Authorization

## Status
Accepted

## Context
Tiffany’s distributed runtime, including the orchestrator, local FAR agents, and external API clients (such as developer UIs or automated tools), must be protected through a secure and auditable authentication and authorization system. The system must support:

- Autonomous agents authenticating to central systems
- Developer CLI and UI access with role separation
- Internal service-to-service trust establishment
- Auditability and fine-grained permission enforcement

We require a flexible design that supports open standards (e.g., OIDC, mTLS) and is compatible with cloud-native environments.

---

## Decision

### 🛂 Authentication Types

| Entity         | Method               | Provider                        |
|----------------|----------------------|----------------------------------|
| Developers     | OAuth2/OIDC          | GitHub, Google, Custom Identity |
| FAR Agents     | mTLS + Signed Claims | Agent CA (Orchestrator Root)    |
| UIs / Services | JWT Bearer Tokens    | OIDC or Service Token Issuers   |
| CLI Users      | OIDC Token Flow      | Auth0, GitHub, or self-hosted    |

- Tokens must be short-lived and refreshable
- All tokens must be cryptographically verifiable (JWTs or cert chains)

---

### 🔐 Authorization Model

We adopt **Role-Based Access Control (RBAC)** with optional future extension to ABAC.

#### Roles (examples):
- `agent.worker`
- `agent.coordinator`
- `user.viewer`
- `user.editor`
- `admin.system`

#### Permission Scopes:
- `read:task`, `write:task`
- `read:agent`, `control:agent`
- `manage:secrets`, `view:logs`
- `edit:canvas`, `approve:plan`

Authorization decisions will be enforced at:

- The orchestrator (for cross-agent communication)
- The API gateway / CLI entrypoint
- The FAR agent task executor (on local secret or skill usage)

---

### 🔑 Token Strategy

- Tokens are passed via HTTP headers or mutual TLS certs
- Tokens must carry:
  - `sub` (subject ID)
  - `exp`, `iat`, and `nbf`
  - `scope` or `roles` claim
  - `agent_id` or `user_id` for audit tracing

Tokens are verified using JWKS from trusted issuers or CA bundles.

---

### 🔍 Auditing

All authentication events and authorization decisions are logged in the PAL:

- Subject ID
- Role / scopes
- Resource accessed
- Action (allow/deny)
- Timestamp

WAL logs may contain references to auth events via correlation IDs but never sensitive tokens.

---

### 🧱 Trust Boundaries

```mermaid
flowchart TD
    User[Dev CLI / UI] -->|OIDC Login| AuthProvider
    AuthProvider -->|JWT| Gateway
    Gateway --> Orchestrator
    Orchestrator -->|mTLS| Agent1
    Orchestrator -->|mTLS| Agent2

    subgraph Agent1 [FAR Agent]
        Executor
        SecretProvider
    end

    Executor --> SecretProvider
    Executor -->|AuthZ check| ACLDB[(Policy Engine)]
````

---

## Consequences

* Requires deployment of trusted OIDC provider or identity federation
* Agent certificates must be provisioned securely (e.g., SPIFFE, bootstrap token flow)
* Policy enforcement libraries must be present in orchestrator and agent
* Contributing developers must follow access scope definitions strictly in code

---

## Related

* ADR-0023: Secrets and Credentials Management
* ADR-0007: PAL Logging
* ADR-0013: REST/gRPC API Design

