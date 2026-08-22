---
id: BUGRAIL-SPECOS-022
version: "0.1"
title: "Team Retry And Agent Reassignment"
status: draft
changeType: team-recovery-control
prd: ".prd/prd-agent-team-mode-roadmap.md"
design: "design/agent-team-mode-architecture.md"
adr: "design/adr/ADR-002-agent-profile-team-worktask.md"
dependsOn: [BUGRAIL-SPECOS-002, BUGRAIL-SPECOS-003, BUGRAIL-SPECOS-004, BUGRAIL-SPECOS-019]
---

# BUGRAIL-SPECOS-022: Team Retry And Agent Reassignment

## 1. Outcome

Retry or reassign one Team node through existing WorkTask generation semantics
without rerunning unrelated valid work or hiding stale downstream results.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-022.R01` | Node retry invokes the existing WorkTask retry/claim path and creates a new `run_seq`; prior generations remain auditable. |
| `BUGRAIL-SPECOS-022.R02` | Reassignment selects an enabled Team member for the next generation and snapshots the new resolved Agent/model/Context route. |
| `BUGRAIL-SPECOS-022.R03` | The operation uses a node generation/state precondition and is idempotent under repeated or concurrent client requests. |
| `BUGRAIL-SPECOS-022.R04` | Downstream nodes that consumed an older upstream generation become explicitly stale or blocked according to a declared invalidation policy. |
| `BUGRAIL-SPECOS-022.R05` | Already valid unrelated branches are not rerun, canceled, or rewritten. |
| `BUGRAIL-SPECOS-022.R06` | Retry/reassignment reason, actor, old/new resolution and affected downstream nodes are durably recorded. |

## 3. Existing Modules

- Reuse WorkTask retry, `run_seq`, dependencies, run snapshots and events.
- Reuse Agent Profile resolution and Context Package compilation.
- Extend Team projection with generation provenance and stale blockers.

## 4. Interface

Add Team node retry/reassign operations that delegate to WorkTask command-core
behavior. They return the new WorkTask generation and downstream invalidation
projection, not a separate Team attempt.

## 5. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-022.AC01` | Retrying one failed node creates exactly one new WorkTask generation and does not rerun a valid sibling. |
| `BUGRAIL-SPECOS-022.AC02` | Reassignment records the new Agent resolution while preserving every prior generation. |
| `BUGRAIL-SPECOS-022.AC03` | Double-click/concurrent retry cannot create duplicate active generations. |
| `BUGRAIL-SPECOS-022.AC04` | Downstream work based on an older generation cannot remain silently eligible for final acceptance. |
| `BUGRAIL-SPECOS-022.AC05` | Restart preserves retry, reassignment and invalidation provenance. |

## 6. Non-Goals

Automatic model/provider fallback and budget-driven route changes are separate
Features.

