---
id: issue-059
title: "Context Overview and activity projection"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-009
sourceSpecVersion: "0.2"
sourceSpecHash: "0eb4d34c5db0677a58fafff40c9c963359455ea3d1750d4bb52fc53e707913a7"
requirements: [BUGRAIL-SPECOS-009.R01]
dependsOn: [issue-054, issue-057]
---

# Context Overview and activity projection

## Outcome

Query validated config, Provider health, recent packages and bounded activity through shared command core.

## Scope

Persist attributable package/provider activity and keep correctness independent of live events.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

