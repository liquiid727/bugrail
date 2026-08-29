---
id: issue-070
title: "Skill lifecycle UI and independent verification"
status: superseded
retiredOn: "2026-08-19"
retirementCommit: "e8c6d7332359a70206dfa55e7eb97f94ee86650f"
retirementReason: "Source Features 010-014 were withdrawn during the Memory plugin delivery migration"
supersededBy: []
kind: verification
sourceSpecId: BUGRAIL-SPECOS-014
sourceSpecVersion: "0.1"
sourceSpecHash: "b909465f96a945391b7ac02dae776ef01c56d5546743e21c5e6cc7d1eddaf1d7"
requirements: []
dependsOn: [issue-069]
---

# Skill lifecycle UI and independent verification

## Outcome

Expose evidence, fixtures, validation comparison, approval, activation, degradation and rollback.

## Scope

Verify single-run rejection, conflict handling, no Agent self-approval, file safety, refresh and parity.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.
