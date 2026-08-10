---
id: issue-023
title: "Integrate impact snapshots with cache, Context Pack, and transports"
status: draft
kind: implementation
type: fullstack
priority: medium
sourceSpecId: BUGRAIL-SPECOS-006
sourceSpecVersion: "0.1"
sourceSpecHash: "ad1ac57268e3fed82c35e8c7f3b57eda1e29fca753eb6b2ad747e328ffe223f5"
requirements: [BUGRAIL-SPECOS-006.R04, BUGRAIL-SPECOS-006.R05]
dependsOn: [issue-019, issue-022]
---

# Integrate Impact Snapshots With Cache, Context Pack, And Transports

## Outcome

Runs reuse revision-keyed analysis safely, record immutable snapshot metadata,
feed optional context candidates, and expose stored results after restart.

## Scope

- Add repository-revision cache under BugRail cache storage, not project/SQLite.
- Bind immutable snapshot metadata/hash to the run Context Pack.
- Preserve impact and budget reasons for included/excluded candidates.
- Implement `work_task_impact_get` with Tauri/Axum parity and exact TS client DTO.
- Invalidate through revision key changes; cache loss affects performance only.

## Acceptance Criteria

- Same revision policy uses a valid cache hit; HEAD change creates a miss.
- In-flight runs retain recorded revision even when repository HEAD changes.
- Context security/budgets remain authoritative over selected impact nodes.
- Deleting/corrupting cache never changes task, gate, merge, or stored snapshot truth.
- Desktop/server return identical complete/partial/truncated metadata.

## Verification

Cache hit/miss/corruption, revision race, restart, Context integration, transport,
and no-cache correctness tests pass.
