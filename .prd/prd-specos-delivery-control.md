# PRD: SpecOS Delivery Control In Code: BugRail

## Meta

- Status: draft
- Product vision: `docs/specos/product-vision.md`
- Existing-code map: `docs/specos/codeg-module-map.md`
- Design: `design/specos-control-plane-design.md`
- Supersedes product baseline: `docs/specos/rfcs/control-plane-v0/01-SpecOS-PRD.md`
- Target sequence: `BUGRAIL-SPECOS-001` through `BUGRAIL-SPECOS-010`

## 1. Product Decision

SpecOS is a delivery-control capability inside BugRail, not a second runtime
beside CodeG. It extends the inherited WorkTask, ACP, Worktree, SQLite,
conversation, event, transport, provider, Skill, and Tasks UI modules so a
developer can move from an approved Feature Spec to a traceable, gated merge.

The first useful product is not a generic Artifact kernel or Plugin Registry.
It is a WorkTask that knows exactly which Spec it implements, receives bounded
context, respects dependencies, records evidence, and cannot merge until its
declared gates pass.

## 2. Problem

BugRail already provides projects, conversations, Agent sessions, delegation,
Worktrees, Git operations, task scheduling, preflight commands, merge/retry,
token accounting, and desktop/server transports. The missing layer is delivery
control across those capabilities:

- a WorkTask does not bind to an exact approved Spec and acceptance snapshot;
- Agent completion and one preflight result cannot express independent gates;
- retries and Sessions are visible, but not projected as a version-bound run
  trace;
- tasks can run concurrently, but do not express delivery dependencies;
- integration across task Worktrees relies on manual coordination;
- prompt composition does not record which project context was selected;
- Agent/model selection is mostly an override/default chain rather than an
  explainable task decision;
- successful and failed runs do not feed a governed evaluation, memory, or
  Skill-candidate lifecycle.

## 3. Users And Outcomes

### Primary user

A developer or technical lead using BugRail to implement a reviewed Feature
Spec with one or more Agents while retaining control of risky decisions.

### Required outcomes

1. The user can bind work to an exact Spec and see what “done” means.
2. The user can understand why execution is ready, blocked, routed, or rejected.
3. Parallel work remains isolated and is integrated through Git truth.
4. Every merge decision is backed by durable, inspectable evidence.
5. Repeated evidence may improve future work, but never silently changes
   project behavior or creates an active Skill.

## 4. Golden Path

```text
approved Feature Spec
  -> create one or more Spec-bound WorkTasks
  -> declare dependencies and integration sources
  -> compile a bounded, attributable context pack per run
  -> choose an installed Agent/model with an explainable decision
  -> execute through the existing ACP Session and Worktree flow
  -> record handoff, gates, token use, diff and review evidence
  -> integrate eligible source branches in an integration WorkTask
  -> enforce gates in existing merge/complete commands
  -> inspect the exact run trace and evaluation
  -> propose project memory or Skill candidates for human review
```

At every running or review step the user can inspect, cancel, retry, return with
feedback, answer permission/questions, or take over through existing BugRail
controls.

## 5. Product Requirements

### Contract And Quality

| ID | Requirement |
|---|---|
| `P-DC-01` | A WorkTask can bind to one repository-local Feature Spec by ID, version, path, content hash, selected acceptance criteria, and gate policy. |
| `P-DC-02` | Existing WorkTasks without a Spec binding retain current behavior. |
| `P-DC-03` | Agent completion is evidence only; merge and no-change completion remain blocked until every required gate is eligible. |
| `P-DC-04` | Gate decisions are durable, explainable, scoped to a run generation, and cannot be forged by an Agent or untrusted client payload. |

### Trace And Coordination

| ID | Requirement |
|---|---|
| `P-DC-05` | Each WorkTask run has a durable projection of its Spec, effective config, Session, Worktree, timeline, gates, token use, diff, outcome, and failure category. |
| `P-DC-06` | A WorkTask can depend on other WorkTasks without adding a second task state machine; readiness and block reasons are derived from current WorkTask facts. |
| `P-DC-07` | An integration WorkTask can consume eligible source WorkTasks, their branches, and structured handoffs, resolve conflicts in its own Worktree, and land through the existing merge path. |
| `P-DC-08` | Dependency, integration, retry, cancellation, and crash recovery decisions preserve current CAS and Git-truth invariants. |

### Context And Routing

| ID | Requirement |
|---|---|
| `P-DC-09` | Every run receives a bounded Context Pack whose items record source, revision/hash, reason, order, size, and inclusion/exclusion decision. |
| `P-DC-10` | Repository impact analysis can expand a declared file/module scope to related manifests, imports, tests, and recent change evidence without requiring a remote indexing service. |
| `P-DC-11` | Agent/model routing uses installed Agent metadata, user overrides, task needs, risk, context size, and provider health, and records an explainable immutable decision for the run. |
| `P-DC-12` | Explicit user selection always wins; fallback cannot weaken required gates or silently cross a denied permission. |

### Evaluation And Learning

| ID | Requirement |
|---|---|
| `P-DC-13` | Version-bound run evidence can be aggregated by Agent, model, task kind, risk, context policy, and failure category without copying transcripts into a second event store. |
| `P-DC-14` | Memory extraction creates reviewable, source-linked project candidates; accepted memory is project-local and Git-trackable. |
| `P-DC-15` | A Skill candidate requires repeated independent evidence, validation, explicit approval, versioning, and rollback; one successful run cannot activate a Skill. |

### Control And Compatibility

| ID | Requirement |
|---|---|
| `P-DC-16` | Desktop and standalone-server transports expose equivalent behavior through existing Tauri/Axum patterns. |
| `P-DC-17` | Existing `codeg` command names, routes, URI schemes, database filename, and `CODEG_*` variables remain compatibility contracts. |
| `P-DC-18` | Task Detail and related views cover empty, loading, success, blocked, stale, failed, waived, and transport-error states for the capabilities they expose. |

## 6. Feature Decomposition

| Order | Feature Spec | User-visible closure | Existing module deepened | Depends on |
|---:|---|---|---|---|
| 1 | `BUGRAIL-SPECOS-001` Spec-Linked WorkTask Quality | Exact Spec/AC binding and enforced preflight/human gates | WorkTask | none |
| 2 | `BUGRAIL-SPECOS-002` WorkTask Run Evidence | Inspect one durable run and its evidence | WorkTask event/conversation/token projection | 001 |
| 3 | `BUGRAIL-SPECOS-003` WorkTask Dependencies | See and enforce task readiness/DAG | WorkTask scheduler/service | 001 |
| 4 | `BUGRAIL-SPECOS-004` Integration WorkTask And Handoff | Integrate eligible parallel branches safely | WorkTask, Worktree, ACP task tools | 001-003 |
| 5 | `BUGRAIL-SPECOS-005` Deterministic Context Pack | Inspect exactly what context a run received | WorkTask prompt composition | 001-004 |
| 6 | `BUGRAIL-SPECOS-006` Repository Impact Snapshot | Expand scope to related code/tests with reasons | Git/file workspace internals | 005 |
| 7 | `BUGRAIL-SPECOS-007` Explainable Agent/Model Routing | Explain and override the run route | ACP registry/settings/provider config | 002, 005 |
| 8 | `BUGRAIL-SPECOS-008` Run Evaluation Projection | Compare reliable outcomes and failure classes | Run/token/gate projections | 002, 007 |
| 9 | `BUGRAIL-SPECOS-009` Project Memory Candidates | Review and accept source-linked memory | Run evidence and Context Pack | 005, 008 |
| 10 | `BUGRAIL-SPECOS-010` Controlled Skill Candidates | Validate, approve, activate, and roll back Skills | Existing ACP Skill management | 008, 009 |

Each Feature is a vertical slice. Database, Rust behavior, Tauri/Axum
transport, TypeScript types, UI states, compatibility, and verification remain
in the same Spec rather than becoming separate horizontal module Specs.

## 7. Release And Safety Rules

- Normative Feature/Test Specs are Git-tracked; SQLite stores runtime facts,
  immutable references, decisions, and evidence projections.
- Live `InternalEventBus` and `EventEmitter` delivery improves UX but is never a
  correctness or release-evidence source.
- Backend command-core behavior is authoritative. Disabled buttons do not
  enforce a gate.
- Human approval actor and reason come from trusted command context; an Agent
  or arbitrary request body cannot self-assign the human role.
- Context and evidence use repository-relative canonical paths, size caps, and
  redaction. Secrets and full environment dumps are not retained.
- New external seams require at least two real production adapters. Pure
  policies and local filesystem substitutes remain internal seams.

## 8. Success Criteria

- A valid Spec-bound task completes the golden path through existing WorkTask,
  ACP, Worktree, SQLite, and merge behavior.
- Required gates cannot be bypassed through Agent verdict, stale UI, retry,
  transport differences, stale Spec content, or integration flow.
- A user can answer “what ran, with what context, why this route, what changed,
  what passed, and why it merged” from durable data after process restart.
- Legacy task, Session, Git, and transport regression suites remain green.
- No Feature introduces a parallel workflow engine, Agent runtime, event bus,
  editable Artifact database, or generic Plugin Registry.

## 9. Non-Goals

- Rebuilding the editor, terminal, Git client, Worktree manager, ACP runtime,
  conversation parser, or existing delegation UI.
- Fully autonomous Spec creation, release, deployment, or high-risk approval.
- A universal semantic code graph in the first repository-impact Feature.
- Cross-project memory sharing or automatic global-rule mutation.
- Automatic Skill activation from a threshold alone.
- Renaming inherited CodeG compatibility identifiers.

## 10. Assumptions

- The active WorkTask folder identifies one local Git repository root.
- The first implementation is local-first and single-user; standalone-server
  token authentication represents the user for human actions.
- Feature approval, exact Test Spec binding, and implementation Issues are
  separate delivery gates; a draft Spec does not authorize code changes.
