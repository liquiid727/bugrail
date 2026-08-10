# SpecOS Control Plane Design

## Meta

- Product: `Code: BugRail`
- Status: proposed
- Date: `2026-08-09`
- Product source: `docs/specos/product-vision.md`
- Product requirements: `.prd/prd-specos-delivery-control.md`
- Existing-code map: `docs/specos/codeg-module-map.md`
- First Feature: `.features/BUGRAIL-SPECOS-001-work-task-quality/spec.md`
- Decision: `design/adr/ADR-001-embed-specos-in-work-task.md`

## Purpose

Define where SpecOS behavior belongs in BugRail and how it extends inherited
CodeG modules without creating parallel workflow, runtime, event, or storage
systems.

## System Context

```text
Git-tracked Feature/Test Specs
              |
              v
       existing WorkTask module
   contract + gates + durable timeline
       |          |           |
       v          v           v
 existing ACP  existing Git  existing SQLite
 runtime       Worktrees     and migrations
       \          |           /
        \         v          /
         existing Tasks UI and transports
```

SpecOS is an internal product capability of BugRail. Desktop, standalone server,
and future remote clients use the same WorkTask command-core behavior already
shared by Tauri and Axum.

## Design Decisions

| Decision | Choice | Reason |
|---|---|---|
| Workflow ownership | Deepen the existing WorkTask module. | It already owns task state, concurrency, recovery, Session, Worktree, merge, and timeline invariants. |
| Runtime | Reuse ACP manager and registries. | BugRail already drives multiple built-in and custom Agent adapters. |
| Runtime persistence | SQLite/SeaORM. | Current task transitions and audit records are transactional and queryable. |
| Delivery artifact persistence | Git-tracked Markdown. | Human review, version binding, and repository history remain visible. |
| Live events | Reuse InternalEventBus and EventEmitter. | Existing Tauri/Web adapters cover current clients; live events are not release evidence. |
| Durable events | Extend `work_task_event`. | It is already written in the same transaction as task state changes. |
| Plugin policy | Add an external seam only for behavior with multiple justified adapters. | Avoid pass-through registries and hypothetical interfaces. |
| First delivery | Spec-linked WorkTask with enforced quality gates. | It validates the central product claim using existing behavior. |

## Deep Modules And Interfaces

### WorkTask Delivery Module

This is the primary deep module. Its interface remains the existing WorkTask
command family plus typed results. The implementation owns:

- task contract binding and validation;
- task state transitions and run generations;
- Worktree and ACP Session allocation;
- gate policy and gate-result aggregation;
- merge/complete eligibility;
- durable timeline records;
- crash and retry semantics.

Callers do not orchestrate these rules themselves. Tauri, Axum, and the React UI
cross the same command-core interface.

### Spec Contract Reader

An internal, local-substitutable module reads one Git-tracked Feature Spec and
returns a validated immutable reference:

```text
SpecReference {
  id
  version
  path
  sha256
  acceptance_criteria[]
}
```

The WorkTask stores the validated reference and selected acceptance snapshot.
Tests use a temporary filesystem fixture. This seam remains internal because
there is no remote Spec adapter in the first slice.

### Quality Gate Decision

An in-process module accepts a gate policy and gate attempts, then returns an
explainable decision:

```text
evaluate(policy, attempts) -> GateDecision {
  eligible
  required
  unmet
  waived
}
```

The function has no transport or database concerns. The WorkTask module owns
loading attempts, applying the decision to merge/complete commands, and writing
the decision to the timeline.

### Existing External Seams

- ACP Agent implementations and deterministic test substitutes.
- Tauri and WebSocket event/command transports.
- Git process execution with repository fixtures.

No new generic Plugin Registry is introduced by the first Feature.

## First-Slice Data Model

### `work_task_contract`

One optional contract per WorkTask:

| Field | Meaning |
|---|---|
| `task_id` | Primary key and FK to `work_task.id` |
| `source_spec_id` | Stable Feature Spec ID |
| `source_spec_version` | Exact source version |
| `source_spec_path` | Repository-relative path |
| `source_spec_hash` | SHA-256 of approved source content |
| `acceptance_criteria` | JSON snapshot of selected AC IDs and text |
| `gate_policy` | JSON snapshot of required gates and waiver policy |
| `created_at`, `updated_at` | Audit timestamps |

Index `source_spec_id` for trace queries. Existing tasks have no row and keep
their current behavior.

### `work_task_gate_result`

Append one row per gate attempt:

| Field | Meaning |
|---|---|
| `id` | Local database identifier |
| `task_id`, `run_seq` | Task and execution generation |
| `gate_id`, `gate_type` | Stable gate identity; first slice supports `preflight` and `human_approval` |
| `status` | `running`, `passed`, `failed`, `blocked`, or `waived` |
| `required` | Whether this gate participates in the merge decision |
| `evidence` | JSON references to command, output, test, review, commit, or human approval |
| `actor` | `engine` or authenticated `human`; request JSON cannot choose it |
| `reason` | Required for `failed`, `blocked`, and `waived` |
| timestamps | start and completion timestamps |

Index `(task_id, run_seq, gate_id, id)` for latest-attempt lookup.

## State And Command Semantics

- The existing WorkTask state machine remains authoritative.
- Agent `TurnComplete` can move a task into `review`; it cannot satisfy required
  gates merely by reporting success.
- `work_task_merge` and `work_task_complete` evaluate the latest required gate
  attempts before their existing CAS transition.
- An unmet gate returns a typed, explainable error and records a `gate_decision`
  timeline event without changing the task state.
- A waiver requires a human actor and non-empty reason.
- Gate records are scoped to `run_seq`; retry does not silently reuse a previous
  execution generation's passing result unless the policy explicitly marks the
  gate reusable.
- Existing no-contract tasks keep current preflight and acceptance behavior.

## Preflight Compatibility

The current `WorkTaskPreflight` remains readable during migration. For a
contract-bound task, the configured preflight command produces a structured
gate attempt and the existing `preflight` snapshot remains a UI compatibility
projection. Later cleanup may remove the projection only after all clients use
gate results.

## Errors

New domain conditions map through the existing `AppCommandError` shape:

| Code/i18n key | Condition | Retry |
|---|---|---|
| `workTask.specContract.invalid` | Spec path, ID, version, hash, or AC selection is invalid. | After correcting input |
| `workTask.specContract.stale` | Current file hash differs from the bound hash. | After explicit rebind |
| `workTask.qualityGate.unmet` | Merge/complete requested with required gates not passed. | After gate completion |
| `workTask.qualityGate.invalidWaiver` | Non-human or reasonless waiver. | After valid approval |

Existing HTTP paths, Tauri command names, and `codeg` compatibility identifiers
do not change.

## Security And Privacy

- Spec paths are repository-relative, canonicalized, and constrained to the
  active project root.
- Gate evidence stores references and capped output tails, not secrets or full
  environment dumps.
- Human approvals record actor identity from the authenticated command context.
  The first single-user server mode treats possession of its configured access
  token as the human identity; multi-user authorization is not implied.
- Agent output cannot create a human waiver.

## Performance

- Task list queries continue to avoid loading full gate history.
- Task detail loads the contract and paginated/latest gate attempts.
- Merge/complete performs one indexed latest-attempt query before the existing
  state transition.
- No repository-wide artifact scan occurs on task list or task execution.

## Testing Surface

- Pure tests for gate evaluation and waiver rules.
- SeaORM migration and repository tests for both tables and transactionality.
- WorkTask command-core integration tests for merge/complete blocking.
- Preflight compatibility tests.
- Tauri/Axum transport parity tests through existing handlers.
- React tests for empty, loading, success, stale, failed, waived, and blocked
  states in Task Detail.

## Delivery Feature Placement

| Feature | Placement decision |
|---|---|
| `BUGRAIL-SPECOS-001` | WorkTask Spec contract and trusted preflight/human gate producers. |
| `BUGRAIL-SPECOS-002` | WorkTask run records plus projections over existing events, Conversations, token use, gates, and Git. |
| `BUGRAIL-SPECOS-003` | Dependencies and readiness inside the WorkTask scheduler; no DAG state machine. |
| `BUGRAIL-SPECOS-004` | Integration as a typed WorkTask using existing ACP and Worktrees. |
| `BUGRAIL-SPECOS-005` | Deterministic Context Pack inside WorkTask prompt composition. |
| `BUGRAIL-SPECOS-006` | Bounded local repository-impact snapshot with Rust and TypeScript internal adapters. |
| `BUGRAIL-SPECOS-007` | Deterministic policy over existing ACP/agent/provider registries. |
| `BUGRAIL-SPECOS-008` | Read-only evaluation projection over durable run evidence. |
| `BUGRAIL-SPECOS-009` | SQLite memory candidates; accepted project memory is Git-tracked Markdown. |
| `BUGRAIL-SPECOS-010` | Governance in front of existing ACP Skill storage and refresh behavior. |

The Feature Specs own their exact schemas and behavior. Later concepts are not
registered as empty plugins before their implementing Feature.
