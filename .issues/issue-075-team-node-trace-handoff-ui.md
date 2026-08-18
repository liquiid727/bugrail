---
id: issue-075
title: "Team node trace and handoff interaction"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-016
sourceSpecVersion: "0.1"
sourceSpecHash: "67a4fcb4682ca22c2994c7114376887e9e15a12c6b8daf54137ba051a47e7517"
requirements: [BUGRAIL-SPECOS-016.R01]
dependsOn: [issue-050, issue-059, issue-073, issue-074]
---

# Team node trace and handoff interaction

## Outcome

Link Team nodes to task/run/Session/Worktree/Context/contract/gates and editable structured handoff.

## Scope

Preserve last-good data during refresh errors and never infer authority from UI state.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

