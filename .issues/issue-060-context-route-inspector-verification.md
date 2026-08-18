---
id: issue-060
title: "Context route, inspector and independent verification"
status: pending_verification
kind: verification
sourceSpecId: BUGRAIL-SPECOS-009
sourceSpecVersion: "0.2"
sourceSpecHash: "0eb4d34c5db0677a58fafff40c9c963359455ea3d1750d4bb52fc53e707913a7"
requirements: []
dependsOn: [issue-059]
---

# Context route, inspector and independent verification

## Outcome

Add first-level Context navigation and task package drill-down with complete interaction states.

## Scope

Verify no workspace/loading/empty/ready/degraded/blocked/error, last-good refresh, accessibility, responsiveness and all locales.

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
- Spec hash recomputed: match `0eb4d34c5db0677a58fafff40c9c963359455ea3d1750d4bb52fc53e707913a7`
- Evidence: `tests/results/2026-08-13-specos-002-009-verification.md`
- QA decision: **blocked**. Overview/provenance/join/last-good/activity pass. T06 has locale key parity only — no keyboard/responsive browser evidence.
- Status remains `pending_verification`.

