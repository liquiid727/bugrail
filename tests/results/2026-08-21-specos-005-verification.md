# BUGRAIL-SPECOS-005 Verification Record

- Date: `2026-08-21`
- Feature/Test Spec: `BUGRAIL-SPECOS-005` / `0.2`
- Feature SHA-256: `1d5ff5e900247259bab0b2ad292246fb92dbcccc7e089b5295425ecab2c47678`
- Test Spec SHA-256: `7ed765c13a5c3de7c40347221162ef2408ba3058a7e642950e06ce603be7e99d`
- Repository HEAD: `3c7240c08184c28658330f15e5bcd08d35ee8c4d` (`main`)
- Database fixture: in-memory SQLite migrated through the current migration set;
  migration oracle is `tests/specos_migration.rs`.
- Git fixture: temporary repository `/tmp/bugrail-specos-005-zGdYyH`.
  `source-a=6645c38e9569b8d23b854669af28be4c9b9b75ef`,
  `source-b=05fdc0793e885973567f843ebe00baf018ff2988`, and
  `integration-landing=64d80f42dcc31b90caa94e5884369c53076efc74`.
  `git merge-base --is-ancestor source-a integration-landing` exited `0`.

## Commands

| Command                                                                                                                                 | Exit | Result                                                                                                              |
| --------------------------------------------------------------------------------------------------------------------------------------- | ---: | ------------------------------------------------------------------------------------------------------------------- |
| `cargo test --manifest-path src-tauri/Cargo.toml --features test-utils --test specos_agent_team_context -- --test-threads=1`            |    0 | 47 passed; includes T01-T06 and source-reservation regression.                                                      |
| `cargo test --manifest-path src-tauri/Cargo.toml --features test-utils --test specos_migration -- --test-threads=1`                     |    0 | 4 passed.                                                                                                           |
| `cargo check --manifest-path src-tauri/Cargo.toml --features test-utils`                                                                |    0 | desktop/default core compiles.                                                                                      |
| `cargo check --manifest-path src-tauri/Cargo.toml --no-default-features --bin codeg-server`                                             |    0 | server core compiles.                                                                                               |
| `pnpm exec vitest run src/components/tasks/specos/task-traceability-panel.test.tsx`                                                     |    0 | 4 passed; handoff and integration transport states.                                                                 |
| `pnpm exec tsc --noEmit`                                                                                                                |    0 | no diagnostics.                                                                                                     |
| `pnpm exec eslint src/components/tasks/specos/task-traceability-panel.tsx src/components/tasks/specos/task-traceability-panel.test.tsx` |    0 | no diagnostics.                                                                                                     |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check`                                                                                |    0 | Rust formatting is clean.                                                                                           |
| `pnpm build`                                                                                                                            |    0 | static production build completed. Node `25.6.0` emitted the repository engine-version warning for expected `24.x`. |
| `git diff --check`                                                                                                                      |    0 | no whitespace errors.                                                                                               |

## T01-T06

| Case                       | Core evidence                                                                                                                                                                                                                                                                    | Expected rejection / failure reason                                                                                 | Decision                                                         |
| -------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- |
| T01 handoff roundtrip      | `t01_handoff_roundtrip` records the correlated agent handoff for run 1 with actor, branch `src-a`, and its exact fixture HEAD.                                                                                                                                                   | stale/non-running generation cannot write an agent handoff.                                                         | command/SQLite/Git oracle pass                                   |
| T02 missing handoff blocks | `t02_missing_handoff_blocks_integration` returns `waiting_source`.                                                                                                                                                                                                               | `source handoff is missing for the live run`.                                                                       | command/SQLite oracle pass                                       |
| T03 source-head order      | `t03_source_head_order` captures `src-a` then `src-b` in deterministic task-ID order and snapshots full Git heads.                                                                                                                                                               | changed run/head makes the captured plan stale.                                                                     | command/SQLite/Git oracle pass                                   |
| T04 conflict recovery      | `t04_conflict_recovery` observes `MERGE_HEAD`; the plan now probes the recorded integration Worktree, falling back to the project root only for legacy tasks without a Worktree.                                                                                                 | active merge residue reports `MERGE_HEAD` and blocks launch.                                                        | command/Git oracle pass; engine recovery incomplete              |
| T05 gated landing          | `t05_gated_integration_landing` proves containment before `merge_landed`, records a common merge commit and `integrated_by`, and rejects a non-contained head. `integration_source_cannot_bypass_landing_settlement` rejects direct source completion while integration is live. | `workTask.integration.notContained` or `workTask.integration.sourceReserved`; source remains `review` on rejection. | command/SQLite/Git oracle pass; restart/engine parity incomplete |
| T06 legacy compatibility   | `t06_legacy_summary_compatibility` preserves unprofiled task readability and accepts an old `task_complete`-equivalent verdict/summary without creating a handoff.                                                                                                               | no handoff is required for legacy task readability.                                                                 | command/SQLite oracle pass                                       |

## Decision

The current command-core, SQLite, Git, and targeted UI/transport evidence is
green. This is not sufficient for independent verification of the approved
Test Spec: no real TaskEngine conflict-resolution/retry fixture, restart and
idempotent settlement fixture, Tauri/Axum roundtrip comparison, or full UI
state/accessibility coverage has run. Therefore `issue-050` remains
`implemented_pending_verification` and `issue-051` remains
`pending_verification`. The Feature Spec remains `approved`; its implementation
baseline is not independently verified.
