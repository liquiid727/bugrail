---
id: issue-062
title: "Impact inspector and independent verification"
status: planned
kind: verification
sourceSpecId: BUGRAIL-SPECOS-010
sourceSpecVersion: "0.2"
sourceSpecHash: "ce2c6b4f1261366ff46f52890b10e0d7529373f0de745331e4246ec0645af7fe"
requirements: []
dependsOn: [issue-061]
---

# Impact inspector and independent verification

## Outcome

Feed optional impact candidates into Context Package and expose relationships/reasons.

## Scope

Verify budgets, staleness, cache/restart, path security, degraded analyzers, UI and transports.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

