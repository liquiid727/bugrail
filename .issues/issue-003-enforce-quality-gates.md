---
id: issue-003
title: "Enforce gate decisions in WorkTask merge and completion"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-001
sourceSpecVersion: "0.3"
sourceSpecHash: "79160488f65ae762decaa6db4987c15a783f61c886588c1e9157fc1bb40ab0d0"
requirements: [BUGRAIL-SPECOS-001.R04, BUGRAIL-SPECOS-001.R05, BUGRAIL-SPECOS-001.R06, BUGRAIL-SPECOS-001.R07, BUGRAIL-SPECOS-001.R09, BUGRAIL-SPECOS-001.R10]
dependsOn: [issue-001, issue-002]
---

# Enforce Gate Decisions In WorkTask Merge And Completion

## Scope

- Record structured gate attempts and map existing preflight to the preflight
  gate for bound tasks.
- Add the explicit authenticated-human gate-decision command; do not expose a
  generic client-controlled gate-record command or actor field.
- Evaluate Spec staleness and current-run gate eligibility in existing merge
  and no-change completion command cores.
- Return typed errors, preserve `review` state, and record decision events when
  blocked.
- Preserve legacy task behavior and existing Git-truth merge invariants.

## Existing Modules

- `src-tauri/src/work_task/engine.rs`
- `src-tauri/src/db/service/work_task_service.rs`
- `src-tauri/src/commands/work_task.rs`
- `src-tauri/src/work_task/git.rs`

## Acceptance

- Feature Test cases `T10-T24` pass.
- No live event, Agent verdict, frontend button, or stale gate result can bypass
  backend enforcement.
