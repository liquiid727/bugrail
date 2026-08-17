---
id: issue-064
title: "Route inspector and independent verification"
status: planned
kind: verification
sourceSpecId: BUGRAIL-SPECOS-011
sourceSpecVersion: "0.1"
sourceSpecHash: "be800ff77f52024ee316b144b3b0eaf2acfe617be01675c03decb0bd3a809590"
requirements: []
dependsOn: [issue-063]
---

# Route inspector and independent verification

## Outcome

Expose immutable route decisions and verify explicit-selection precedence and fallback boundaries.

## Scope

Test availability, provider errors, post-prompt fallback prohibition, permission/gate preservation and UI states.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

