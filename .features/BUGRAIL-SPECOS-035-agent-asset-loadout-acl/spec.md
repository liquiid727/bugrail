---
id: BUGRAIL-SPECOS-035
version: "0.1"
title: "Agent Asset Loadouts And ACL"
status: draft
changeType: agent-context-policy
prd: ".prd/prd-memory-operating-layer-roadmap.md"
design: ".features/bugrail-specoos-memory/05-AGENT-LOADOUT-SCOPE-权限.md"
codeBaseline: "2ab6d5cf"
dependsOn: [BUGRAIL-SPECOS-002, BUGRAIL-SPECOS-008, BUGRAIL-SPECOS-028, BUGRAIL-SPECOS-030, BUGRAIL-SPECOS-032, BUGRAIL-SPECOS-033, BUGRAIL-SPECOS-034]
---

# BUGRAIL-SPECOS-035: Agent Asset Loadouts And ACL

## 1. Outcome

Resolve one immutable, backend-enforced asset loadout per Agent/WorkTask
generation so Memory, Wiki, CodeGraph and Skills remain scoped and explainable
across single-Agent, Team and A/B workflows.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-035.R01` | Agent Profiles select named asset loadouts with explicit inheritance and per-plugin enablement, budgets and query policy. |
| `BUGRAIL-SPECOS-035.R02` | Effective access is the intersection of project, user/team, Agent, task and asset scope; backend Adapter calls enforce it. |
| `BUGRAIL-SPECOS-035.R03` | Private Memory never enters Team/shared runs without an explicit authorized share; project assets never cross project identity. |
| `BUGRAIL-SPECOS-035.R04` | Task context may transfer between assigned Agents through attributable task scope, while unconfirmed hypotheses and A/B variants remain isolated. |
| `BUGRAIL-SPECOS-035.R05` | Every generation persists the exact resolved loadout, asset revisions, exclusions and policy reasons used by Context compilation. |
| `BUGRAIL-SPECOS-035.R06` | UI can edit presets and inspect effective loadouts/denials without treating hidden controls as authorization. |

## 3. Existing Modules

- Deepen Agent Profiles and Context Loadouts from `002` and `008`.
- Reuse WorkTask generation resolution, Team node bindings and immutable
  Context Package evidence.
- Do not add an independent identity or permission service.

## 4. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-035.AC01` | Two Agents using the same model resolve different declared asset loadouts and immutable package evidence. |
| `BUGRAIL-SPECOS-035.AC02` | Project/private/team scope matrix tests reject every unauthorized Adapter query before network or index access. |
| `BUGRAIL-SPECOS-035.AC03` | Planner-to-implementer-to-reviewer handoff shares task evidence but not private Memory or unapproved hypotheses. |
| `BUGRAIL-SPECOS-035.AC04` | A/B runs cannot promote conflicting variants into shared Memory or Skills without an explicit winner/evidence decision. |
| `BUGRAIL-SPECOS-035.AC05` | Tauri/Axum/UI display the same effective policy after restart and configuration revision. |

## 5. Non-Goals

No organization-wide IAM system, model-provider permission replacement or
silent cross-project asset federation is included.
