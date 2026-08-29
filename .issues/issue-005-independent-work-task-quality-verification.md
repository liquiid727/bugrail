---
id: issue-005
title: "Independently verify Spec-linked WorkTask quality"
status: pending_verification
kind: verification
sourceSpecId: BUGRAIL-SPECOS-001
sourceSpecVersion: "0.3"
sourceSpecHash: "79160488f65ae762decaa6db4987c15a783f61c886588c1e9157fc1bb40ab0d0"
testSpecId: BUGRAIL-SPECOS-001.test
testSpecVersion: "0.3"
dependsOn: [issue-001, issue-002, issue-003, issue-004]
---

# Independently Verify Spec-Linked WorkTask Quality

## Scope

- Verify the Test Spec source hash before execution.
- Execute `T01-T31`, migration/rollback checks, compatibility regression, and
  the declared frontend/Rust command matrix.
- Retain raw outputs and normalize results under `tests/results/`.
- Review gate-bypass risk independently from the implementation context.

## Acceptance

- Every `AC01-AC10` has version-bound passing evidence.
- Missing, stale, partial, or implementation-owned evidence remains blocking.

## Verification Record

- Date: `2026-08-23` (loop gap-closure iteration)
- Evidence: `tests/results/2026-08-23-specos-001-gap-evidence/README.md`
  (raw command logs retained under its `logs/` directory)
- Closed the `T17` missing-producer engine fixture, the `T20`
  real-unchanged-worktree cleanup fixture, and the `T27` command-core
  atomicity fixture (`src-tauri/tests/specos_engine_gaps.rs`, 3 tests).
  Full matrix re-run green except `cargo fmt --check` on the untracked
  uncommitted 017 memory files.
- Status remains `pending_verification`: `T28` real Tauri invocations,
  `T29-T31` UI journeys/redaction/screenshots, the migration
  interrupted-transaction fixture, and a code-free independent verification
  run are still required against this Spec hash.

### 2026-08-28 reconciliation

- Evidence: `tests/results/2026-08-28-specos-approved-issue-reconciliation.md`.
- The Spec contract migration now has explicit transactional DDL and the real
  interruption fixture passes, closing that previously recorded gap.
- Status remains `pending_verification`: `T28-T31` and a code-free independent
  verification run are still required against the bound Test Spec.
