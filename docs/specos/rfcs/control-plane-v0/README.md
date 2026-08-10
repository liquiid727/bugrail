# Archived Control Plane V0 RFC

> Status: superseded design exploration
> Archived: 2026-08-09

This directory preserves the earlier generated blueprint, PRD, module
decomposition, module SPECs, and 86-Issue backlog. The material is retained for
concept recovery, but it is not an approved product baseline or delivery plan.

It was superseded because it:

- treated BugRail as a greenfield runtime instead of extending existing CodeG
  WorkTask, ACP, event, persistence, transport, and UI modules;
- introduced a second artifact path outside BugRail's GoalSpec manifest;
- froze broad interfaces while deployment, persistence, and compatibility
  decisions remained open;
- decomposed the first milestone into 12 kernel modules, 13 plugin seams, and
  40 M0 Issues before validating one vertical user flow;
- lacked exact Feature/Test Spec version binding.

Concepts such as acceptance traceability, context inspection, risk-aware gates,
run evaluation, and controlled Skill evolution remain useful. They must be
promoted through the canonical `design/ -> .features/ -> .issues/` chain.

## Canonical Replacement Map

| Archived concept | Canonical Feature Spec |
|---|---|
| Artifact/Quality Gate/Storage fragments | `BUGRAIL-SPECOS-001` Spec-linked WorkTask quality |
| Run Trace/Event fragments | `BUGRAIL-SPECOS-002` WorkTask run evidence |
| Workflow/DAG | `BUGRAIL-SPECOS-003` WorkTask dependencies |
| Handoff/Integration | `BUGRAIL-SPECOS-004` Integration WorkTask and handoff |
| Context Pack/Resolver | `BUGRAIL-SPECOS-005` Deterministic Context Pack |
| Code Intelligence | `BUGRAIL-SPECOS-006` Repository impact snapshot |
| Agent Resolver/Model Router | `BUGRAIL-SPECOS-007` Explainable routing |
| Eval Aggregator | `BUGRAIL-SPECOS-008` Run evaluation projection |
| Memory Provider | `BUGRAIL-SPECOS-009` Project memory candidates |
| Skill Evolution | `BUGRAIL-SPECOS-010` Controlled Skill candidates |

Archived Runtime Provider, Event Bus, Storage Engine, Control API, and Plugin
Registry Specs have no one-to-one replacement: their required behavior already
exists in CodeG and is extended inside the vertical Features above.
