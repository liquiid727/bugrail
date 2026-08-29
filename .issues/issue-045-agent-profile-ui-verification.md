---
id: issue-045
title: "Agent profile UI and independent verification"
status: pending_verification
kind: verification
sourceSpecId: BUGRAIL-SPECOS-002
sourceSpecVersion: "0.2"
sourceSpecHash: "f9342aa89f379719d89fc8bcbcc7e612208652ec42fa9dd464c671185587fee9"
requirements: []
dependsOn: [issue-043, issue-044]
---

# Agent profile UI and independent verification

## Outcome

Make expert identities editable/inspectable and verify Feature 002 against its Test Spec.

## Scope

Cover no workspace, starter, validation, save/reload, same-model identities, legacy fallback, locales and transport parity.

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
- Spec hash recomputed: match `f9342aa89f379719d89fc8bcbcc7e612208652ec42fa9dd464c671185587fee9`
- Evidence: `tests/results/2026-08-13-specos-002-009-verification.md`
- Result: **not verified** (T05/T06 partial). Missing `agents.yaml` fails closed instead of legacy fallback. No live Axum/Tauri catalog roundtrip.
- Status remains `pending_verification`.

### 2026-08-28 reconciliation

- Deterministic repository tests were rerun; see
  `tests/results/2026-08-28-specos-approved-issue-reconciliation.md`.
- Status remains `pending_verification`: the live Axum/Tauri catalog roundtrip
  and independent Test Spec acceptance were not performed.
