---
id: issue-055
title: "Provider health UI and independent verification"
status: pending_verification
kind: verification
sourceSpecId: BUGRAIL-SPECOS-007
sourceSpecVersion: "0.2"
sourceSpecHash: "fc10e29a5c2be849a1573875aee7b3fb73f0292dcbfb5e23623c88318f6d669f"
requirements: []
dependsOn: [issue-054]
---

# Provider health UI and independent verification

## Outcome

Show Provider health/degradation and verify the adapter boundary and credential redaction.

## Scope

Cover healthy/disabled/degraded/required-blocked, timeout, retry, last-good display, logs/DTO storage and parity.

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
- Spec hash recomputed: match `fc10e29a5c2be849a1573875aee7b3fb73f0292dcbfb5e23623c88318f6d669f`
- Evidence: `tests/results/2026-08-13-specos-002-009-verification.md`
- QA decision: **blocked**. T01/T03-T06 pass; T02 has no successful remote health fixture. No live transport test.
- Status remains `pending_verification`.

### 2026-08-28 reconciliation

- Deterministic provider/Context regressions were rerun; see
  `tests/results/2026-08-28-specos-approved-issue-reconciliation.md`.
- Status remains `pending_verification`: successful remote health, live
  transport, and independent Test Spec evidence remain outstanding.
