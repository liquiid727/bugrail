---
id: issue-044
title: "Resolve and snapshot Agent runtime"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-002
sourceSpecVersion: "0.2"
sourceSpecHash: "f9342aa89f379719d89fc8bcbcc7e612208652ec42fa9dd464c671185587fee9"
requirements: [BUGRAIL-SPECOS-002.R01]
dependsOn: [issue-043]
---

# Resolve and snapshot Agent runtime

## Outcome

Resolve profile/model/mode/reasoning into the existing ACP adapter and persist a redacted immutable run decision.

## Scope

Apply explicit precedence, legacy fallback and reason codes before prompt dispatch; never call a model Provider directly.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

