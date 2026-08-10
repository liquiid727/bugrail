# Archived Module Specs

> Status: superseded design exploration; these files do not authorize
> implementation.

The `SPEC-001..113` files in this directory were generated before the design
was mapped to BugRail's inherited CodeG modules. They propose parallel Artifact,
Workflow, Event Bus, Runtime Provider, Storage Engine, Control API, and Plugin
Registry layers that duplicate existing WorkTask, ACP, SQLite, Tauri/Axum, and
event behavior.

Use the canonical vertical Feature Specs instead:

| Implementable capability | Canonical Spec |
|---|---|
| Spec contract and quality gates | `../../../../../.features/BUGRAIL-SPECOS-001-work-task-quality/spec.md` |
| Durable run evidence | `../../../../../.features/BUGRAIL-SPECOS-002-work-task-run-evidence/spec.md` |
| WorkTask dependencies | `../../../../../.features/BUGRAIL-SPECOS-003-work-task-dependencies/spec.md` |
| Integration and handoff | `../../../../../.features/BUGRAIL-SPECOS-004-integration-work-task-handoff/spec.md` |
| Deterministic Context Pack | `../../../../../.features/BUGRAIL-SPECOS-005-deterministic-context-pack/spec.md` |
| Repository impact snapshot | `../../../../../.features/BUGRAIL-SPECOS-006-repository-impact-snapshot/spec.md` |
| Agent/model routing | `../../../../../.features/BUGRAIL-SPECOS-007-explainable-routing/spec.md` |
| Run evaluation | `../../../../../.features/BUGRAIL-SPECOS-008-run-evaluation-projection/spec.md` |
| Project memory candidates | `../../../../../.features/BUGRAIL-SPECOS-009-project-memory-candidates/spec.md` |
| Controlled Skill candidates | `../../../../../.features/BUGRAIL-SPECOS-010-controlled-skill-candidates/spec.md` |

The product requirement source is
`../../../../../.prd/prd-specos-delivery-control.md`; dependency order is
`../../../../../.features/roadmap.md`.
