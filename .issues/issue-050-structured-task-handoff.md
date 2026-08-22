---
id: issue-050
title: "Structured WorkTask handoff persistence"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-005
sourceSpecVersion: "0.2"
sourceSpecHash: "1d5ff5e900247259bab0b2ad292246fb92dbcccc7e089b5295425ecab2c47678"
requirements: [BUGRAIL-SPECOS-005.R01]
dependsOn: [issue-046, issue-048]
---

# Structured WorkTask handoff persistence

## Outcome

Store one bounded handoff per task generation with summary, artifacts, risks and open questions.

## Scope

Use trusted command core, exact run attribution and restart-safe get/save semantics; do not copy conversations.

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
- Evidence: `tests/results/2026-08-21-specos-005-verification.md`
- Result: correlated handoff command-core, SQLite/Git facts, legacy summary
  compatibility, and targeted UI/transport tests pass.
- Status remains `implemented_pending_verification`: restart, full transport
  parity, and independent end-to-end evidence remain incomplete.
