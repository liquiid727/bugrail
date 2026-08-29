# SpecOS Agent Team And Context Handoff

## Meta

- Date: `2026-08-28`
- PRD: `.prd/prd-specos-agent-team-context-system.md`
- Source proposals: `docs/codeg-agent-team-orchestration-spec.md`,
  `docs/codeg-memory-context-system-spec.md`
- Decisions: ADR `001-004`
- Active Features: `BUGRAIL-SPECOS-001-009`, `015-017`
- Current implementation Issues: `001-005`, `043-060`, `071-081`
- Release posture: Features `017` and `028` verified; baseline Features
  `001-009` and `015-016` retain explicit independent-verification gaps

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
- Feature `BUGRAIL-SPECOS-017` Memory Plugin MVP01 is implemented: TencentDB
  v3 capture outbox with restart-safe delivery, recall normalization into
  immutable Context packages with provenance evidence, provider/delivery/
  preview commands and the Context page Memory tab. The exact Test Spec
  T01-T08 was executed 2026-08-22 against the pinned `v2.0.0+bugrail.1`
  fixture; see `tests/results/2026-08-22-specos-017-memory-verification.md`.
  Issues `077-081` are `verified`; the accepted evidence record is bound to
  the exact Feature/Test Spec hashes.
- Feature `BUGRAIL-SPECOS-028` is verified through Issues `082-085`: separate
  plugin contracts, shared assets, durable provider jobs and safe operations
  projections are established without authorizing draft Features `029-036`.
- `Teams` and `Context` are first-level localized workbench routes. Task Detail
  links persisted Contract, Run, Context and Handoff facts.

## Deliberate Boundaries

- Feature `017` MVP01 covers the WorkTask capture/recall slice only. The full
  Memory/Knowledge vision (Memory Hub, short-term Offload, Skill, Wiki,
  CodeGraph, Recall Router) remains unauthorized by
  `.features/bugrail-specoos-memory/`.
- Static configured DAGs are supported. Dynamic planner DAGs, supervisor loops,
  Agent-as-Tool and autonomous delegation remain outside this slice.
- Model fallback IDs remain configuration metadata; automatic routing is not an
  active Feature.

## Draft Team Mode Expansion

The previous all-in-one Agent Team Mode PRD/SPEC has been superseded as an
implementation source. Product and architecture umbrellas now live at
`.prd/prd-agent-team-mode-roadmap.md` and
`design/agent-team-mode-architecture.md`.

Draft Features `BUGRAIL-SPECOS-018` through `027` split reliability, goal
intake, dynamic planning, quality/finalization, retry/reassignment, permission,
budget, fallback, restart recovery, and remote operations into independently
reviewable contracts. Their Feature/Test Spec drafts are now committed under
`.features/` with test specs for `015-016`; they do not change the implemented
baseline and are not authorized for implementation until reviewed and approved.

## Verification Gate

Before release, execute Test Specs `001-009` and Issues `005`, `045`, `047`,
`049`, `051`, `053`, `055`, `058`, `060`, `073`, and `076`. Feature `017`
is already accepted by Issue `081`; its supplemental evidence does not close
the other Features' exact Test Specs. Record migration
up/down and legacy fixtures, Rust tests/check, command-core/Tauri/Axum parity,
frontend unit tests, TypeScript check, production build, responsive/keyboard
states, all locale catalogs, restart/concurrency, required-context failure, and
credential/path redaction evidence.
