---
id: issue-048
title: "Dependency store and readiness enforcement"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-004
sourceSpecVersion: "0.2"
sourceSpecHash: "81ee40fe121cef77cb45120768e949a06e5883ea5b83cb64f6106a29fb8a9d4d"
requirements: [BUGRAIL-SPECOS-004.R01]
dependsOn: [issue-046]
---

# Dependency store and readiness enforcement

## Outcome

Persist WorkTask dependency edges and prevent claims until parent tasks are done.

## Scope

Validate self/duplicate/cross-folder/cycle cases, concurrency and explicit block reasons while retaining WorkTask states.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

## Verification Record

- Date: `2026-08-21`
- Evidence: `tests/results/2026-08-21-specos-004-verification.md`
- Result: all approved 004 scenarios pass, including concurrent reverse-edge
  rejection and direct `TaskEngine::start` readiness enforcement.
- Status remains `implemented_pending_verification`: the required repository
  Rust suite currently fails in out-of-scope Memory 017 coverage and
  `pnpm exec tsc --noEmit` fails in out-of-scope 005 Integration UI i18n work.
