---
id: issue-051
title: "Integration/handoff UI and independent verification"
status: pending_verification
kind: verification
sourceSpecId: BUGRAIL-SPECOS-005
sourceSpecVersion: "0.2"
sourceSpecHash: "1d5ff5e900247259bab0b2ad292246fb92dbcccc7e089b5295425ecab2c47678"
requirements: []
dependsOn: [issue-050]
---

# Integration/handoff UI and independent verification

## Outcome

Expose handoff editing/inspection and verify integration eligibility and Git-truth behavior.

## Scope

Cover missing/stale handoff, conflicts, source heads, gates, legacy summaries, desktop/server and error states.

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
- Spec hash recomputed: match `99bb0dc5d494226570962a462e11177fa9c78f92e72bf6f8d3db877629b1d9d9`
- Evidence: `tests/results/2026-08-13-specos-002-009-verification.md`
- Result: **not verified** at first run (T03-T05 missing). Follow-up 2026-08-13: `integration_plan` / `refresh` / containment-gated `merge_landed` plus Git oracles for T03-T05. Feature Spec 005 approved for T01-T06. Re-run the Test Spec before flipping this Issue.
- Status remains `pending_verification`.

### 2026-08-21 follow-up

- Evidence: `tests/results/2026-08-21-specos-005-verification.md` at
  repository HEAD `3c7240c08184c28658330f15e5bcd08d35ee8c4d`.
- Result: T01-T06 command-core/SQLite/Git oracles and targeted integration UI
  transport states pass. Source direct completion, merge, cleanup, and
  auto-merge are now blocked while a live integration reserves the source;
  conflict inspection prefers the integration Worktree.
- Status remains `pending_verification`: real TaskEngine conflict
  resolution/retry, restart/idempotency, Axum/Tauri parity, and the full UI
  interaction contract have not been independently demonstrated.

### 2026-08-28 reconciliation

- Deterministic WorkTask/UI regressions were rerun; see
  `tests/results/2026-08-28-specos-approved-issue-reconciliation.md`.
- Status remains `pending_verification`: TaskEngine conflict/retry, restart,
  live transport parity, and independent UI acceptance remain outstanding.
