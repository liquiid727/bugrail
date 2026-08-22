# Agent Team Mode Architecture

## Meta

- Status: proposed umbrella design
- Date: 2026-08-21
- Product source: `.prd/prd-agent-team-mode-roadmap.md`
- Existing design: `design/specos-control-plane-design.md`
- Decisions: ADR `001-003`

This document fixes the architectural invariants shared by the split Agent Team
Features. It is not a substitute for their normative interfaces and acceptance
criteria.

## 1. Core Decision

Agent Team Mode deepens the existing WorkTask delivery module.

```text
.codeg Agent/Team/Workflow definitions
                 |
        validation and snapshot
                 |
          TeamRun projection
                 |
       ordinary WorkTasks + edges
                 |
      existing TaskEngine claim path
                 |
 ACP / Session / Worktree / Context / gates
```

The external seam remains the existing command-core families used by Tauri and
Axum. Team commands create and control projections; WorkTask commands operate
the nodes.

## 2. Required Module Placement

| Concern | Existing module to deepen |
|---|---|
| Node lifecycle, retry, cancel, recovery | `work_task/engine.rs` and WorkTask persistence |
| DAG readiness and concurrency | WorkTask claim predicates |
| Agent execution | ACP manager and existing Agent adapters |
| Run generation | `work_task_run` keyed by `task_id/run_seq` |
| Dependencies | `work_task_dependency` |
| Gates and acceptance | WorkTask contract/gate modules |
| Handoff and integration | WorkTask handoff/integration modules |
| Worktree ownership | existing WorkTask Git/Worktree implementation |
| Context | existing Context Loadout/Package modules |
| Durable task audit | `work_task_event` |
| Live client refresh | existing EventEmitter and transport adapters |
| Team projection/control | `team_run` and `team_run_task` |
| Client interaction | existing Teams page, Tasks context and Task Detail |

## 3. Rejected Parallel Modules

The following modules must not be introduced:

- a `team_task` lifecycle mirroring WorkTask;
- a `team_task_attempt` lifecycle mirroring `work_task_run`;
- a `team_task_dependency` graph mirroring WorkTask dependencies;
- a Team scheduler loop competing with TaskEngine;
- a Team event store mirroring `work_task_event`;
- a Team artifact database duplicating gates, handoffs, diffs, or run evidence;
- a Team Worktree manager wrapping the existing implementation;
- a Team knowledge adapter replacing the Context control plane.

A Feature that needs a fact not represented by an existing module may add the
smallest Team-specific projection after showing why the fact cannot be derived.

## 4. Authority Model

| Fact | Authority |
|---|---|
| Agent/Model/Team/Workflow/Context definitions | validated `.codeg/*.yaml` |
| Team identity, control and definition snapshot | `team_run` |
| Team node membership | `team_run_task` |
| Node execution state | `work_task` |
| Attempt/generation state | `work_task_run` |
| Dependency readiness | `work_task_dependency` plus WorkTask facts |
| Acceptance and gates | WorkTask contract/gate results |
| Session and Worktree | existing WorkTask/ACP/Git facts |
| Context used | immutable Context Package bound to `task_id/run_seq` |
| Durable timeline | WorkTask events and minimal Team control audit |
| Live notification | EventEmitter; never the sole truth |

## 5. State Semantics

### WorkTask node state

The existing WorkTask state machine is unchanged and authoritative.

### Team control state

The baseline control state is intentionally small:

```text
running | paused | cancel_pending | canceled
```

`cancel_pending` requires a Feature migration and must not be assumed before
`BUGRAIL-SPECOS-018` is accepted.

### Team display status

Display status is derived from control state and node facts. Values such as
`planning`, `awaiting_approval`, `blocked`, `reviewing`, `failed`, and
`completed` are projections, not a second persisted node lifecycle.

Terminal transitions and user controls require transactional preconditions.
Cancellation cannot claim completion until active WorkTasks have reached the
declared cancellation/recovery disposition.

## 6. Configuration

The current source-of-truth files remain:

```text
.codeg/agents.yaml
.codeg/teams.yaml
.codeg/context.yaml
```

Changing to per-profile or per-workflow directories requires a separate
migration Feature with compatibility and rollback. Unknown fields must not be
presented as enforced.

## 7. Static Run Materialization

1. Load and validate project catalogs.
2. Resolve the selected static Workflow and Team.
3. Snapshot workflow identity, version, hash, and run input.
4. In one transaction, create `team_run`, ordinary WorkTasks, node bindings,
   and WorkTask dependencies.
5. Claim only nodes allowed by dependency, Team control, and concurrency
   predicates.
6. Let TaskEngine allocate Session, Worktree, Context, and run generation.
7. Project Team status from persisted WorkTask facts.

Partial materialization cannot be reported as a successfully started run.

## 8. Dynamic Planning

Dynamic planning is a proposal pipeline, not another runtime:

```text
goal -> Planner WorkTask -> bounded plan artifact -> deterministic validation
     -> approval/policy decision -> immutable run snapshot -> WorkTasks
```

Planner output cannot write runtime state directly. The planning Feature must
define schema limits, invalid-output recovery, approval semantics, and snapshot
hashing before implementation.

## 9. Quality And Finalization

Reviewer work is an ordinary WorkTask using a reviewer Agent Profile. Command,
test, build, and approval semantics reuse existing WorkTask gates where they
fit. A new node kind is justified only when it has execution, retry, recovery,
and evidence semantics that cannot be expressed through a WorkTask and gate.

Team finalization aggregates node facts and evidence references. It does not
copy transcripts or gate records into a second artifact store.

## 10. Retry And Reassignment

Retry uses existing WorkTask generation semantics. Reassignment changes the
next generation's resolved Agent input and preserves prior generations.

The owning Feature must define downstream invalidation. A dependent node that
already consumed an older generation cannot remain silently valid after an
upstream retry, profile change, Context change, or Git head change.

## 11. Permissions, Budgets, And Fallback

These are independent policies:

- permission policy decides whether an action is allowed;
- budget policy decides whether more execution may begin or continue;
- fallback policy decides whether a failed route may create another execution.

No field is considered enforced until its Feature is accepted. Each decision
must be persisted with its inputs, outcome, actor/policy, and affected
`task_id/run_seq`.

## 12. Recovery And Ownership

TaskEngine remains the scheduler owner. Team start must not claim active
execution when no process owns the TaskEngine lease/lock.

Backend recovery first reconciles WorkTasks, Sessions, Git and Worktrees, then
recomputes Team projections. Unknown execution never becomes success without
evidence. Recovery actions are idempotent and operator-visible.

## 13. Client Strategy

- Extend the existing Teams page and Task Detail.
- Link Team nodes to ordinary WorkTask inspection and actions.
- Reuse the existing transport interface and reconnect hooks.
- Treat events and polling as refresh mechanisms over persisted projections.
- Do not create a second frontend task/attempt store.
- Keep a semantic list as the accessible graph fallback.

The first intake path may live on Teams. Composer integration is an application
execution-mode control, not an ACP `SessionModeInfo` value.

## 14. Verification Strategy

Tests cross the existing module interfaces:

- pure validation and policy tests for deterministic logic;
- SQLite-backed command-core tests for transactions and CAS behavior;
- fake ACP adapters for execution and failure paths;
- Git/Worktree fixtures for integration and recovery;
- Tauri/Axum parity tests over shared core behavior;
- Teams/Task Detail tests over persisted projections and reconnect refresh;
- one representative static Workflow E2E before dynamic planning expands it.

Paid providers and mobile clients are not required for deterministic core CI.

## 15. Feature Order

The normative order and dependencies are declared in `.features/roadmap.md`.
Draft Features `018-027` progressively add reliability, intake, planning,
quality, retry/reassignment, governance, recovery, and remote operations.

