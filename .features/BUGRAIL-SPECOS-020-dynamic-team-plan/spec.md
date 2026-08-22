---
id: BUGRAIL-SPECOS-020
version: "0.1"
title: "Dynamic Team Plan Proposal"
status: draft
changeType: team-planning
prd: ".prd/prd-agent-team-mode-roadmap.md"
design: "design/agent-team-mode-architecture.md"
adr: "design/adr/ADR-002-agent-profile-team-worktask.md"
dependsOn: [BUGRAIL-SPECOS-001, BUGRAIL-SPECOS-019]
---

# BUGRAIL-SPECOS-020: Dynamic Team Plan Proposal

## 1. Outcome

Allow a Planner Agent to propose a bounded execution DAG that becomes runnable
only after deterministic validation and the configured approval decision.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-020.R01` | Planning runs as an ordinary WorkTask with an immutable Agent/model/Context snapshot. |
| `BUGRAIL-SPECOS-020.R02` | Planner output conforms to a versioned, bounded schema covering objective, assumptions, tasks, profiles, dependencies, acceptance references, expected impact, validation, and risks. |
| `BUGRAIL-SPECOS-020.R03` | Validation rejects duplicate IDs, cycles, unknown/disabled profiles, invalid limits, missing required acceptance, and unsupported node semantics before materialization. |
| `BUGRAIL-SPECOS-020.R04` | A plan is a proposal artifact; it cannot mutate Team, Workflow, WorkTask, permission, budget, or gate state directly. |
| `BUGRAIL-SPECOS-020.R05` | Approval or project policy selects one exact plan hash; edits create a new proposal version and invalidate the prior decision. |
| `BUGRAIL-SPECOS-020.R06` | Accepted plans materialize ordinary WorkTasks and dependencies atomically and retain a reference to the approved proposal. |
| `BUGRAIL-SPECOS-020.R07` | Invalid output exposes actionable validation errors and can retry the planning WorkTask without starting implementation nodes. |

## 3. Existing Modules

- Planner execution uses Agent Profiles, WorkTask run evidence and Context.
- Human approval reuses the trusted WorkTask gate path where applicable.
- DAG validation extends the existing static Workflow validator as internal
  deterministic logic.
- Materialization reuses TeamRun and WorkTask command-core behavior.

## 4. Interface

The Feature adds plan propose/get/list/approve/reject/materialize operations.
The interface returns proposal hashes and validation results; it never exposes
an internal scheduler or mutable node state.

## 5. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-020.AC01` | Malformed, cyclic or unsupported Planner output cannot create implementation WorkTasks. |
| `BUGRAIL-SPECOS-020.AC02` | Approval binds an exact proposal hash and a changed proposal requires a new decision. |
| `BUGRAIL-SPECOS-020.AC03` | Materialization creates one WorkTask per accepted Agent node and preserves dependency order. |
| `BUGRAIL-SPECOS-020.AC04` | Planner retry preserves earlier proposal/output evidence without authorizing it. |
| `BUGRAIL-SPECOS-020.AC05` | Restart and reconnect restore the current proposal, decision and any materialized run. |

## 6. Non-Goals

No post-launch graph mutation, supervisor loop, Agent-as-Tool ownership, or
unbounded autonomous Team creation is allowed.

