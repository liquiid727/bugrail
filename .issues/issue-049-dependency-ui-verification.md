---
id: issue-049
title: "Dependency trace UI and independent verification"
status: pending_verification
kind: verification
sourceSpecId: BUGRAIL-SPECOS-004
sourceSpecVersion: "0.2"
sourceSpecHash: "81ee40fe121cef77cb45120768e949a06e5883ea5b83cb64f6106a29fb8a9d4d"
requirements: []
dependsOn: [issue-048]
---

# Dependency trace UI and independent verification

## Outcome

Show dependency edges and readiness in the run trace and verify scheduler behavior.

## Scope

Cover ready/blocked/failed parents, parallel claims, restart, accessible list fallback and transport parity.

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
- Spec hash recomputed: match `f22459922841ca0a1136b6337f363753817b1d49c3657e3adbeddc15f8aef775`
- Evidence: `tests/results/2026-08-13-specos-002-009-verification.md`
- Result: **not verified** at first run (`t05_concurrency_race_cycle` failed; start did not check readiness). Follow-up 2026-08-13: dependency writes are serialized, user/auto claims return `workTask.dependency.unmet`, Feature Spec 004 approved for T01-T06. Re-run the Test Spec before flipping this Issue.
- Status remains `pending_verification`.

- Date: `2026-08-21`
- Evidence: `tests/results/2026-08-21-specos-004-verification.md`
- Result: T01-T06 are now green, including the direct manual-start regression
  and 004 traceability-panel UI test.
- Status remains `pending_verification`: repository-level Rust and TypeScript
  verification is blocked by out-of-scope Memory 017 and 005 Integration UI
  failures, so the Test Spec's required evidence is not fully green.

### 2026-08-28 reconciliation

- Deterministic WorkTask/UI regressions were rerun; see
  `tests/results/2026-08-28-specos-approved-issue-reconciliation.md`.
- Status remains `pending_verification`: the exact Test Spec still requires
  live transport and independent acceptance evidence.
