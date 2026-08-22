---
id: issue-001
title: "Add WorkTask contract and gate persistence"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-001
sourceSpecVersion: "0.3"
sourceSpecHash: "79160488f65ae762decaa6db4987c15a783f61c886588c1e9157fc1bb40ab0d0"
requirements: [BUGRAIL-SPECOS-001.R01, BUGRAIL-SPECOS-001.R03, BUGRAIL-SPECOS-001.R05, BUGRAIL-SPECOS-001.R09]
dependsOn: []
---

# Add WorkTask Contract And Gate Persistence

## Scope

- Add the ordered SeaORM migration for `work_task_contract` and
  `work_task_gate_result`.
- Add entities, Rust DTOs, TypeScript mirrors, repositories, and pure gate
  decision logic.
- Keep existing `work_task`, status values, and preflight columns unchanged.

## Existing Modules

- `src-tauri/src/db/migration/`
- `src-tauri/src/db/entities/`
- `src-tauri/src/db/service/work_task_service.rs`
- `src-tauri/src/models/work_task.rs`
- `src/lib/types.ts`

## Acceptance

- Migration up/down, FK cascade, indexes, limits, append-only attempts, actor
  rules, reusable results, and transactional timeline behavior are tested.
- No existing WorkTask row needs a data backfill.
