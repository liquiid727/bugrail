---
id: issue-065
title: "Evidence-qualified run evaluation"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-012
sourceSpecVersion: "0.1"
sourceSpecHash: "c43a38ce9dd0315d0fe1ffcef8ff3b4154eaa6645db74327586a08fbef71780d"
requirements: [BUGRAIL-SPECOS-012.R01]
dependsOn: [issue-047, issue-064]
---

# Evidence-qualified run evaluation

## Outcome

Project run, route, gate, token, review and Git evidence into idempotent qualified facts.

## Scope

Preserve unknown/pending values and exclude stale/incomplete legacy cohorts from strict metrics by default.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

