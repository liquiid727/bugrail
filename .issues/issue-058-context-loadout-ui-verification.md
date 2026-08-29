---
id: issue-058
title: "Loadout UI, security and independent verification"
status: pending_verification
kind: verification
sourceSpecId: BUGRAIL-SPECOS-008
sourceSpecVersion: "0.2"
sourceSpecHash: "244f55f97115b3b067dc7fc031ee55bdf26879f30dcbc3a2d3b5ade24174046d"
requirements: []
dependsOn: [issue-056, issue-057]
---

# Loadout UI, security and independent verification

## Outcome

Expose loadout sources/budgets and verify project boundary, immutability and prompt integration.

## Scope

Cover symlink/path escape, UTF-8, dedupe, caps, precedence, retries, locales and desktop/server parity.

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
- Spec hash recomputed: match `244f55f97115b3b067dc7fc031ee55bdf26879f30dcbc3a2d3b5ade24174046d`
- Evidence: `tests/results/2026-08-13-specos-002-009-verification.md`
- Command-core T01-T06: pass. QA decision: **blocked** (no live Axum/Tauri parity, no real ACP prompt-dispatch evidence).
- Status remains `pending_verification`.

### 2026-08-28 reconciliation

- Deterministic Context/loadout regressions were rerun; see
  `tests/results/2026-08-28-specos-approved-issue-reconciliation.md`.
- Status remains `pending_verification`: live Axum/Tauri parity, real ACP
  prompt dispatch, and independent Test Spec evidence remain outstanding.
