# ADR-0031: Accessibility and Usability Guidelines

## Status
Accepted

## Context
Tiffany aims to empower a broad audience of developers, operators, and contributors through both CLI and browser-based interfaces. Accessibility and usability must be foundational—not optional—especially for:

- Users with visual impairments or cognitive differences
- Non-native English speakers
- Teams relying on screen readers or high-contrast environments
- Developers using terminals, mobile devices, or constrained setups

This ADR defines standards for inclusive UX across all official interfaces and documentation.

---

## Decision

### 🧑‍🦯 Accessibility Standards (WCAG 2.1)

| Target Area     | Standard                                   |
|-----------------|--------------------------------------------|
| Web UI          | WCAG 2.1 AA compliant                      |
| CLI Output      | ANSI color-aware, screen reader-friendly   |
| Docs            | Semantic markdown with alt-text & headers  |
| Terminal TUI    | Keyboard navigable, no mouse requirement   |

All interface surfaces (CLI, TUI, Web) must support:

- Keyboard-only navigation
- Color-blind safe palettes (8+% contrast)
- Focus indicators and tab ordering
- Alt text for icons and illustrative SVGs

---

### 📦 CLI / Terminal Requirements

- Use semantic output:
  - Avoid raw tables or hard-aligned blocks unless explicitly requested
  - Support `--json`, `--plain`, and `--color=never` for accessibility tooling
- Display durations, timestamps, and progress bars using accessible formats
- Support shell-level pagination (`--pager` option) for large output

---

### 🌐 Web UI Guidelines

- Built with semantic HTML and accessible component libraries (e.g. Radix UI or ARIA-compliant)
- All keyboard shortcuts documented and customizable
- Dynamic regions (alerts, toasts) must be ARIA-live
- Charts and logs must support text-mode fallback or export

---

### 📖 Documentation Guidelines

- Every CLI command includes:
  - `--help` with short and long descriptions
  - Examples (both happy path and failure modes)
- Diagrams and figures:
  - Always include textual descriptions
  - Localizable via `.locales/` system (see ADR-0027)

---

### 🧪 Validation and Tooling

| Surface       | Tooling                 | Frequency     |
|---------------|-------------------------|---------------|
| Web UI        | axe-core, Lighthouse     | On each PR    |
| CLI Output    | snapshot + `cargo test` | On CI builds  |
| Docs          | markdownlint + manual   | On pre-merge  |

Automated checks will be extended to cover:
- Accessibility regression (e.g., color contrast drift)
- Missing alt text or ARIA violations
- Unlabeled input or focus traps

---

## Consequences

- Increases front-end and CLI dev complexity (e.g. text-mode fallbacks)
- Promotes universal accessibility across environments (remote SSH, screen readers, touch UIs)
- Boosts adoption by inclusive engineering teams and open-source contributors

---

## Related

- ADR-0027: Localization and Internationalization Policy
- ADR-0013: REST/gRPC API Design
- ADR-0005: Virtual Canvas & Git Strategy
