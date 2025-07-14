# ADR-0025: Dependency Management and Update Policy

## Status
Accepted

## Context
Tiffany spans a complex, distributed runtime involving:

- Rust crates and workspaces
- Container images
- Web UI dependencies
- Kubernetes resources and Helm charts
- Agent and orchestrator configuration schemas

Managing this evolving stack requires a clear update policy that prioritizes **security**, **stability**, and **observability**. Without automated tooling and policy discipline, the system becomes brittle and vulnerable to supply chain risks.

---

## Decision

### 📦 Dependency Categories

| Type              | Examples                          | Tooling                          |
|-------------------|-----------------------------------|----------------------------------|
| Rust crates       | `serde`, `tokio`, `sea-orm`       | `cargo-audit`, `cargo-outdated` |
| System packages   | libc, `openssl`, runtime libs     | Dockerfile + base image scanner |
| Container images  | agent/orchestrator builds         | Snyk, Trivy, `cosign`            |
| JS/UI dependencies| SolidJS, Tailwind, Vite           | `npm audit`, `npm-check`        |
| Helm/K8s charts   | FluxCD, Loki, Prometheus, etc.    | `helm diff`, `kube-score`       |

---

### 🧰 Tooling and Automation

- **Rust**
  - `cargo deny` for license, version, and vulnerability policy
  - `cargo audit` in CI
  - `cargo outdated --root-deps-only` in PR checks

- **Containers**
  - `trivy` or `grype` scans for CVEs
  - `cosign` for image signing and verification
  - Pin base image digests in Dockerfiles (avoid `latest`)

- **JavaScript/Web**
  - `npm audit` in CI
  - `npm-check` for dev-time review
  - Lockfile commits enforced

- **Kubernetes**
  - `kube-score` and `kubescape` for hardening
  - FluxCD automation policies and lockfile commits
  - Hash-pinned Helm chart versions

---

### 🧱 Update Policy

| Policy                             | Rule                                                                           |
|------------------------------------|--------------------------------------------------------------------------------|
| Vulnerability patch window        | CVSS ≥7 must be patched within 48 hours                                       |
| Minor crate upgrades              | Allowed at any time via PRs                                                   |
| Major crate upgrades              | Require RFC or ADR if breaking changes affect public APIs                     |
| Image patching                    | Base image scans weekly; new builds triggered for security patches            |
| Automated dependency PRs          | RenovateBot / Dependabot enabled on all crates and containers                 |
| Deprecation policy                | Must announce deprecations in CHANGELOG 2 versions ahead                      |

---

### 🚨 Enforcement in CI

- Failing `cargo audit` blocks merge
- Trivy scan with fail-on-high severity
- License violations (non-OSI approved) trigger `cargo deny` failure
- Helm diff alerts on drift

---

### 🧩 Ephemeral Tooling

- All dev tooling (formatters, linters, generators) must be version-pinned
- Tool version drift checked via `just lint` or `make check-env`
- Developer onboarding docs specify exact setup instructions

---

### 📓 Developer Workflow Summary

1. `just upgrade` will bump root dependencies using `cargo upgrade`
2. `just security-scan` runs full stack scans (Rust + containers)
3. PRs triggering dependency changes must:
   - Include changelog note
   - Link to CVE or release notes
   - Pass `just test-all && just lint`

---

## Consequences

- Adds continuous overhead in CI/CD and linting pipelines
- Requires clear changelog hygiene and semver discipline
- Strongly improves long-term security, reliability, and trustworthiness
- Prevents unvetted transitive vulnerabilities

---

## Related

- ADR-0022: Backup & Recovery (for supply chain concerns)
- ADR-0026: Performance and Scalability Strategy
- ADR-0017: Release Policy
