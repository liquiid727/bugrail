---
id: BUGRAIL-SPECOS-024
version: "0.1"
title: "Team Usage Accounting And Budget"
status: draft
changeType: team-budget
prd: ".prd/prd-agent-team-mode-roadmap.md"
design: "design/agent-team-mode-architecture.md"
dependsOn: [BUGRAIL-SPECOS-003, BUGRAIL-SPECOS-018]
---

# BUGRAIL-SPECOS-024: Team Usage Accounting And Budget

## 1. Outcome

Attribute measurable runtime usage to WorkTask generations and Team runs, warn
at configured thresholds, and prevent new dispatch when an enforceable hard
budget is exhausted.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-024.R01` | Usage records provider/model, input/output/cached tokens when reported, duration, cost basis, source and affected `task_id/run_seq`. |
| `BUGRAIL-SPECOS-024.R02` | Team totals are projections over existing usage/run facts and do not copy provider records into a second accounting store. |
| `BUGRAIL-SPECOS-024.R03` | Budget configuration distinguishes warning, hard, per-run and optional per-profile limits with currency/model-price provenance. |
| `BUGRAIL-SPECOS-024.R04` | Hard budgets block new claims before dispatch when the required measurement is available and current; delayed provider usage cannot be described as a strict pre-spend guarantee. |
| `BUGRAIL-SPECOS-024.R05` | Budget stop preserves outputs, Worktrees and evidence and requires an explicit policy adjustment/resume decision. |
| `BUGRAIL-SPECOS-024.R06` | Unknown or unsupported usage is displayed as unknown and never fabricated as zero. |

## 3. Existing Modules

- Reuse token usage persistence and WorkTask run/conversation associations.
- Add budget evaluation as an internal WorkTask claim predicate.
- Extend Team projection and UI with attributable summaries and blockers.

## 4. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-024.AC01` | Reported usage aggregates exactly once across retries and Team nodes. |
| `BUGRAIL-SPECOS-024.AC02` | A current hard-limit decision prevents the next claim and records an explainable blocker. |
| `BUGRAIL-SPECOS-024.AC03` | Warning thresholds do not stop execution but produce one deduplicated visible fact. |
| `BUGRAIL-SPECOS-024.AC04` | Missing provider usage remains unknown and cannot falsely satisfy a budget assertion. |
| `BUGRAIL-SPECOS-024.AC05` | Budget adjustment and resume are authenticated, audited and recover after restart. |

## 5. Non-Goals

Budgeting does not silently choose another model; route fallback belongs to
`BUGRAIL-SPECOS-025`.

