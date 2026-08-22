# Verification — BUGRAIL-SPECOS-004

- Date: `2026-08-21`
- Agent: `Fairy`
- Standard: `specos-test-standard/v1`
- Workspace commit: `3c7240c08184c28658330f15e5bcd08d35ee8c4d`
- Feature Spec: `approved`, SHA-256 `81ee40fe121cef77cb45120768e949a06e5883ea5b83cb64f6106a29fb8a9d4d`
- Test Spec: `approved`, SHA-256 `8deae207f7db73d44bd36f96fe92ab5ecc62df67254fba9a13224a93977e4b05`
- Decision: **not verified** — the 004 scenario suite passes, but the Test
  Spec's required repository-level evidence is not fully green.

## Scenario evidence

| Test Spec | Result | Durable oracle |
|---|---|---|
| T01 acyclic-edge-validation | pass | self/cycle rejection leaves the graph valid |
| T02 blocked-child-not-claimed | pass | service claim and `TaskEngine::start` return `workTask.dependency.unmet`; child remains `todo`, `run_seq=0` |
| T03 parallel-ready-claims | pass | two children become selectable after their common parent is `done` |
| T04 parent-failure-reason | pass | failed parent keeps child ineligible without a cascade transition |
| T05 concurrency-race | pass | opposite concurrent inserts yield exactly one edge; claim/write race cannot both commit |
| T06 legacy-task-readiness | pass | an edge-free legacy task stays ready and selectable |

## Commands

| Command | Exit | Result |
|---|---:|---|
| `cargo test --features test-utils --test specos_agent_team_context` | 0 | 47 passed; includes all 004 oracles and direct manual-start regression |
| `cargo test --features test-utils` | 101 | 004 test target passes, but unrelated `memory_capture_outbox::legacy_project_without_memory_provider_stages_nothing` fails (11 passed, 1 failed) |
| `cargo check` | 0 | desktop/shared core compiles |
| `cargo check --no-default-features --bin codeg-server` | 0 | standalone-server core compiles |
| `cargo clippy --all-targets --features test-utils -- -D warnings` | 0 | desktop/shared lint clean |
| `cargo clippy --no-default-features --bin codeg-server -- -D warnings` | 0 | standalone-server lint clean |
| `cargo test --no-default-features --bin codeg-server --lib` | 0 | 2,708 server/lib tests passed |
| `pnpm test src/components/tasks/specos/task-traceability-panel.test.tsx` | 0 | 3 passed; dependency rendering/loading/error states covered |
| `pnpm exec tsc --noEmit` | 2 | blocked by four missing `Tasks.specosIntegration*` keys in existing 005 Integration UI work |
| `pnpm build` | 0 | static production build completed |
| `cargo fmt --check` | 1 | pre-existing formatting drift across unrelated files; no formatting rewrite performed |

## Remaining risks and status gate

The dependency fix serializes graph writes with readiness/claim/setup checks,
so a writer cannot validate an old graph while a child is claimed. The direct
race, manual start, blocked child, parallel-ready children, failed parent, and
legacy compatibility scenarios are green.

Issue 048 remains `implemented_pending_verification` and Issue 049 remains
`pending_verification`. Their source Feature/Test Specs are approved and the
004 scenarios are green, but this record cannot claim full Test Spec evidence
while the required repository Rust suite and TypeScript check fail outside the
authorized 004 scope. The failures must be cleared or separately waived and
the affected commands rerun before either Issue is marked verified.

The tested serialization guard is process-local. The current single-engine
deployment contract makes that sufficient for the exercised command clients;
if independent processes are ever supported as concurrent writers of the same
SQLite database, add a database-level write-lock strategy and a multi-process
race test before relying on this guarantee across processes.
