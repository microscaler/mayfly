# Tiffany ADR Index

This document tracks relevant Architectural Decision Records (ADRs) for the Tiffany agentic runtime project. ADRs are numbered sequentially and grouped by functional domain.

## 🗂️ Summary of Additional ADR Recommendations:

| ADR | Description                                     | Summary                                                                                                    | Importance |
| --- |-------------------------------------------------|------------------------------------------------------------------------------------------------------------| ---------- |
| 1   | Choice of Rust Version                          | Use the latest stable Rust version for performance, safety, and ecosystem benefits.                        | High       |
| 2   | Doc book Policy                                 | Documentation standards, structure, and tooling for high-quality docs.                                     | High       |
| 3   | Task Scheduler Model                            | Coroutine-first, cooperative scheduling with stateful yield and resume semantics.                           | High       |
| 4   | Agent Loop and ReAct Design                     | Agent lifecycle: reasoning, acting, yielding, and tool interaction.                                        | High       |
| 5   | Virtual Canvas Git Strategy                     | Code change tracking, patch application, and micro-commits.                                                | Medium     |
| 6   | WAL Schema and Replay Policy                    | Write-ahead log format, recovery semantics, and log compaction.                                            | High       |
| 7   | Process Activity Log (PAL)                      | Real-time activity tracking for agent tasks, complementing the WAL.                                        | Medium     |
| 8   | Persistent Agent Memory Strategy                | Neo4j semantic memory, graph structure, and retrieval policies for agent continuity.                       | High       |
| 9   | Code Structure Graph and Symbol Analysis        | Graph-based code indexing and semantic AST analysis into Neo4j.                                            | Medium     |
| 10  | Task Plan Timeline and Execution Metadata       | Structure for task execution metadata, plan history, and decision lineage.                                 | Medium     |
| 11  | Plugin and MCP Tooling Architecture             | Integration of internal and external tools securely and generically.                                       | High       |
| 12  | Agent Skill System (Future Plan)                | Representation of agent capabilities as composable skills.                                                 | Medium     |
| 13  | Firecracker MicroVM Integration                 | VM lifecycle, execution isolation, and resource constraints.                                               | High       |
| 14  | Command Execution and Safety Policy             | Shell execution flow, trust boundaries, and `--yolo` mode considerations.                                  | High       |
| 15  | REST/gRPC API Design                            | Interface contract for external task submission, result retrieval, and metadata queries.                   | High       |
| 16  | Filesystem Socket Protocol for CLI              | Local interaction protocol for invoking agent actions via socket.                                          | Medium     |
| 17  | Kubernetes Operator and CRD Design              | Custom resource definitions for managing agent lifecycle and orchestration.                                 | Medium     |
| 18  | Metrics Policy and Instrumentation Plan         | Prometheus metrics structure, naming conventions, and dashboard philosophy.                                | Medium     |
| 19  | Logging Strategy and Span Hierarchy             | Use of `tracing`, span lifecycles, and log level defaults.                                                 | Medium     |
| 20  | Versioning and Release Policy                   | Semantic versioning, LTS channels, and changelog protocol.                                                | Medium     |
| 21  | Contributor Roles and Governance                | Roles, responsibilities, PR review flow, and escalation path.                                              | Medium     |
| 22  | Backup, Disaster Recovery, Failover             | Policies for data recovery, backups, failover, and explicit RTO/RPO.                                       | Critical   |
| 23  | Secrets and Credentials Management              | Handling of secrets, keys, tokens, integration with vaults, and rotation policies.                         | Critical   |
| 24  | Authentication and Authorization                | Secure authentication, RBAC, OAuth/OIDC, mTLS, and trust boundaries.                                       | Critical   |
| 25  | Dependency Management and Update Policy         | Management and update of dependencies, vulnerability scanning, and update policies.                        | High       |
| 26  | Performance and Scalability Strategy            | Performance benchmarks, scalability tests, and optimization strategies.                                    | High       |
| 27  | Localization and Internationalization           | Handling of i18n, Unicode support, locale-aware formatting, and error handling.                            | Medium     |
| 28  | Compliance, Auditing, Regulatory Considerations | Compliance with privacy regulations, auditing, log retention, and reporting mechanisms.                    | High       |
| 29  | Data Retention and Privacy                      | Data retention policies, lifecycle management, and privacy guarantees.                                     | High       |
| 30  | Cost Management and Budgeting                   | Cost monitoring, cloud usage, budgeting, and cost optimization strategies.                                 | Medium     |
| 31  | Accessibility and Usability Guidelines          | Ensuring interfaces are accessible, usable, and WCAG compliant.                                            | Medium     |


---


## Choice of Rust Version
- **ADR-0001**: Choice of Rust Version  
  Decision to use the latest stable Rust version for performance, safety, and ecosystem benefits.

## Doc book Policy
- **ADR-0002**: Doc book Policy  
  Documentation standards, structure, and tooling for maintaining high-quality project documentation.

## 🧠 Agent Architecture

- **ADR-0003**: Task Scheduler Model  
  Coroutine-first, cooperative scheduling with stateful yield and resume semantics.

- **ADR-0004**: Agent Loop and ReAct Design  
  Lifecycle of an agent: reasoning, acting, yielding, and interacting with tools.

- **ADR-0005**: Virtual Canvas Git Strategy  
  Strategy for code change tracking, patch application, and micro-commits.

---

## 📦 Storage & Durability

- **ADR-0006**: WAL Schema and Replay Policy  
  Format of write-ahead log, recovery semantics, and log compaction strategy.

- **ADR-0007**: Process Activity Log (PAL)  
  Real-time activity tracking for agent tasks, complementing the WAL.

- **ADR-0008**: Persistent Agent Memory Strategy  
  Design of Neo4j semantic memory, graph structure, and retrieval policies.
  Emphasize goals, prompts, plan recall, continuity between sessions.


- **ADR-0009**: Code Structure Graph and Symbol Analysis
  graph-based code indexing and semantic AST analysis into
  Neo4j, enabling precise symbol tracking and refactoring intelligence.


- **ADR-0010**: Task Plan Timeline and Execution Metadata
  Structure for task execution metadata, including plan history and decision lineage.
  

---

## 🧩 Modularity & Extensibility

- **ADR-0011**: Plugin and MCP Tooling Architecture  
  How Tiffany integrates internal and external tools securely and generically.

- **ADR-0012-**: Agent Skill System (Future Plan)  
  Representation of agent capabilities as composable skills.

---

## 🔐 Runtime & Execution
- **ADR-0013**: Firecracker MicroVM Integration  
  VM lifecycle, execution isolation, resource constraints.

- **ADR-0014**: Command Execution and Safety Policy  
  Shell execution flow, trust boundaries, and `--yolo` mode considerations.

---

## 📡 Interfacing & I/O

- **ADR-0015**: REST/gRPC API Design  
  Interface contract for external task submission, result retrieval, and metadata queries.

- **ADR-0016**: Filesystem Socket Protocol for CLI  
  Local interaction protocol for invoking agent actions via socket.

- **ADR-0017**: Kubernetes Operator and CRD Design
  Custom resource definitions for managing agent lifecycle and task orchestration.

---

## 🧪 Observability

- **ADR-0018**: Metrics Policy and Instrumentation Plan  
  Prometheus metrics structure, naming conventions, and dashboard philosophy.

- **ADR-0019**: Logging Strategy and Span Hierarchy  
  Use of `tracing`, span lifecycles, log level defaults.

---

## 🧭 Governance & Open Source

- **ADR-0020**: Versioning and Release Policy  
  Semantic versioning, LTS channels, changelog protocol.

- **ADR-0021**: Contributor Roles and Governance  
  Roles, responsibilities, PR review flow, escalation path.

---

**22. Backup, Disaster Recovery, and Failover Strategy**

* Policies around data recovery, backups, and failover mechanisms.
* Clearly defines how agent memory, logs, and metadata are backed up.
* Explicit recovery process, RTO (Recovery Time Objective), and RPO (Recovery Point Objective).

---

**23. Secrets and Credentials Management**

* Explicit handling of secrets, keys, tokens, and credentials.
* Integration with secure vaults (e.g., HashiCorp Vault, AWS Secrets Manager, Kubernetes Secrets).
* Auditing access and rotation policies.

---

**24. Authentication and Authorization**

* How FAR agents, developers, and other components authenticate securely.
* Role-Based Access Control (RBAC), OAuth/OIDC integration, or mTLS approaches.
* Explicitly defined trust boundaries and security contexts.

---

**25. Dependency Management and Update Policy**

* Management and update processes for third-party libraries, Rust crates, Kubernetes components, and container images.
* Vulnerability scanning and patching procedures.
* Policies around dependency updates, deprecations, and removals.

---

**26. Performance and Scalability Strategy**

* Detailed performance benchmarks, scalability tests, and capacity planning.
* Horizontal vs. vertical scaling policies.
* Strategies for proactive performance optimization and reactive capacity adjustments.

---

**27. Localization and Internationalization (i18n) Policy**

* How the system will handle internationalization, localization, and multilingual capabilities.
* Consideration of Unicode support, locale-aware formatting, and error handling.

---

**28. Compliance, Auditing, and Regulatory Considerations**

* Compliance with privacy regulations (GDPR, CCPA), standards (SOC 2, ISO 27001).
* Auditing policies, log retention strategies, and compliance reporting mechanisms.

---

**29. Data Retention and Privacy**

* Explicit data retention policies, data lifecycle management, and privacy guarantees.
* Handling and protection of personally identifiable information (PII), if relevant.

---

**30. Cost Management and Budgeting**

* Explicit policies around cost monitoring, cloud usage, and resource budgeting.
* Alerting on cost overruns, proactive cost management strategies, and cost optimization.

---

**31. Accessibility and Usability Guidelines**

* Ensuring the interfaces (CLI, UI, documentation) are accessible and usable.
* Compliance with WCAG accessibility standards.

---

