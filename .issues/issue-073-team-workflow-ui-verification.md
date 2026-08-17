---
id: issue-073
title: "Team DAG UI and independent verification"
status: pending_verification
kind: verification
sourceSpecId: BUGRAIL-SPECOS-015
sourceSpecVersion: "0.1"
sourceSpecHash: "cc08a8aa814ea9dfd0658ceb8d0f4a625a960a0f2f8ffd3c4104f8363a9a783f"
requirements: []
dependsOn: [issue-071, issue-072]
---

# Team DAG UI and independent verification

## Outcome

Add first-level Teams navigation, profiles, semantic DAG list, start action and node/task drill-down.

## Scope

Verify sequential/parallel readiness, cycle rejection, concurrency, restart, starter/empty/error, locales and parity.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

