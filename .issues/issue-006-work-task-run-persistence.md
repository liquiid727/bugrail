---
id: issue-006
title: "Persist one durable record per WorkTask run generation"
status: superseded
kind: implementation
type: backend
priority: high
sourceSpecId: BUGRAIL-SPECOS-002
replacementSpecId: BUGRAIL-SPECOS-003
supersededBy: [issue-046, issue-047]
sourceSpecVersion: "0.1"
sourceSpecHash: "4d2c623f35a9caea2d66aee7f716bb8ca41c451a1c1d33b92c763e6dcda87965"
requirements: [BUGRAIL-SPECOS-002.R01, BUGRAIL-SPECOS-002.R02]
dependsOn: [issue-001]
---

# Persist One Durable Record Per WorkTask Run Generation

## Outcome

Every claimed `run_seq` has one restart-safe `work_task_run` row, and new
WorkTask events are attributable to that generation without guessing legacy data.

## Scope

- Add the ordered migration, entity, indexes, Rust DTOs, and repository methods.
- Insert the run row in the existing claim transaction before launch effects.
- Add nullable `run_seq` to WorkTask events and populate it for current writers.
- Attach effective redacted config before prompt dispatch and settlement/Git
  facts in the existing transition transaction.
- Preserve resume/retry/return/merge generation semantics and legacy rows.

## Existing Modules

- `src-tauri/src/db/migration/`
- `src-tauri/src/db/entities/`
- `src-tauri/src/db/service/work_task_service.rs`
- `src-tauri/src/work_task/engine.rs`
- `src-tauri/src/models/work_task.rs`

## Acceptance Criteria

- Duplicate `(task_id, run_seq)` fails as a consistency error, never an upsert.
- Start, retry, return, and merge generations bind to the correct row/event set.
- Process interruption cannot commit a claim without its required run row.
- Existing events remain nullable and are not backfilled with invented runs.
- Stored config contains only allowlisted Agent/model/mode/policy identifiers.

## Verification

Migration up/down, claim concurrency, settlement rollback, restart, redaction,
and legacy fixture tests pass in the Rust test suite.
