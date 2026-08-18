---
id: issue-052
title: "Deterministic Context Package compiler"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-006
sourceSpecVersion: "0.2"
sourceSpecHash: "9a97e6eaecf1248ec4494b17f484f687628d13d27e8bda68d164e6e9755afabb"
requirements: [BUGRAIL-SPECOS-006.R01]
dependsOn: [issue-046, issue-050]
---

# Deterministic Context Package compiler

## Outcome

Compile ordered project-local sources into an immutable, hashed and budgeted package before ACP spawn.

## Scope

Canonicalize paths, reject escape/binary/oversize required sources, deduplicate hashes and persist package/items atomically.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

