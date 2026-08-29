---
id: issue-063
title: "Explainable Agent/model routing policy"
status: superseded
retiredOn: "2026-08-19"
retirementCommit: "e8c6d7332359a70206dfa55e7eb97f94ee86650f"
retirementReason: "Source Features 010-014 were withdrawn during the Memory plugin delivery migration"
supersededBy: []
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-011
sourceSpecVersion: "0.1"
sourceSpecHash: "be800ff77f52024ee316b144b3b0eaf2acfe617be01675c03decb0bd3a809590"
requirements: [BUGRAIL-SPECOS-011.R01]
dependsOn: [issue-046, issue-053]
---

# Explainable Agent/model routing policy

## Outcome

Score installed Agent/model candidates only when explicit profile/folder choices are absent.

## Scope

Persist candidates, disqualifications, scores, policy version, chosen route and safe fallback reason codes.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.
