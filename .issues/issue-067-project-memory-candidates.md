---
id: issue-067
title: "Governed project memory candidates"
status: superseded
retiredOn: "2026-08-19"
retirementCommit: "e8c6d7332359a70206dfa55e7eb97f94ee86650f"
retirementReason: "Source Features 010-014 were withdrawn during the Memory plugin delivery migration"
supersededBy: []
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-013
sourceSpecVersion: "0.1"
sourceSpecHash: "8d04265fb1856a700a0b17eef31cc92da125858c5f2f84040fc291c08fe3d219"
requirements: [BUGRAIL-SPECOS-013.R01]
dependsOn: [issue-053, issue-065]
---

# Governed project memory candidates

## Outcome

Extract source-linked candidates from qualified evidence and explicit corrections, with conflict/dedupe lifecycle.

## Scope

Preview and atomically write accepted non-stale memory to canonical Git-tracked project Markdown only.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.
