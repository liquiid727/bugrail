---
id: BUGRAIL-SPECOS-015
version: "0.1"
title: "Static Team Workflow"
status: approved
changeType: agent-team-context-deepening
prd: ".prd/prd-specos-agent-team-context-system.md"
design: "design/specos-control-plane-design.md"
clientDesign: "design/specos-client-interaction-design.md"
dependsOn: [BUGRAIL-SPECOS-002, BUGRAIL-SPECOS-003, BUGRAIL-SPECOS-004, BUGRAIL-SPECOS-008]
---

# BUGRAIL-SPECOS-015: Static Team Workflow

## 1. Summary

This vertical slice adapts the two 2026-08-12 source proposals to BugRail's
existing WorkTask/ACP/Worktree/SQLite/transport architecture. It deepens those
modules rather than introducing a parallel runtime.

## 2. Requirements

| ID | Requirement |
|---|---|
| BUGRAIL-SPECOS-015.R01 | A project Team is a named expert pool referencing enabled Agent Profiles; a Workflow separately defines version, Team, nodes, prompts, dependencies, and max concurrency. |
| BUGRAIL-SPECOS-015.R02 | Catalog validation rejects duplicate IDs, unknown profiles/Teams, missing nodes, self edges, cycles, and invalid concurrency before save/start. |
| BUGRAIL-SPECOS-015.R03 | Starting a Workflow snapshots its identity/hash and materializes each node as an ordinary WorkTask with profile/loadout/team identifiers. |
| BUGRAIL-SPECOS-015.R04 | Workflow edges become persisted WorkTask dependencies; ready nodes are claimed by the existing scheduler and never exceed Team max concurrency. |
| BUGRAIL-SPECOS-015.R05 | Team run/node state is reconstructed from persisted bindings plus current WorkTask facts without a second node state machine. |
| BUGRAIL-SPECOS-015.R06 | Partial launch failure is reported explicitly and never presented as a fully started Team run. |

## 3. Architecture And Placement

specos_control validates .codeg/teams.yaml; command core creates team_run, WorkTasks, team_run_task, and dependency edges. Existing next_queued evaluates dependency and Team predicates.

### Command/API contract

specos_team_catalog_get/save and specos_team_run_start/list. Start returns the projected Team run and node WorkTask references.

### Data and migration

New runtime facts use additive SeaORM migrations with foreign keys and indexed
lookup keys. Git-trackable definitions remain files; existing task rows and
legacy configuration are not rewritten. Down migration drops dependent rows
before parent projections.

## 4. Client Interaction

Teams shows Agent/Model profiles, Team membership, a semantic DAG list, start controls, current node states and validation errors. A starter definition is explicit user action.

Backend projections are authoritative. UI disablement is never enforcement,
and live events are refresh hints only.

## 5. Failure, Security And Compatibility

Version/hash workflow snapshot, validated prompts/IDs, bounded nodes/concurrency, folder ownership, no dynamic Agent-authored DAG in this slice.

Errors are typed and leave the previous valid definition or WorkTask state
unchanged. Existing unprofiled tasks, ACP adapters, commands and routes remain
compatible.

## 6. Acceptance Criteria

| ID | Criterion |
|---|---|
| BUGRAIL-SPECOS-015.AC01 | Sequential and parallel DAG fixtures launch only ready nodes. |
| BUGRAIL-SPECOS-015.AC02 | A cycle or unknown profile is rejected before any WorkTask is created. |
| BUGRAIL-SPECOS-015.AC03 | Concurrent active Team nodes never exceed the configured bound. |
| BUGRAIL-SPECOS-015.AC04 | Every node is inspectable and operable as an ordinary WorkTask. |
| BUGRAIL-SPECOS-015.AC05 | Restart reconstructs Team status from persisted bindings and task facts. |

## 7. Verification

The matching test-spec.md independently covers schema/validation, happy, error,
edge, concurrency/restart, security, legacy, Tauri/Axum parity, and UI states.
Completion requires persisted or command-captured evidence; implementation
output alone is not verification.

