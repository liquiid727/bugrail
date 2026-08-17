---
id: issue-069
title: "Experience-to-Skill candidate lifecycle"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-014
sourceSpecVersion: "0.1"
sourceSpecHash: "b909465f96a945391b7ac02dae776ef01c56d5546743e21c5e6cc7d1eddaf1d7"
requirements: [BUGRAIL-SPECOS-014.R01]
dependsOn: [issue-065, issue-067]
---

# Experience-to-Skill candidate lifecycle

## Outcome

Separate execution traces, Experience, patterns and Skill candidates; require repeated independent evidence.

## Scope

Reuse existing ACP Skill operations only after validation and human approval; version activation and rollback.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

