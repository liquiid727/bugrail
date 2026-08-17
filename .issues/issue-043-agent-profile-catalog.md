---
id: issue-043
title: "Agent/model profile catalog and validation"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-002
sourceSpecVersion: "0.2"
sourceSpecHash: "f9342aa89f379719d89fc8bcbcc7e612208652ec42fa9dd464c671185587fee9"
requirements: [BUGRAIL-SPECOS-002.R01]
dependsOn: []
---

# Agent/model profile catalog and validation

## Outcome

Persist validated project Agent and Model profiles with atomic, symlink-safe YAML writes.

## Scope

Add DTO/schema validation, duplicate/reference checks, defaults, Tauri/Axum catalog commands and compatibility fixtures.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

