# CodeG Module Map For SpecOS

> Observed baseline: BugRail commit `55545d43`
> Purpose: map SpecOS concepts to inherited CodeG modules before proposing code

## Existing Deep Modules

| SpecOS concern | Existing CodeG-derived module | Current behavior | First-slice decision |
|---|---|---|---|
| Task lifecycle and scheduling | `src-tauri/src/work_task/engine.rs` | Drives `todo -> queued -> preparing -> running <-> awaiting_input -> review -> merging -> done`; owns concurrency, recovery, Worktree and Session coordination. | Extend. Do not add another workflow engine. |
| Task persistence and audit | `db/entities/work_task.rs`, `db/entities/work_task_event.rs`, `db/service/work_task_service.rs` | SeaORM state, CAS transitions, run generations, append-only timeline written with transitions. | Extend with a task contract and gate results. |
| Git and Worktree truth | `src-tauri/src/work_task/git.rs` | Creates, compares, merges, and cleans Worktrees; merge completion is verified from Git truth. | Reuse without wrapping. |
| Agent runtime | `src-tauri/src/acp/manager.rs`, `connection.rs`, `registry.rs` | Starts and observes built-in and custom ACP agents, permissions, questions, and session state. | Reuse. Existing multiple agents form the real runtime seam. |
| Delegation | `src-tauri/src/acp/delegation/`, `commands/delegation.rs` | Spawns and tracks sub-agent work and live replies. | Reuse for later team Features. |
| In-process events | `src-tauri/src/acp/internal_bus.rs` | Typed broadcast for backend consumers; bounded and allowed to report lag. | Reuse for live reaction, not durable delivery evidence. |
| UI event transport | `src-tauri/src/web/event_bridge.rs` | Tauri, WebSocket, and Noop adapters. | Reuse as the existing transport seam. |
| Desktop/server command surface | `commands/work_task.rs`, `web/router.rs`, `web/handlers/work_task.rs` | Shared command-core behavior exposed through Tauri and Axum. | Extend existing command names and handlers. |
| Frontend task contract | `src/lib/types.ts`, `src/lib/api.ts` | TypeScript mirror and transport-independent client functions. | Extend in lockstep with Rust wire types. |
| Task interaction | `src/components/tasks/` and `src/contexts/tasks-view-context.tsx` | Board, details, timeline, preflight, review, merge, completion, retry, and takeover entrypoints. | Extend Task Detail and existing dialogs. |
| Current quality signal | `WorkTaskPreflight`, `verdict`, `task-acceptance.ts` | One configurable command result plus Agent verdict and Git diff checks. | Promote to structured gates while preserving legacy behavior. |
| Product storage | `src-tauri/src/db/` | SQLite/SeaORM migrations, WAL, foreign keys, pooled runtime connections. | SQLite remains runtime truth; Git Markdown remains delivery-spec truth. |

## Authority Split

| Information | Authority |
|---|---|
| Product intent, design, Feature/Test Specs, Issues | Git-tracked Markdown under the paths in `.specos/manifest.yaml` |
| Live WorkTask state, gate attempts, Session/Worktree references | BugRail SQLite through SeaORM |
| Code changes and merge truth | Git repository and Worktree state |
| Live ACP progress | ACP session state and the typed in-process bus |
| Durable task timeline | `work_task_event` and related task tables |
| Release evidence | normalized files under `tests/results/` plus review records |

The first slice stores only an immutable reference and acceptance snapshot from
the Feature Spec in SQLite. It does not copy the entire Markdown artifact graph
into a second runtime artifact store.

## Compatibility Names

The product name is `Code: BugRail`. Existing `codeg` binaries, command names,
HTTP routes, URI schemes, database filenames, and `CODEG_*` environment
variables remain inherited compatibility identifiers until a dedicated
compatibility Feature Spec defines aliases, rollout, and rollback.

## Later Capability Placement

| Later capability | Preferred placement |
|---|---|
| Task dependencies/DAG | Deepen WorkTask scheduling and persistence. |
| Context Pack | Internal WorkTask run-preparation module using current prompt hydration and code/file access. |
| Risk policy | Pure internal policy consumed by gate selection; no external seam initially. |
| Review | Reviewer Agent Profile executed through the existing ACP runtime. |
| Model routing | Internal policy over existing model/provider registries. |
| Eval | Projection over durable WorkTask/Session/gate evidence. |
| Memory and Skill evolution | Later modules with explicit evidence sources and lifecycle; no M0 placeholders. |
| Code intelligence | A real seam only when both an in-process implementation and a remote/external adapter are justified. |

