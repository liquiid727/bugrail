# Independent verification — BUGRAIL-SPECOS-015/016

- Date: `2026-08-23`
- Verifier: `Fairy`
- Scope: issue-071, issue-072, issue-073, issue-074, issue-075, issue-076
- Decision: **partial; retain `implemented_pending_verification` / `pending_verification`**
- SpecOS route: only approved `015/016`; no `018-027` implementation

## Evidence commands

| Command | Result |
|---|---:|
| `cargo check --no-default-features --features test-utils` | pass |
| `cargo clippy --no-default-features --bin codeg-server --lib --features test-utils -- -D warnings` | pass |
| `cargo test --no-default-features --features test-utils --test specos_agent_team_015_016 -- --test-threads=1` | pass, 4/4 |
| `cargo test --features test-utils --test specos_agent_team_015_016 -- --test-threads=1` | pass, 4/4 |
| `pnpm exec vitest run src/i18n/messages.test.ts src/components/teams/teams-page.test.tsx src/components/tasks/specos/task-traceability-panel.test.tsx src/contexts/workbench-route-context.test.tsx` | pass, 23/23 |
| `pnpm test` | pass, 319 files / 4228 tests |
| `pnpm exec tsc --noEmit` | pass |
| targeted `pnpm eslint` | pass after hook dependency fix |

Both the no-default-feature server path and the default-feature Tauri path
passed the same real SQLite/command-core integration suite.

## Scenario matrix

| Acceptance surface | Evidence | Result |
|---|---|---|
| Real SQLite Team start | `specos_agent_team_015_016::t015_real_start_orders_dag_and_reserves_team_concurrency` | pass |
| Sequential DAG | root dependency only becomes runnable after `root-a` is done | pass |
| Parallel DAG | independent `root-b` becomes runnable after the first slot is released | pass |
| Cycle and oversized prompt rejection | `t015_catalog_rejects_cycles_oversized_prompts_and_unknown_profiles` | pass |
| `maxConcurrent` | second root is rejected while the first is `preparing` | pass |
| TaskEngine unavailable | start remains `queued`; cancel returns explicit `task engine not running` | pass |
| Pause/resume CAS and repeats | `t016_controls_are_idempotent_cas_and_engine_absence_is_explicit` | pass |
| Terminal control guard | canceled run accepts repeated cancel but rejects resume | pass |
| Partial launch durability | implementation marks already materialized todo/queued nodes as `failed` with `team_launch_error`; no induced DB-fault oracle yet | partial |
| Team node → ordinary Task Detail | keyboard-capable node button and route handoff unit coverage; live `codeg-server` browser opened Plan detail | pass |
| Axum/shared core parity | catalog save JSON and run start through real Axum router | pass |
| Handoff generation | UI requests handoff with current `task.run_seq` | pass in unit scope |
| Last-good evidence recovery | refresh failure keeps previously loaded Traceability tabs visible | pass in unit scope |
| Keyboard/narrow layout | live Enter activation opened Plan detail; 1512px and 375px screenshots had `scrollWidth == innerWidth` | pass for checked routes |
| Locale rendering | live zh-CN Teams workflow/status view; 10-message-file key parity passes | partial; full live locale matrix pending |

## Status action

Do not mark any of issue-071/072/073/074/075/076 as `verified`. The remaining
verification gap is runtime/browser evidence, not a draft SpecOS feature. No
Issue, Spec, Test Spec, `.eve/`, `agent/`, or `assets/` file was rewritten.
