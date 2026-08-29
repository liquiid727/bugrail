---
id: issue-047
title: "Run inspector and independent verification"
status: pending_verification
kind: verification
sourceSpecId: BUGRAIL-SPECOS-003
sourceSpecVersion: "0.2"
sourceSpecHash: "6dcbc16b16a05b5d98fcadd7393098558d2fbe2d334d40af4ed4672b5df38009"
requirements: []
dependsOn: [issue-046]
---

# Run inspector and independent verification

## Outcome

Expose run history in Task Detail and verify restart-safe generation attribution.

## Scope

Test claim/retry/return/merge generations, legacy data, interrupted transactions, typed clients and empty/error UI.

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
- Agent: `testing-agent`
- Spec hash recomputed: match `7dd9483c9193323ffe22b37c4b375056af0a556e6b927d502294373b22657492`
- Evidence: `tests/results/2026-08-13-specos-002-009-verification.md`
- Command-core T01-T06: pass. Feature Spec 003 is now **approved** for that slice. Re-run the Test Spec (including live transport) before flipping this Issue.
- Status remains `pending_verification`.

### 2026-08-28 reconciliation

- Deterministic WorkTask/UI regressions were rerun; see
  `tests/results/2026-08-28-specos-approved-issue-reconciliation.md`.
- Status remains `pending_verification`: live transport and independent Test
  Spec acceptance were not performed.
