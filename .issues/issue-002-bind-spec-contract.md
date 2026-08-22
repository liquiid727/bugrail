---
id: issue-002
title: "Bind and validate a Feature Spec on WorkTask"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-001
sourceSpecVersion: "0.3"
sourceSpecHash: "79160488f65ae762decaa6db4987c15a783f61c886588c1e9157fc1bb40ab0d0"
requirements: [BUGRAIL-SPECOS-001.R01, BUGRAIL-SPECOS-001.R02, BUGRAIL-SPECOS-001.R03, BUGRAIL-SPECOS-001.R10]
dependsOn: [issue-001]
---

# Bind And Validate A Feature Spec On WorkTask

## Scope

- Implement the internal repository-local Spec reader.
- Add preview/bind/get command-core functions and Tauri/Axum transport parity.
- Validate canonical path, file size, ID/version/hash, AC selection, and gate
  policy limits.
- Preview parses identity/hash/AC without mutation; bind accepts selected AC IDs
  and the preview hash, then resolves authoritative AC text server-side.
- Record explicit bind/rebind events transactionally.

## Existing Modules

- `src-tauri/src/commands/work_task.rs`
- `src-tauri/src/web/handlers/work_task.rs`
- `src-tauri/src/web/router.rs`
- `src/lib/types.ts`
- `src/lib/api.ts`

## Acceptance

- Feature Test cases `T01-T09`, `T25-T28`, and relevant migration/transaction
  checks pass in focused tests.
- No path outside the active project root can be read through this command.
