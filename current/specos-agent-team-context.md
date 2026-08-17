# SpecOS Agent Team And Context Handoff

## Meta

- Date: `2026-08-12`
- PRD: `.prd/prd-specos-agent-team-context-system.md`
- Source proposals: `docs/codeg-agent-team-orchestration-spec.md`,
  `docs/codeg-memory-context-system-spec.md`
- Decisions: ADR `001-003`
- Features: `BUGRAIL-SPECOS-001` through `016`
- Current implementation Issues: `001-005`, `043-076`
- Release posture: implemented foundations, independent verification pending

## Implemented Baseline

- `BUGRAIL-SPECOS-001`: existing contract/gate persistence, enforcement and
  Task Detail traceability are retained; Task Detail now exposes contract bind,
  AC/gates, run/context/dependency and handoff evidence in one panel.
- Project `.codeg/agents.yaml`, `.codeg/teams.yaml` and `.codeg/context.yaml`
  have validated, atomic, symlink-safe command-core read/write paths.
- Agent/Model profiles resolve into the existing ACP adapter and persist a
  redacted immutable `work_task_run` snapshot with legacy fallback.
- WorkTask run, dependency, handoff, Team run/node, Context Package/item and
  Context Activity persistence is added through one ordered migration.
- The existing scheduler filters dependency readiness, Team pause state and
  Team concurrency; Team Workflow launch creates ordinary WorkTasks.
- Context compilation reads bounded project-local sources, validates selected
  Provider health, persists one immutable package per run and injects it before
  ACP prompt dispatch.
- Tauri and Axum expose Agent/Team/Context/run/dependency/handoff commands from
  shared core behavior.
- `Teams` and `Context` are first-level localized workbench routes. Task Detail
  links persisted Contract, Run, Context and Handoff facts.

## Deliberate Boundaries

- TencentDB remote retrieval/write is not claimed yet; the bootstrap implements
  its Provider health/tool-discovery boundary and preserves replacementability.
- Static configured DAGs are supported. Dynamic planner DAGs, supervisor loops,
  Agent-as-Tool and autonomous delegation remain outside this slice.
- Model fallback IDs are configuration metadata; automated fallback execution
  and policy scoring remain `BUGRAIL-SPECOS-011`.
- Repository impact, evaluation, Memory promotion and Skill evolution remain
  planned Features `010-014`.

## Verification Gate

Before release, execute Test Specs `001-009` and Issues `005`, `045`, `047`,
`049`, `051`, `053`, `055`, `058`, `060`, `073`, and `076`. Record migration
up/down and legacy fixtures, Rust tests/check, command-core/Tauri/Axum parity,
frontend unit tests, TypeScript check, production build, responsive/keyboard
states, all locale catalogs, restart/concurrency, required-context failure, and
credential/path redaction evidence.
