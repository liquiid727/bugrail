---
id: issue-076
title: "Independently verify Team operations"
status: pending_verification
kind: verification
sourceSpecId: BUGRAIL-SPECOS-016
sourceSpecVersion: "0.1"
sourceSpecHash: "67a4fcb4682ca22c2994c7114376887e9e15a12c6b8daf54137ba051a47e7517"
requirements: []
dependsOn: [issue-074, issue-075]
---

# Independently verify Team operations

## Outcome

Verify pause/resume/cancel, handoff, node inspection, recovery and compatibility end to end.

## Scope

Cover partial failure, concurrent control calls, gate preservation, responsive/localized UI and Tauri/Axum parity.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

