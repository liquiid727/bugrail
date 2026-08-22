# BUGRAIL-SPECOS-001 Gap Evidence — Loop Iteration 2026-08-23 (late)

## Decision

**Still NOT VERIFIED.** This loop iteration closed three of the blocking gaps
from `2026-08-23-specos-001-verification.md` with new engine-level fixtures and
re-ran the repository matrix with raw logs persisted. Issues `001-005` keep
their non-verified statuses. No existing workspace modification was touched;
all additions are new files or appended records.

## Binding

| Field | Value |
|---|---|
| Feature | `BUGRAIL-SPECOS-001` |
| Spec SHA-256 | `79160488f65ae762decaa6db4987c15a783f61c886588c1e9157fc1bb40ab0d0` (recomputed, unchanged) |
| Test Spec SHA-256 | `7e3e34408f23cef03b8cf6b099864b40999ad0f151525fb16d48171dd8ee09a9` (recomputed, unchanged) |
| Branch / HEAD | `main` / `3c7240c08184c28658330f15e5bcd08d35ee8c4d` (dirty worktree preserved) |
| New code | `src-tauri/tests/specos_engine_gaps.rs` (only) |

## Gaps Closed This Iteration

- **T17** — `t17_required_preflight_without_producer_persists_blocked_result`:
  drives `TaskEngine::run_preflight` (via `engine::new_for_test`) on a
  contract-bound task in `review` with default folder settings (no free-form
  command, no command reference). Asserts the persisted gate row is
  `blocked` / reason `producer_unavailable` / actor `engine` /
  terminal `finished_at`, and that merge stays rejected with
  `workTask.qualityGate.unmet` naming gate `preflight`.
- **T20** — `t20_rejected_complete_with_delete_worktree_has_no_cleanup_side_effect`:
  real `git init` + initial commit + real `git worktree add -b work-branch`
  fixture, task attached to the worktree folder row. Unmet required gate +
  `POST /api/work_task_complete {deleteWorktree: true}` → rejected; asserts
  task stays `review`, `cleanup_state` unset, worktree directory survives,
  folder row not soft-deleted, and exactly one new event — the auditable
  `quality_gate_blocked` record.
- **T27** — `t27_bind_core_rejects_mid_window_spec_change_and_persists_one_snapshot`:
  calls `work_task_contract_bind_core` directly (no HTTP transport). A Spec
  edit landing between preview and bind is rejected as
  `workTask.specContract.stale` with **zero** persisted contract rows, gate
  rows, or bind events; a fresh bind persists one internally consistent
  `(id, version, hash)` snapshot verified against an independent
  `read_spec_reference` re-read.

## Executed Matrix (raw logs in `logs/`)

| Command | Result | Log |
|---|---|---|
| `cargo test --features test-utils` (full) | pass — 5,588 tests / 0 failed incl. new suite | `logs/cargo-matrix.log` |
| `cargo clippy --all-targets --features test-utils -- -D warnings` | pass | `logs/cargo-matrix.log` |
| `cargo test --no-default-features --bin codeg-server --lib` | pass | `logs/cargo-matrix.log` |
| `cargo clippy --no-default-features --bin codeg-server --lib -- -D warnings` | pass | `logs/cargo-matrix.log` |
| `cargo check --no-default-features --bin codeg-mcp` | pass | `logs/cargo-matrix.log` |
| `cargo fmt --all -- --check` | **fail — pre-existing**: untracked `src-tauri/src/commands/memory.rs:8` (uncommitted 017 work, untouched) | `logs/cargo-matrix.log` |
| `pnpm test` | pass — 319 files / 4,227 tests | `logs/frontend-matrix.log` |
| `pnpm build` | pass | `logs/frontend-matrix.log` |
| `pnpm eslint .` | pass (42 pre-existing errors from 08-23 no longer reproduce with the current uncommitted `eslint.config.mjs`) | `logs/frontend-matrix.log` |

## Remaining Blocking Gaps (unchanged unless noted)

- `T28`: an actual Tauri invocation of preview/bind/read/gate/human-decision.
  Feasible path: `[dev-dependencies] tauri = { version = "2", features =
  ["test"] }` (feature-unified for test builds only), `tauri::test::mock_app()`
  + `app.manage(db)`, call the `#[tauri::command]` wrappers directly.
- `T29-T31`: full bind/rebind/waive UI journeys, secret-redaction checks, and
  light/dark + narrow/wide screenshots. Feasible path: run `codeg-server`
  against a seeded SQLite project and drive the static export via browser
  automation (session has browser tooling).
- Migration evidence: interrupted-transaction fixture (inject a failing step
  after the SpecOS migrations in a fixture `Migrator`; assert the schema is
  either fully pre- or post-feature). The pre-feature rollback + legacy-data
  survival half already exists in `tests/specos_migration.rs`
  (`down_drops_only_the_specos_tables`).
- Independent-verifier separation: this iteration both authored the fixtures
  and ran the matrix. Promotion of `001-005` still requires a code-free
  verification run against this exact Spec hash.

## Side Finding — BUGRAIL-SPECOS-017 Disposition Discrepancy

`tests/results/2026-08-22-specos-017-memory-verification.md` claims
`077-081`, `054`, `055`, `047`, `053`, `060` were set to **verified**, but:

- the issue files (committed at `e8c6d733`) still read
  `implemented_pending_verification` / `planned`;
- the referenced final audit `2026-08-22-specos-001-017-audit.md` never
  existed in git history;
- the verification record itself is an untracked file, and part of the 017
  implementation (`src-tauri/src/commands/memory.rs` et al.) is still
  uncommitted — the verified workspace state was never made durable;
- the current 017 spec hashes still match the record's binding
  (`f62824d3…` / `480c2e58…`).

Per the `.issues/README.md` execution rules the issue files are authoritative:
nothing in `043-060` / `077-081` is treated as verified by this loop. The
cheap recovery path for a later iteration: commit or explicitly discard the
017 working set, re-run the documented 017 matrix, then apply dispositions and
add the missing audit trail in one bookkeeping pass.
