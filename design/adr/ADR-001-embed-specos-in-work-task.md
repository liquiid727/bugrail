# ADR-001: Embed SpecOS Delivery Control In WorkTask

- Status: accepted
- Date: 2026-08-09
- Accepted: 2026-08-23 (explicit user approval)
- Design: `design/specos-control-plane-design.md`

## Context

BugRail inherits a mature CodeG WorkTask engine, ACP runtime, Worktree flow,
SQLite persistence, Tauri/Axum transports, live event adapters, and Tasks UI.
The archived Control Plane V0 RFC proposed parallel Artifact, Workflow, Event,
Runtime, and Storage modules before mapping these capabilities to the code.

The first SpecOS behavior needs to bind tasks to specifications and prevent
unsafe merge or completion. That behavior crosses the invariants already owned
by WorkTask.

## Decision

SpecOS delivery control is implemented by deepening the existing WorkTask
module. The first Feature adds a task contract, structured gate attempts, and an
explainable merge/complete decision behind existing WorkTask commands.

ACP runtime, Git Worktree behavior, SQLite, `work_task_event`, EventEmitter, and
the Tasks UI remain the production modules. No parallel workflow engine,
generic runtime provider, event store, artifact database, or plugin registry is
introduced.

## Consequences

- Current users and clients retain the WorkTask state model and command names.
- Gate enforcement is centralized where merge and completion already occur.
- Runtime delivery state remains in SQLite; normative Specs remain in Git.
- Later DAG and context features must extend WorkTask or justify a new deep
  module from concrete behavior.
- Internal policies can be tested independently without becoming public plugin
  interfaces.

## Alternatives Considered

### Separate SpecOS process over a new Control API

Rejected for the first slice. It would duplicate orchestration and introduce
distributed consistency before a remote deployment requirement exists.

### New generic Artifact/Workflow kernel inside BugRail

Rejected for the first slice. The proposed interface duplicated WorkTask state,
SQLite records, event handling, and runtime coordination.

### Store all delivery artifacts in SQLite

Rejected. Git-tracked Feature/Test Specs need human review and exact revision
binding. SQLite stores live references and evidence, not a second editable copy.
