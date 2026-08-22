---
id: BUGRAIL-SPECOS-026
version: "0.1"
title: "Team Backend Restart Recovery"
status: draft
changeType: team-recovery
prd: ".prd/prd-agent-team-mode-roadmap.md"
design: "design/agent-team-mode-architecture.md"
dependsOn: [BUGRAIL-SPECOS-003, BUGRAIL-SPECOS-005, BUGRAIL-SPECOS-016, BUGRAIL-SPECOS-018]
---

# BUGRAIL-SPECOS-026: Team Backend Restart Recovery

## 1. Outcome

Reconcile nonterminal Team runs after backend restart from persisted WorkTask,
Session, Git, Worktree and control facts without duplicate dispatch or false
success.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-026.R01` | One process owns TaskEngine scheduling through the existing data-directory ownership mechanism; non-owners cannot independently recover or dispatch the same run. |
| `BUGRAIL-SPECOS-026.R02` | Recovery reconciles WorkTask generations, live/missing Sessions, Worktrees and Git merge truth before deriving Team status. |
| `BUGRAIL-SPECOS-026.R03` | A previously running generation without proof of success becomes interrupted/failed/recoverable according to existing WorkTask semantics, never succeeded. |
| `BUGRAIL-SPECOS-026.R04` | Missing/orphan Session or Worktree facts produce bounded operator-visible recovery records and do not trigger destructive cleanup. |
| `BUGRAIL-SPECOS-026.R05` | Recovery and resume are idempotent, generation-guarded and cannot duplicate active WorkTasks. |
| `BUGRAIL-SPECOS-026.R06` | Completed WorkTasks and their evidence remain immutable while downstream readiness is recalculated from current persisted facts. |

## 3. Existing Modules

- Extend existing TaskEngine startup recovery and WorkTask Git recovery.
- Recompute Team projection only after node reconciliation.
- Reuse WorkTask events; add minimal Team recovery audit only for run-level
  decisions that cannot be attributed to one WorkTask.

## 4. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-026.AC01` | Restart with completed and active nodes preserves completed evidence and never marks the active unknown node successful. |
| `BUGRAIL-SPECOS-026.AC02` | Repeated recovery ticks cannot duplicate a retry, Session, Worktree or downstream claim. |
| `BUGRAIL-SPECOS-026.AC03` | A missing Worktree/Session yields an actionable recovery disposition with no fabricated artifact. |
| `BUGRAIL-SPECOS-026.AC04` | Merge recovery follows Git truth and cannot be canceled into an inconsistent terminal projection. |
| `BUGRAIL-SPECOS-026.AC05` | Desktop/server clients see the same reconstructed persisted facts after reconnect. |

## 5. Non-Goals

Cross-machine active-active scheduling and transparent continuation of an
in-memory Agent process are not promised.

