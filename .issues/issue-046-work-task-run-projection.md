---
id: issue-046
title: "Durable WorkTask run projection"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-003
sourceSpecVersion: "0.2"
sourceSpecHash: "6dcbc16b16a05b5d98fcadd7393098558d2fbe2d334d40af4ed4672b5df38009"
requirements: [BUGRAIL-SPECOS-003.R01]
dependsOn: [issue-044]
---

# Durable WorkTask run projection

## Outcome

Create exactly one run snapshot for each claimed generation and update it through current lifecycle transitions.

## Scope

Bind resolution, Session/Worktree, Context Package and terminal timestamps without copying transcripts or secrets.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

