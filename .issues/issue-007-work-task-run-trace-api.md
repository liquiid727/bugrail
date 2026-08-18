---
id: issue-007
title: "Project and expose bounded WorkTask run traces"
status: superseded
kind: implementation
type: fullstack
priority: high
sourceSpecId: BUGRAIL-SPECOS-002
replacementSpecId: BUGRAIL-SPECOS-003
supersededBy: [issue-046, issue-047]
sourceSpecVersion: "0.1"
sourceSpecHash: "4d2c623f35a9caea2d66aee7f716bb8ca41c451a1c1d33b92c763e6dcda87965"
requirements: [BUGRAIL-SPECOS-002.R03, BUGRAIL-SPECOS-002.R04]
dependsOn: [issue-006, issue-003]
---

# Project And Expose Bounded WorkTask Run Traces

## Outcome

Desktop and server clients can list generations and load one authoritative trace
assembled from run, event, Conversation, token, gate, and Git facts.

## Scope

- Implement `work_task_run_list` with `run_seq DESC` cursor pagination.
- Implement `work_task_run_trace` with event pagination after 500 rows.
- Project pending/unknown token state and `legacy_unscoped` explicitly.
- Add Tauri registration, Axum route/handler parity, TypeScript DTO mirrors, and
  typed `src/lib/api.ts` functions.
- Enforce folder/task authorization and omit raw prompts, transcripts, secrets,
  environment values, and uncapped command output.

## Acceptance Criteria

- A restart returns the same trace ordering and source attribution.
- Missing token sync remains `pending_sync` or null, never numeric zero.
- List defaults to 20 and rejects limits above 100; event pages are bounded.
- Tauri and Axum return identical shapes and stable not-found/inconsistent errors.
- Board/task-summary queries do not join or fetch run event history.

## Verification

Projection fixtures, pagination boundaries, authorization, redaction, desktop/
server contract snapshots, and query-count regression tests pass.
