---
id: BUGRAIL-SPECOS-027
version: "0.1"
title: "Team Notifications And Remote Operations"
status: draft
changeType: team-remote-operations
prd: ".prd/prd-agent-team-mode-roadmap.md"
design: "design/agent-team-mode-architecture.md"
dependsOn: [BUGRAIL-SPECOS-019, BUGRAIL-SPECOS-021, BUGRAIL-SPECOS-023, BUGRAIL-SPECOS-024, BUGRAIL-SPECOS-026]
---

# BUGRAIL-SPECOS-027: Team Notifications And Remote Operations

## 1. Outcome

Notify users about durable Team outcomes and allow authenticated remote clients
to inspect and perform the same bounded control/approval operations as local
clients.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-027.R01` | Notifications derive from durable state transitions for completion, failure, approval/permission need, budget stop, conflict and recovery need. |
| `BUGRAIL-SPECOS-027.R02` | Delivery is deduplicated by durable transition identity; reconnect or replay cannot create repeated user-visible notifications. |
| `BUGRAIL-SPECOS-027.R03` | Remote reads return persisted Team/WorkTask projections and never require replaying all live events. |
| `BUGRAIL-SPECOS-027.R04` | Remote pause/resume/cancel/approval operations use the same command-core preconditions, authorization and audit as local operations. |
| `BUGRAIL-SPECOS-027.R05` | EventEmitter/WebSocket remains a refresh channel; reconnect performs an authoritative fetch and tolerates lost or repeated events. |
| `BUGRAIL-SPECOS-027.R06` | Notification payloads exclude secrets, raw protected context, full prompts and unbounded logs. |

## 3. Existing Modules

- Reuse EventEmitter, WebSocket transport, server authentication and desktop
  notification modules.
- Add a delivery receipt only when required for cross-restart deduplication.
- Extend existing Teams/Task Detail responsive states; do not create a separate
  mobile application in this Feature.

## 4. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-027.AC01` | One durable approval-needed transition produces at most one notification per configured channel/disposition. |
| `BUGRAIL-SPECOS-027.AC02` | WebSocket loss/replay followed by reconnect restores correct state without duplicate control or notification. |
| `BUGRAIL-SPECOS-027.AC03` | Unauthorized remote mutation is denied without changing Team or WorkTask facts. |
| `BUGRAIL-SPECOS-027.AC04` | Authorized local and remote controls produce equivalent outcomes and audit records. |
| `BUGRAIL-SPECOS-027.AC05` | Narrow responsive layouts expose core status and approval actions without requiring graph interaction. |

## 5. Non-Goals

Native mobile packaging, offline mutation queues and cross-project sharing are
separate product decisions.

