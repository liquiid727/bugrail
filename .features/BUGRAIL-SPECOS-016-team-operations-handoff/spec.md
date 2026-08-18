---
id: BUGRAIL-SPECOS-016
version: "0.1"
title: "Team Operations And Handoff"
status: approved
changeType: agent-team-context-deepening
prd: ".prd/prd-specos-agent-team-context-system.md"
design: "design/specos-control-plane-design.md"
clientDesign: "design/specos-client-interaction-design.md"
dependsOn: [BUGRAIL-SPECOS-005, BUGRAIL-SPECOS-009, BUGRAIL-SPECOS-015]
---

# BUGRAIL-SPECOS-016: Team Operations And Handoff

## 1. Summary

This vertical slice adapts the two 2026-08-12 source proposals to BugRail's
existing WorkTask/ACP/Worktree/SQLite/transport architecture. It deepens those
modules rather than introducing a parallel runtime.

## 2. Requirements

| ID | Requirement |
|---|---|
| BUGRAIL-SPECOS-016.R01 | A Team run supports pause, resume, and cancel; pause prevents new claims while active nodes keep existing WorkTask semantics. |
| BUGRAIL-SPECOS-016.R02 | Cancel delegates to existing WorkTask cancellation and records durable Team control state/outcomes. |
| BUGRAIL-SPECOS-016.R03 | Every node links to task detail, run resolution, Session/Worktree when present, Context Package, contract/gates, dependencies, and handoff. |
| BUGRAIL-SPECOS-016.R04 | A task generation may save one bounded structured handoff containing summary, artifacts, risks, and open questions. |
| BUGRAIL-SPECOS-016.R05 | Handoff is a durable artifact reference between nodes; entire conversation history is never implicitly copied. |
| BUGRAIL-SPECOS-016.R06 | Desktop and standalone-server Team operations have equivalent authorization, errors, refresh behavior and recovery. |

## 3. Architecture And Placement

Team control updates team_run.control_state and reuses current WorkTask command-core cancellation/pump behavior. Handoffs are keyed by task/run and shown through task traceability. No ownership transfer runtime is introduced.

### Command/API contract

specos_team_run_control(run_id, pause|resume|cancel), specos_work_task_handoff_get/save, plus existing WorkTask commands for retry/review/merge.

### Data and migration

New runtime facts use additive SeaORM migrations with foreign keys and indexed
lookup keys. Git-trackable definitions remain files; existing task rows and
legacy configuration are not rewritten. Down migration drops dependent rows
before parent projections.

## 4. Client Interaction

Run cards expose pause/resume/cancel, node status and task drill-down. Task detail edits handoff fields and shows the exact saved generation. Loading, partial failure and transport recovery keep last-good facts.

Backend projections are authoritative. UI disablement is never enforcement,
and live events are refresh hints only.

## 5. Failure, Security And Compatibility

Trusted-user control path, non-empty bounded summary, bounded lists/messages, folder/run ownership checks, idempotent/recoverable cancellation.

Errors are typed and leave the previous valid definition or WorkTask state
unchanged. Existing unprofiled tasks, ACP adapters, commands and routes remain
compatible.

## 6. Acceptance Criteria

| ID | Criterion |
|---|---|
| BUGRAIL-SPECOS-016.AC01 | Pause stops new node claims and resume pumps ready work. |
| BUGRAIL-SPECOS-016.AC02 | Cancel uses existing task cancellation and remains recoverable after restart. |
| BUGRAIL-SPECOS-016.AC03 | Structured handoff round-trips with exact task/run attribution. |
| BUGRAIL-SPECOS-016.AC04 | No Team operation bypasses WorkTask contract/gate/merge rules. |
| BUGRAIL-SPECOS-016.AC05 | Tauri/Axum and responsive/localized UI states are equivalent. |

## 7. Verification

The matching test-spec.md independently covers schema/validation, happy, error,
edge, concurrency/restart, security, legacy, Tauri/Axum parity, and UI states.
Completion requires persisted or command-captured evidence; implementation
output alone is not verification.

