---
id: BUGRAIL-SPECOS-019
version: "0.1"
title: "Team Goal Intake And Static Workflow Launch"
status: draft
changeType: team-intake
prd: ".prd/prd-agent-team-mode-roadmap.md"
design: "design/agent-team-mode-architecture.md"
adr: "design/adr/ADR-002-agent-profile-team-worktask.md"
dependsOn: [BUGRAIL-SPECOS-018]
---

# BUGRAIL-SPECOS-019: Team Goal Intake And Static Workflow Launch

## 1. Outcome

Let a user submit a project goal through Team Mode and launch a selected,
validated static Workflow while preserving the existing Teams and WorkTask
execution path.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-019.R01` | Intake records a bounded user goal, project folder, selected Workflow identity/version/hash, actor, and creation time. |
| `BUGRAIL-SPECOS-019.R02` | The selected Workflow is project-authored and validated before any WorkTask is created; `auto` selection is outside this Feature. |
| `BUGRAIL-SPECOS-019.R03` | Run materialization remains one transaction over `team_run`, ordinary WorkTasks, bindings, and dependencies. |
| `BUGRAIL-SPECOS-019.R04` | A client request ID makes repeated create/start submissions idempotent. |
| `BUGRAIL-SPECOS-019.R05` | Teams is the first supported intake surface; any composer entry is an application-level Team action, not an ACP Session mode. |
| `BUGRAIL-SPECOS-019.R06` | Goal and Workflow snapshot remain inspectable after restart without copying the entire project configuration into multiple stores. |

## 3. Existing Modules

- Extend `team_run` with the smallest immutable intake/snapshot facts.
- Reuse `.codeg/teams.yaml`, existing catalog validation and WorkTask creation.
- Extend the current Teams page and typed transport client.
- Link every projected node to existing Task Detail.

## 4. Interface

Add a Team create operation with `folder_id`, `workflow_id`, bounded `goal`, and
`request_id`. Keep the legacy `specos_team_run_start(folder_id, workflow_id)`
behavior available for existing clients.

## 5. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-019.AC01` | A valid goal and static Workflow create exactly one Team run and the expected ordinary WorkTasks. |
| `BUGRAIL-SPECOS-019.AC02` | Duplicate request IDs return the same run and never duplicate nodes. |
| `BUGRAIL-SPECOS-019.AC03` | Invalid or stale Workflow selection creates no partial run. |
| `BUGRAIL-SPECOS-019.AC04` | Refresh/reconnect restores the goal, Workflow snapshot and current WorkTask projections. |
| `BUGRAIL-SPECOS-019.AC05` | The intake is usable through Tauri and server transports and the responsive Teams UI. |

## 6. Non-Goals

No classification, Planner, dynamic DAG, automatic Team selection, or new ACP
mode is included.

