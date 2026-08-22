# Independent release verification — BUGRAIL-SPECOS-001..009

- Date: `2026-08-21`
- Verifier: `Fairy` (independent verification)
- Workspace: `3c7240c08184c28658330f15e5bcd08d35ee8c4d` on `main`
- Worktree: dirty before verification; implementation files were not modified.
- Decision: **do not enter release-candidate stage**.

## Source binding

All recomputed Feature hashes match the Issue front matter. 001's Test Spec
also declares the recomputed 001 Feature hash. Versions match each paired Test
Spec (`001=0.3`; `002..009=0.2`). The 001 Feature and Test Spec remain
`draft`, so neither can qualify as a release verification source.

| Feature | Feature / Test SHA-256 | Spec state | Result |
|---|---|---|---|
| 001 | `81b9aff1…d657d` / `35584e8a…772ae` | draft / draft | **fail** |
| 002 | `f9342aa8…7fee9` / `dc82ce17…1318a` | approved / approved | **partial** |
| 003 | `6dcbc16b…38009` / `89e5bb0d…bfe92` | approved / approved | **partial** |
| 004 | `81ee40fe…a9d4d` / `8deae207…4b05` | approved / approved | **partial** |
| 005 | `1d5ff5e9…47678` / `7ed765c1…7e99d` | approved / approved | **partial** |
| 006 | `9a97e6ea…5afabb` / `dcd67c95…69904` | approved / approved | **partial** |
| 007 | `fc10e29a…669f` / `15e2f3c9…db6cb` | approved / approved | **partial** |
| 008 | `244f55f…4046d` / `c437b594…c3412` | approved / approved | **partial** |
| 009 | `0eb4d34c…7913a7` / `4ae27c01…909c8` | approved / approved | **fail** |

Full values are retained in the JSON peer record.

## Executed evidence

Minimal suite was run first:

| Command | Exit | Result |
|---|---:|---|
| `cargo test --features test-utils --test specos_agent_team_context -- --test-threads=1` | 0 | 47/47: run, dependency, handoff, package, provider, loadout, inspector; includes restart, concurrency, required/optional failure, path/credential redaction, and legacy cases. |
| `cargo test --features test-utils --test specos_migration -- --test-threads=1` | 0 | 4/4: migration up/down, indexes, FK cascade, legacy task readability. Down rolls back the current task-kind/integration migrations too. |
| targeted Teams/Context/Traceability/i18n Vitest | 0 | 25/25. |

Additional checks:

| Command | Exit | Result |
|---|---:|---|
| `cargo test --features test-utils --test specos_spec_binding -- --test-threads=1` | 0 | 10/10 real Axum router tests for 001 preview/bind/read/error/staleness/rebind. |
| `cargo test --no-default-features --bin codeg-server --lib` | 0 | 2,707 passed, 1 ignored. |
| `cargo check` / server / MCP variants | 0 | desktop, Axum server, and MCP compile. |
| desktop/server/MCP clippy with `-D warnings` | 0 | clean. |
| `pnpm test` | 0 | 318 files, 4,216 tests passed. |
| `pnpm exec tsc --noEmit` | 0 | no diagnostics. |
| `pnpm build` | 0 | static production build passed. |
| `cargo test --features test-utils` | 101 | unrelated repository regression: `memory_capture_outbox::legacy_project_without_memory_provider_stages_nothing` expected 0 staged rows, observed 1. |

`pnpm lint` was stopped after two minutes without output; it did not provide a
pass/fail result. A direct `vitest` invocation was also intentionally not used
as a gate: Node 25 enables experimental Web Storage and produces unrelated
`localStorage.* is not a function` failures. The official `pnpm test` wrapper
adds `--no-experimental-webstorage` and passed.

## Browser evidence

An isolated `codeg-server` was started with a temporary data directory and the
production `out/` directory. It applied the complete current migration set,
served the static UI, authenticated a test client, and opened the Context
workbench for this repository. The live page showed the expected empty Package
state and optional-Provider degradation. At `375×812`, `scrollWidth <=
innerWidth`; tabs and these state messages remained visible.

However, every live Context tab (`Overview`, `Codebase`, `Provider`,
`Loadout`, `Activity`) rendered `tabindex="-1"`, including the selected tab.
The tab strip is therefore not reachable by normal keyboard Tab navigation;
ArrowRight could not move selection because focus never entered the widget.
This is a concrete accessibility failure, not a missing test. It blocks the
keyboard clauses shared by 001 and 002..009, and directly fails 009 T06.

Unit tests demonstrate loading, empty, error and last-good/stale behavior;
the live browser demonstrated empty/degraded/responsive states. There is no
browser fixture covering every loading/error/stale/blocked state, so that
portion remains incomplete as well.

## Transport and scope gaps

001 has real Axum coverage; its Tauri wrapper and Axum handler call the same
core, but a real Tauri invoke comparison was not executed. 002..009 have
shared-core tests and frontend transport mocks, but no live Axum-vs-Tauri
parity fixture. This cannot be promoted to transport parity from source review.

The current command-core suite supplies strong coverage for WorkTask runs,
dependencies, handoff, Context Package, Provider, Loadout, Inspector,
restart, concurrency, required blocking, optional degradation, redaction, and
legacy compatibility. It does not satisfy all real-engine / real-transport / UI
contracts: in particular 005 lacks a real TaskEngine conflict/retry/restart
fixture; 007 lacks a successful remote health fixture; 008 lacks real ACP
prompt dispatch; and 002..009 lack the requested live parity fixture.

All ten locale catalogs have exactly 4,397 flattened keys and no missing or
extra keys versus `en.json`; the catalog unit test also passed.

## Issue disposition

| Issue | Feature | Evidence result | Status action |
|---|---|---|---|
| 004 (rechecked) | 001 | fail: 001 draft plus incomplete T01–T31 and keyboard failure | retain `implemented_pending_verification` |
| 005 | 001 | fail | retain `pending_verification` |
| 045 | 002 | partial: core/UI mocks pass; no actual parity; keyboard failure applies | retain `pending_verification` |
| 047 | 003 | partial: core restart/legacy pass; no actual parity/browser contract | retain `pending_verification` |
| 049 | 004 | partial: 004 core/migration regression passes; no actual parity/browser contract | retain `pending_verification` |
| 051 | 005 | partial: command/SQLite/Git pass; real engine/restart/parity absent | retain `pending_verification` |
| 053 | 006 | partial: core pass; no real transport/engine/browser contract | retain `pending_verification` |
| 055 | 007 | partial: no successful remote Provider health fixture or parity | retain `pending_verification` |
| 058 | 008 | partial: no real ACP prompt dispatch or parity | retain `pending_verification` |
| 060 | 009 | fail: live keyboard defect; state-browser matrix incomplete | retain `pending_verification` |

No Issue was changed to `verified`.

## Release gate

Release-candidate entry is denied until: (1) 001 Feature/Test Specs are
approved and every 001 blocking test has version-bound evidence; (2) the tab
keyboard defect is fixed and verified at wide/narrow viewports; (3) actual
Tauri/Axum parity fixtures cover 002..009; (4) the listed feature-specific
engine/remote/dispatch gaps are closed; (5) the repository Rust regression and
full lint gate are green; and (6) each Issue is independently re-reviewed
before its individual status is promoted.
