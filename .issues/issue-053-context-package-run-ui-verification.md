---
id: issue-053
title: "Bind, inspect and verify Context Packages"
status: pending_verification
kind: verification
sourceSpecId: BUGRAIL-SPECOS-006
sourceSpecVersion: "0.2"
sourceSpecHash: "9a97e6eaecf1248ec4494b17f484f687628d13d27e8bda68d164e6e9755afabb"
requirements: []
dependsOn: [issue-052]
---

# Bind, inspect and verify Context Packages

## Outcome

Bind a package to exact task/run, inject it into the prompt and expose it in Task Detail.

## Scope

Verify deterministic hashes, retry isolation, required blocking, optional absence, restart, prompt guards and UI states.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

## Verification Record

- Date: `2026-08-13`
- Agents: `testing-agent`, `qa-agent`
- Spec hash recomputed: match `1769705f83b91b410e8472c70f891b974b7062da0b8fd413fd5a4f138d5e1878`
- Evidence: `tests/results/2026-08-13-specos-002-009-verification.md`
- Command-core T01-T06: pass. Feature Spec 006 is now **approved** for that slice. QA still **blocked** on live transport / packaged-engine evidence.
- Status remains `pending_verification`.

### 2026-08-28 reconciliation

- Deterministic Context/WorkTask regressions were rerun; see
  `tests/results/2026-08-28-specos-approved-issue-reconciliation.md`.
- Status remains `pending_verification`: live transport, packaged-engine, and
  independent Test Spec evidence remain outstanding.
