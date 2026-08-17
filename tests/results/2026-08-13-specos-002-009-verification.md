# Independent verification — BUGRAIL-SPECOS-002..009

- Date: `2026-08-13`
- Agents: `testing-agent` (045/047/049/051/053/055/058/060), `qa-agent` (053/055/058/060)
- Workspace commit: `c33b288f0d8b94432ae91069593fb8d7ee762a3b`
- Nested `bugrail/` commit: `b2b8c7e17a9e14462af5d20125d2bca42d0eff9f`
- Standard: `specos-test-standard/v1`
- Quality profile: `fullstack-flow`
- Decision: **not verified**. QA for 053/055/058/060: **blocked**.

## Spec hash check

Recomputed SHA-256 of each Feature Spec file. All eight issue frontmatter
hashes match the files on disk.

| Feature | Spec status | Spec hash | Issue |
|---|---|---|---|
| 002 Agent/Model Profiles | approved | `f9342aa89f379719d89fc8bcbcc7e612208652ec42fa9dd464c671185587fee9` | 045 |
| 003 Run Evidence | **draft** | `7dd9483c9193323ffe22b37c4b375056af0a556e6b927d502294373b22657492` | 047 |
| 004 Dependencies | **draft** | `f22459922841ca0a1136b6337f363753817b1d49c3657e3adbeddc15f8aef775` | 049 |
| 005 Integration/Handoff | **draft** | `99bb0dc5d494226570962a462e11177fa9c78f92e72bf6f8d3db877629b1d9d9` | 051 |
| 006 Context Package | **draft** | `1769705f83b91b410e8472c70f891b974b7062da0b8fd413fd5a4f138d5e1878` | 053 |
| 007 Provider Bootstrap | approved | `fc10e29a5c2be849a1573875aee7b3fb73f0292dcbfb5e23623c88318f6d669f` | 055 |
| 008 Context Loadouts | approved | `244f55f97115b3b067dc7fc031ee55bdf26879f30dcbc3a2d3b5ade24174046d` | 058 |
| 009 Context Inspector | approved | `0eb4d34c5db0677a58fafff40c9c963359455ea3d1750d4bb52fc53e707913a7` | 060 |

Test Specs `002-009` are `approved` / `0.2`. Feature Specs **003-006 remain draft**, so those Issues cannot be marked verified even when command-core tests pass.

## Commands

| Command | Exit | Notes |
|---|---:|---|
| `python3` SHA-256 of 16 spec/test-spec files | 0 | hashes above |
| `cargo test --features test-utils --test specos_agent_team_context -- --test-threads=1` | 101 | 32 passed, 1 failed (`t05_concurrency_race_cycle`) |
| `cargo test --features test-utils --test specos_migration -- --test-threads=1` | 0 | 4 passed (up/down, FK, indexes) |
| `cargo check --features test-utils` | 0 | command-core crate |
| `pnpm exec vitest run` Teams / Context / Traceability / i18n | 0 | 24 passed |
| `pnpm exec tsc --noEmit` | 0 | |
| `pnpm build` | 0 | Next.js 16.1.6 production |

Working directory: `bugrail/` (Rust commands from `bugrail/src-tauri/`).

New oracles:

- `src-tauri/tests/specos_agent_team_context.rs` — T01-T06 command/SQLite/YAML oracles
- `src/components/teams/teams-page.test.tsx` — Agent Profile UI states
- `src/components/context-system/context-page.test.tsx` — Provider/Loadout/Inspector states
- `src/components/tasks/specos/task-traceability-panel.test.tsx` — Run/Dep/Handoff/Package UI

## Scenario matrix

### 045 / Feature 002 — Agent Profile

| ID | Result | Oracle |
|---|---|---|
| T01 distinct-profile-same-model | pass | two profiles, one model, distinct resolved IDs |
| T02 resolution-precedence | pass | task > project default > explicit legacy |
| T03 invalid-catalog-atomicity | pass | secret-like save rejected; last valid YAML kept |
| T04 secret-redaction | pass | unknown adapter + secret key + symlink `.codeg` rejected |
| T05 legacy-fallback | **partial** | empty valid catalog falls back; **missing `agents.yaml` fails closed** (`version must be positive`) |
| T06 transport-and-ui-states | **partial** | UI no-workspace/loading/empty/success/last-good/error pass; no live Axum/Tauri catalog roundtrip |

### 047 / Feature 003 — Run

| ID | Result | Oracle |
|---|---|---|
| T01 one-row-per-generation | pass | claim + retry → two `work_task_run` rows |
| T02 claim-rollback | pass | wrong-status claim writes no row / no `run_seq` bump |
| T03 retry-run-attribution | pass | retry is `run_seq=2` with same profile id |
| T04 immutable-resolution | pass | persisted resolution survives status update |
| T05 restart-projection | pass | second `list_runs` returns the same row |
| T06 legacy-event-compatibility | pass | unclaimed task lists empty runs and stays readable |

Blocker: Feature Spec 003 is **draft**.

### 049 / Feature 004 — Dependencies

| ID | Result | Oracle |
|---|---|---|
| T01 acyclic-edge-validation | pass | cycle and self-edge rejected |
| T02 blocked-child-not-claimed | **partial** | `next_queued` skips blocked child; `TaskEngine::start` still claims without a readiness check |
| T03 parallel-ready-claims | pass | two ready children both selectable |
| T04 parent-failure-reason | pass | failed parent keeps child unselected |
| T05 concurrency-race | **fail** | opposite concurrent inserts both commit (`t05_concurrency_race_cycle`) |
| T06 legacy-task-readiness | pass | no edges → ready |

Blockers: T05 fail; Feature Spec 004 is draft; manual start is not gated.

### 051 / Feature 005 — Handoff

| ID | Result | Oracle |
|---|---|---|
| T01 handoff-roundtrip | pass | save/get summary/artifacts |
| T02 missing-handoff-blocks | **partial** | empty summary rejected and leaves no row; **no integration-eligibility block** |
| T03 source-head-order | **missing** | no `integration_plan` / source-head API |
| T04 conflict-recovery | **missing** | no integration Git merge residue path |
| T05 gated-integration-landing | **missing** | no contained-head settlement |
| T06 legacy-summary-compatibility | pass | unprofiled task has no handoff and remains readable |

Blockers: T03-T05 unimplemented; Feature Spec 005 is draft.

### 053 / Feature 006 — Context Package

| ID | Result | Oracle |
|---|---|---|
| T01 deterministic-order-hash | pass | second `prepare_run` on same `(task, run_seq)` reuses package + hash |
| T02 required-budget-block | pass | required oversize source errors, no package |
| T03 optional-source-absence | pass | missing optional skipped |
| T04 path-and-symlink-escape | pass | escaped symlink / `..` rejected |
| T05 retry-package-isolation | pass | retry binds a new package id |
| T06 post-restart-inspection | pass | `package_get` reconstructs items + hash |

Blocker: Feature Spec 006 is draft. No live transport test. No packaged/real-engine run.

### 055 / Feature 007 — Provider

| ID | Result | Oracle |
|---|---|---|
| T01 local-provider-health | pass | `kind=local` → `healthy` |
| T02 remote-health-normalization | **partial** | connection-refused → `degraded`; no successful `/v3/tools/list` fixture |
| T03 required-provider-block | pass | required unhealthy remote blocks `prepare_run` |
| T04 optional-provider-degradation | pass | optional remote yields package `status=degraded` |
| T05 timeout-and-redaction | pass | disabled health; secret_env name only; file:// and lowercase env rejected |
| T06 provider-ui-states | pass | healthy / degraded / last-good / error UI |

Blocker: T02 incomplete; no live transport test.

### 058 / Feature 008 — Loadout

| ID | Result | Oracle |
|---|---|---|
| T01 loadout-precedence | pass | default vs requested `review` loadout |
| T02 project-boundary | pass | symlink escape + `..` validation |
| T03 dedupe-and-budget | pass | identical content kept once |
| T04 required-source-failure | pass | required missing/oversize/escape blocks |
| T05 immutable-run-binding | pass | retry isolation |
| T06 prompt-injection-order | pass | `b.md` appears before `c.md` in prompt |

Blocker: no live transport test; no packaged/real-engine injection.

### 060 / Feature 009 — Inspector

| ID | Result | Oracle |
|---|---|---|
| T01 overview-empty-loading | pass | loading + empty packages UI |
| T02 package-provenance | pass | item provenance + inspector list |
| T03 task-run-package-join | pass | overview package has task/run ids |
| T04 last-good-refresh-failure | pass | refresh error keeps last-good snapshot |
| T05 activity-restart | pass | activity row after compile |
| T06 locale-and-accessibility | **partial** | all 9 locale catalogs have Teams/Context/Tasks keys; no keyboard/responsive browser evidence |

Blocker: no Playwright/responsive/keyboard evidence.

## Cross-cutting

1. Invalid writes do not replace last-valid YAML or create empty handoff rows — observed for catalogs, handoff summary, escaped sources.
2. Restart-safe projections exist for runs, packages, and overview. Live events are not the oracle in these tests.
3. Secrets: catalog secret-like keys rejected; provider health DTO has no bearer token; UI shows `secretEnv` **name** only.
4. Legacy unprofiled tasks remain readable. Missing `agents.yaml` is **not** treated as empty catalog.
5. Tauri and Axum handlers call the same `commands/specos_control.rs` cores (code inspection). **No live Axum/Tauri parity test was executed** for these eight Issues.
6. Frontend unit tests cover no-workspace, loading, empty, success, last-good, transport-error. They do not cover keyboard reachability or viewport variants.

## QA (053 / 055 / 058 / 060)

| Issue | Decision | Why |
|---|---|---|
| 053 | `blocked` | Feature 006 draft; no live transport; no packaged/real-engine compile |
| 055 | `blocked` | T02 incomplete; no live transport |
| 058 | `blocked` | no live transport; no real prompt-dispatch evidence |
| 060 | `blocked` | no keyboard/responsive browser evidence |

Merge / promote recommendation: **do not promote**. Residual risk is the 004 concurrent-cycle hole and the 005 integration Git-truth gap, which also sit under draft Feature Specs.

## Smallest rerun

```bash
cd bugrail/src-tauri
cargo test --features test-utils --test specos_agent_team_context --test specos_migration -- --test-threads=1
cd ..
pnpm exec vitest run src/components/teams/teams-page.test.tsx \
  src/components/context-system/context-page.test.tsx \
  src/components/tasks/specos/task-traceability-panel.test.tsx \
  src/i18n/messages.test.ts
```
