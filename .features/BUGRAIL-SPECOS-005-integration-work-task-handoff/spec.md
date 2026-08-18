---
id: BUGRAIL-SPECOS-005
version: "0.2"
title: "Integration WorkTask And Handoff"
status: approved
changeType: work-task-deepening
prd: ".prd/prd-specos-agent-team-context-system.md"
design: "design/specos-control-plane-design.md"
clientDesign: "design/specos-client-interaction-design.md"
codeBaseline: "55545d43"
dependsOn: [BUGRAIL-SPECOS-001, BUGRAIL-SPECOS-003, BUGRAIL-SPECOS-004]
---

# BUGRAIL-SPECOS-005: Integration WorkTask And Handoff

Approved slice: Test Spec 0.2 `T01-T06` (handoff roundtrip, missing handoff
blocks `integration_source`, source-head order, conflict `MERGE_HEAD`,
containment-gated landing, legacy compatibility). Full Worktree merge
orchestration in later sections stays planned.

## 1. Summary

Represent integration as a typed WorkTask that consumes eligible source
WorkTasks, merges their branches inside its own Worktree, and lands through the
existing gated merge flow. Extend the current `task_complete` reporting path
with a structured handoff; do not add an Integration runtime or bypass Git
truth.

### Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-005.R01` | A WorkTask is explicitly typed `implementation` or `integration`; legacy rows default to implementation. |
| `BUGRAIL-SPECOS-005.R02` | A source run can record one versioned, bounded handoff through the trusted WorkTask reporting path. |
| `BUGRAIL-SPECOS-005.R03` | An integration task becomes ready only when every source is in review, Spec-current, gate-eligible, and has an existing branch plus handoff. |
| `BUGRAIL-SPECOS-005.R04` | Integration occurs in the integration task Worktree and records source heads, merge order, conflicts, resolution, and verification. |
| `BUGRAIL-SPECOS-005.R05` | Landing the integration task proves that every recorded source head is contained in base before atomically settling source tasks. |

PRD coverage: `P-DC-07`, `P-DC-08`, `P-DC-16`, `P-DC-18`.

## 2. Existing Modules And Interface

| Existing module | Change |
|---|---|
| WorkTask entity/service | Add task kind, handoff storage, integration-source eligibility, and coordinated settlement. |
| `acp/work_task_tools.rs` | Add optional handoff fields to `task_complete`; older Agents remain compatible. |
| `work_task/engine.rs` | Compose integration instructions and reuse ACP spawn/resume, Worktree, preflight, and merge recovery. |
| `work_task/git.rs` | Add source-head containment/merge helpers behind the existing Git implementation. |
| Task Detail/Graph | Show source eligibility, handoffs, merge order, conflicts, and contained heads. |

Caller-facing operations remain in the WorkTask command family:

```text
work_task_handoff_get(task_id, run_seq) -> Handoff?
work_task_integration_plan(task_id) -> IntegrationPlan
work_task_integration_refresh(task_id) -> IntegrationPlan
```

Starting, canceling, returning, retrying, merging, and completing continue to use
existing commands.

## 3. Data Contract

- Add `work_task.task_kind TEXT NOT NULL DEFAULT 'implementation'`.
- `work_task_handoff` primary key `(task_id, run_seq)` stores schema version,
  source branch/head, changed paths, decisions, risks, verification references,
  unresolved items, capped summary, and creation time.
- `work_task_integration_source` stores integration task, source task, source
  run, captured head, merge order, and settlement status. It is materialized
  from `integration_source` dependencies when execution is claimed.

Limits: 100 changed paths, 50 decisions/risks/unresolved items, 64 KiB serialized
handoff, 32 sources per integration task. All paths are repository-relative.

## 4. State And Git Rules

1. Only `task_complete` correlated to the live `(connection_id, run_seq)` can
   write an Agent handoff. User edits create a new human-authored revision and
   are visibly attributed; arbitrary gate/result commands cannot write it.
2. Source eligibility is calculated from persisted Spec/gate/Git facts. Agent
   verdict alone is insufficient.
3. Claiming an integration task snapshots every source `run_seq` and branch
   head. Any later source retry/rebind/head change makes the plan stale and
   blocks launch/merge until refresh.
4. The integration Agent receives structured handoffs and exact source refs,
   merges in recorded order on the integration branch, resolves conflicts, and
   runs normal configured gates.
5. Existing merge recovery remains authoritative for the integration branch.
   After landing, Git containment is checked for every captured source head.
6. Only after containment succeeds may source tasks transition `review -> done`
   with `merge_commit` equal to the integration landing commit and an
   `integrated_by` event. The integration task and all source transitions commit
   in one database transaction after Git truth is known.
7. Cancel/failure leaves source tasks in review. Cleanup never deletes a source
   Worktree before successful contained-head settlement.

## 5. Errors, Security, And UI

| Error key | Condition |
|---|---|
| `workTask.integration.invalidSource` | Source type/folder/state/branch/handoff is invalid. |
| `workTask.integration.ineligibleSource` | Spec or gates are not eligible. |
| `workTask.integration.stalePlan` | Source run or head changed after snapshot. |
| `workTask.integration.conflict` | Merge conflicts remain unresolved. |
| `workTask.integration.notContained` | Base does not contain every captured source head after landing. |

Handoff text is treated as untrusted Agent output and rendered as plain
Markdown through existing sanitization. Source paths are canonicalized. The UI
covers no sources, loading, waiting source, eligible, stale, integrating,
conflict, verification failure, landed, and transport failure.

## 6. Client Interaction Contract

This Feature extends Task Detail `Plan`, Graph edges, and Run Inspector Summary.

- Creating/editing a task exposes `Implementation` or `Integration`; legacy
  tasks display Implementation without requiring migration input.
- An integration task Plan lists source tasks in deterministic merge order with
  readiness, captured `run_seq`/head, branch, handoff presence, Spec/gate state,
  and a direct link to the source task/session/diff.
- Selecting a source expands its bounded handoff: summary, changed paths,
  decisions, risks, verification, and unresolved items. Agent Markdown uses the
  existing sanitized renderer and is visibly labeled as Agent-authored.
- `Refresh plan` previews old/new run and head facts before replacing a stale
  snapshot. It cannot silently refresh as part of Start or Merge.
- Conflict state shows conflicting paths, merge order, current integration
  branch, and the next safe action (`Open session`, `Retry`, or `Return`).
- Landed state shows containment for every captured source head and the shared
  integration commit; source tasks link back to the integration task.
- Graph uses labeled integration-source edges and a distinct task-kind badge,
  without adding a new status color vocabulary.

Client calls are `workTaskHandoffGet`, `workTaskIntegrationPlan`, and
`workTaskIntegrationRefresh` in `src/lib/api.ts`, with exact DTOs in
`src/lib/types.ts`. UI modules are `integration-plan`, `handoff-panel`,
`integration-refresh-dialog`, and `containment-summary` under the SpecOS task
component directory.

Required states are no sources, loading, waiting source, eligible, stale plan,
integrating, conflict, verification failure, landed, and transport failure.
Source tables become stacked disclosure cards below the tablet breakpoint and
preserve keyboard access to every action.

## 7. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-005.AC01` | Legacy task rows and old `task_complete` payloads remain compatible and produce implementation tasks with optional summary only. |
| `BUGRAIL-SPECOS-005.AC02` | A correlated run stores one bounded handoff with exact branch/head and actor attribution. |
| `BUGRAIL-SPECOS-005.AC03` | Missing/stale Spec, gate, handoff, branch, or run facts prevent integration claim with source-specific reasons. |
| `BUGRAIL-SPECOS-005.AC04` | Parallel eligible sources merge in deterministic order inside the integration Worktree; conflicts remain inspectable and retryable. |
| `BUGRAIL-SPECOS-005.AC05` | Integration cannot land through stale UI, Agent verdict, or direct source-task completion bypass. |
| `BUGRAIL-SPECOS-005.AC06` | Successful landing proves all source heads contained, settles sources once, and survives crash recovery idempotently. |
| `BUGRAIL-SPECOS-005.AC07` | Failed/canceled integration preserves source review state and Worktrees. |
| `BUGRAIL-SPECOS-005.AC08` | Desktop/server behavior and all UI states are equivalent. |

## 8. Testing And Implementation Order

1. Task-kind/handoff/source migrations and bounded DTO tests.
2. Trusted MCP correlation and backward-compatibility tests.
3. Eligibility snapshot/staleness and Git fixture tests, including conflicts.
4. Coordinated landing, containment, crash recovery, and cleanup tests.
5. Transport and Task Detail/Graph state tests.
6. Full WorkTask, ACP, Git, desktop, and server regression suites.
