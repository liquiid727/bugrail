# PRD: Agent Team Mode Roadmap In BugRail

## Meta

- Status: product roadmap draft
- Date: 2026-08-21
- Product: Code: BugRail
- Existing baseline: `BUGRAIL-SPECOS-001` through `017`
- Architecture: `design/agent-team-mode-architecture.md`
- Decisions: ADR `001-003`

This PRD is an umbrella product document. It describes the intended user
experience and capability order but does not authorize implementation. A
versioned Feature Spec and matching Test Spec are required for every delivery
slice.

## 1. Product Decision

BugRail evolves its existing static Team Workflow into an inspectable,
controllable, evidence-backed delivery mode. Team Mode remains an extension of
WorkTask, TaskEngine, ACP, Session, Worktree, Context, SQLite, transport, and
Tasks/Teams UI modules.

It does not introduce a second task state machine, scheduler, Agent runtime,
event store, artifact database, Context control plane, or Worktree manager.

## 2. Current Baseline

The repository already provides:

- project Agent and Model Profiles;
- `.codeg/teams.yaml` static Team and Workflow definitions;
- static DAG validation and WorkTask materialization;
- WorkTask dependencies and Team concurrency predicates;
- Team pause, resume, and cancel controls;
- WorkTask run evidence, Context Packages, gates, handoff, and Worktrees;
- shared Tauri/Axum command-core behavior;
- Teams and Task Detail surfaces.

The baseline is implemented but still requires independent verification and
reliability hardening. It must not be described as a greenfield Team runtime.

## 3. Product Goals

### G-01 - Reliable static Team delivery

A user can select a validated project Workflow, submit a goal, observe its
WorkTasks, and control the run without a frontend session owning execution.

### G-02 - Proposed dynamic planning

A Planner may propose a machine-readable DAG. BugRail validates and snapshots
the proposal, and policy or a human decides whether it may materialize as
WorkTasks. Planner output is never runtime authority.

### G-03 - Verifiable completion

Team completion is derived from WorkTask state, Git truth, required gates, and
recorded evidence. Assistant self-report cannot satisfy a gate.

### G-04 - Selective execution context

Every WorkTask uses the existing Agent resolution and Context Package path.
Team Mode references Context Loadouts; it does not own retrieval or prompt
composition.

### G-05 - Controlled recovery

Retry, reassignment, pause, cancel, restart recovery, budget stops, permission
decisions, and route fallback are explicit and auditable.

### G-06 - Desktop and server parity

Tauri and standalone server clients cross the same command-core interface.
Events are refresh hints; persisted facts reconstruct state.

## 4. Product Principles

1. Workflow nodes are WorkTasks.
2. WorkTask status is the only node execution state.
3. Team run status is a projection plus minimal control facts.
4. Existing WorkTask commands remain the operational interface for node work.
5. Configuration is Git-trackable; runtime facts are stored in SQLite.
6. Planning proposes; deterministic runtime policy decides.
7. Evidence is immutable by generation and attributable to its producer.
8. Advanced governance is delivered independently so one capability cannot
   silently pretend another is enforced.

## 5. Primary User Flow

```text
project goal
  -> choose static Workflow or request a plan proposal
  -> validate and approve the execution snapshot
  -> materialize ordinary WorkTasks and dependencies
  -> existing TaskEngine launches ready work
  -> existing gates, Context, Worktrees and handoffs apply
  -> Team projection shows progress and blockers
  -> evidence-backed finalization
```

## 6. Capability Ownership

| Capability | Owning delivery slice |
|---|---|
| WorkTask contract and gates | `BUGRAIL-SPECOS-001` |
| Agent/Model profiles | `BUGRAIL-SPECOS-002` |
| Run evidence | `BUGRAIL-SPECOS-003` |
| Dependencies | `BUGRAIL-SPECOS-004` |
| Worktree integration and handoff | `BUGRAIL-SPECOS-005` |
| Context package and routing | `BUGRAIL-SPECOS-006` through `009` |
| Static Team Workflow | `BUGRAIL-SPECOS-015` |
| Team controls and node trace | `BUGRAIL-SPECOS-016` |
| Memory Provider | `BUGRAIL-SPECOS-017` |
| Static baseline hardening | `BUGRAIL-SPECOS-018` |
| Goal intake and static launch | `BUGRAIL-SPECOS-019` |
| Dynamic planning proposal | `BUGRAIL-SPECOS-020` |
| Team quality and finalization | `BUGRAIL-SPECOS-021` |
| Retry and reassignment | `BUGRAIL-SPECOS-022` |
| Effective permissions | `BUGRAIL-SPECOS-023` |
| Usage and budgets | `BUGRAIL-SPECOS-024` |
| Provider/model fallback | `BUGRAIL-SPECOS-025` |
| Backend restart recovery | `BUGRAIL-SPECOS-026` |
| Notifications and remote operations | `BUGRAIL-SPECOS-027` |

## 7. Scope Boundaries

### Current static baseline

- validated project-authored DAGs;
- Agent WorkTask nodes;
- completion dependencies;
- bounded concurrency;
- existing WorkTask retry, cancel, review, merge, gates, and Worktrees;
- Team run controls and projection.

### Later capabilities

- goal intake from the composer;
- Planner-authored run proposals;
- reviewer and approval orchestration;
- team-aware retry/reassignment;
- enforceable permission intersection;
- token/cost budgets;
- provider/model fallback;
- restart recovery records;
- notifications and remote control.

### Non-goals

- a generic multi-agent framework;
- another Session, WorkTask, or scheduler runtime;
- arbitrary Agent-authored graph mutation after launch;
- direct provider SDK execution outside existing adapters;
- Team-owned Memory, Wiki, CodeGraph, Skill, or Context abstractions;
- mobile-specific UI in the core Team delivery Features;
- silent merge, permission escalation, model fallback, or destructive cleanup.

## 8. Release Strategy

Each Feature closes one observable user outcome and has its own Test Spec. A
later Feature may depend on a verified interface, but cannot make an unverified
earlier capability appear complete.

The release order is:

1. verify and harden the static baseline;
2. add a project-goal intake path for static Workflows;
3. add dynamic planning as a proposal and approval flow;
4. aggregate existing WorkTask quality into Team finalization;
5. add retry and reassignment semantics;
6. add permission, budget, and fallback independently;
7. add restart recovery, then notifications and remote operations.

## 9. Product Success

A mature Team Mode allows a user to answer:

- Which persisted WorkTasks make up this run?
- Why is each task ready, blocked, running, or complete?
- Which Agent/model/context snapshot did each generation use?
- What changed, what passed, and what still needs approval?
- What happened during retry, reassignment, fallback, cancellation, or restart?
- Can the final result be accepted from persisted evidence alone?

No single all-capabilities Definition of Done exists. Completion is evaluated
per Feature/Test Spec and rolled up through `.features/roadmap.md`.

