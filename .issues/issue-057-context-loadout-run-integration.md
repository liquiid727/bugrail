---
id: issue-057
title: "Integrate loadouts with WorkTask launch"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-008
sourceSpecVersion: "0.2"
sourceSpecHash: "244f55f97115b3b067dc7fc031ee55bdf26879f30dcbc3a2d3b5ade24174046d"
requirements: [BUGRAIL-SPECOS-008.R01]
dependsOn: [issue-056]
---

# Integrate loadouts with WorkTask launch

## Outcome

Compile the selected loadout before ACP spawn and bind its immutable package to the current run.

## Scope

Block required failures, record optional degradation and preserve legacy prompt/session/worktree semantics.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

