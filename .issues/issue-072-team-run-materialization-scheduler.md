---
id: issue-072
title: "Materialize and schedule Team Workflow runs"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-015
sourceSpecVersion: "0.1"
sourceSpecHash: "cc08a8aa814ea9dfd0658ceb8d0f4a625a960a0f2f8ffd3c4104f8363a9a783f"
requirements: [BUGRAIL-SPECOS-015.R01]
dependsOn: [issue-046, issue-048, issue-071]
---

# Materialize and schedule Team Workflow runs

## Outcome

Snapshot a Workflow, create normal WorkTasks/node bindings/dependencies and launch ready nodes within concurrency.

## Scope

Keep WorkTask status authoritative and expose partial launch failures without fabricating success.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

