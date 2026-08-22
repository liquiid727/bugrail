# BugRail SpecOS Roadmap

## Requirement Source

- `.prd/prd-specos-agent-team-context-system.md` — current product baseline.
- `.prd/prd-memory-plugin-mvp01.md` — current Memory Plugin MVP01 baseline.
- `.prd/prd-agent-team-mode-roadmap.md` — umbrella roadmap for Team Mode
  capabilities after the static Team baseline; it does not authorize a whole
  runtime as one Feature.
- `docs/codeg-agent-team-orchestration-spec.md` — approved source proposal for
  profiles, Teams, Workflows, runtime resolution, DAGs, and controls.
- `docs/codeg-memory-context-system-spec.md` — approved source proposal for the
  CodeG-owned Context control plane and TencentDB bootstrap boundary.
- `design/specos-control-plane-design.md` and ADRs `001-004` — code placement
  and architecture decisions.
- `design/agent-team-mode-architecture.md` — shared invariants and module
  placement for split Team Features `018-027`.
- `design/specos-client-interaction-design.md` — shared UI and state contract.

## Planned Sequence

| Order | Feature | Status | Depends on | User-visible closure |
|---:|---|---|---|---|
| 1 | `BUGRAIL-SPECOS-001` Spec-Linked WorkTask Quality | implemented, verification pending | — | Contract, AC and trusted merge gates |
| 2 | `BUGRAIL-SPECOS-002` Agent And Model Profiles | implementation baseline | 001 | Project profiles and immutable resolution |
| 3 | `BUGRAIL-SPECOS-003` WorkTask Run Evidence | implementation baseline | 001-002 | Inspect exact run generations |
| 4 | `BUGRAIL-SPECOS-004` WorkTask Dependencies | implementation baseline | 001,003 | Readiness and dependency evidence |
| 5 | `BUGRAIL-SPECOS-005` Integration WorkTask And Handoff | implementation baseline | 001,003-004 | Structured handoff and integration |
| 6 | `BUGRAIL-SPECOS-006` Deterministic Context Pack | implementation baseline | 001,003,005 | Exact bounded package per run |
| 7 | `BUGRAIL-SPECOS-007` Context Provider Bootstrap | implementation baseline | 006 | Provider health and TencentDB boundary |
| 8 | `BUGRAIL-SPECOS-008` Agent Context Loadouts | implementation baseline | 002,006-007 | Agent-scoped required/optional context |
| 9 | `BUGRAIL-SPECOS-009` Context Activity And Inspector | implementation baseline | 006-008 | Context route, packages and provenance |
| 15 | `BUGRAIL-SPECOS-015` Static Team Workflow | implementation baseline | 002-004,008 | Static Team DAG execution |
| 16 | `BUGRAIL-SPECOS-016` Team Operations And Handoff | implementation baseline | 005,009,015 | Team controls, node trace and handoff |
| 17 | `BUGRAIL-SPECOS-017` Memory Plugin MVP01 | implementation baseline | 003,006-009 | TencentDB v3 capture, recall, Context injection and UI |
| 18 | `BUGRAIL-SPECOS-018` Team Runtime Reliability Hardening | draft | 015-016 | Atomic controls, scheduler barrier, recovery projection and bounded run queries |
| 19 | `BUGRAIL-SPECOS-019` Team Goal Intake And Static Workflow Launch | draft | 018 | Persisted project goal and idempotent launch of a selected static Workflow |
| 20 | `BUGRAIL-SPECOS-020` Dynamic Team Plan Proposal | draft | 001,019 | Planner proposal, deterministic validation, exact-hash approval and WorkTask materialization |
| 21 | `BUGRAIL-SPECOS-021` Team Quality And Finalization | draft | 001,003,005,019 | Reviewer WorkTask, aggregate gate decision and evidence-backed final report |
| 22 | `BUGRAIL-SPECOS-022` Team Retry And Agent Reassignment | draft | 002-004,019 | Generation-safe retry/reassignment and downstream invalidation |
| 23 | `BUGRAIL-SPECOS-023` Team Effective Permission Policy | draft | 002,018 | Backend-enforced permission intersection and bounded escalation |
| 24 | `BUGRAIL-SPECOS-024` Team Usage Accounting And Budget | draft | 003,018 | Attributable usage, warnings and enforceable claim-time budgets |
| 25 | `BUGRAIL-SPECOS-025` Team Provider And Model Route Fallback | draft | 022-024 | Policy-controlled fallback through WorkTask generations |
| 26 | `BUGRAIL-SPECOS-026` Team Backend Restart Recovery | draft | 003,005,016,018 | Scheduler ownership and idempotent WorkTask-first recovery |
| 27 | `BUGRAIL-SPECOS-027` Team Notifications And Remote Operations | draft | 019,021,023,024,026 | Deduplicated durable notifications and authenticated remote controls |

## Migration Notes

- `BUGRAIL-SPECOS-001` remains the original accepted slice and is not
  renumbered or rewritten.
- Previous Features `002-005` were moved to `003-006` according to the table in
  the current PRD. Their old Issues `006-021` are retained as historical
  planning records and marked superseded/stale; replacement Issues
  start at `043` so no Issue identity is reused.
- Feature Specs `002-009` carry matching Test Specs. A code change can be
  implemented without claiming independent verification until its Test Spec
  evidence is executed and recorded.
- The previous all-in-one `.features/agent-team` PRD/SPEC has been superseded
  and removed from the Feature directory. Draft Features `018-027` own the
  split behavior and cannot advance to implementation before review.

## Delivery Boundaries

- Workflow nodes are WorkTasks; WorkTask remains the state machine.
- Agent execution always traverses existing ACP/CLI adapters.
- Context Providers contribute assets; CodeG owns loadout, budget, package,
  provenance, injection, and observability.
- Live events trigger refresh only. SQLite, Git, and validated `.codeg/*.yaml`
  are the reconstructible truth.
- Memory, Wiki, CodeGraph and Skill Evolution use separate module interfaces.
  Feature `017` covers Memory only; dynamic orchestration, Agent-as-Tool, full
  ContextFS and autonomous promotion remain later work.
- Team run status is a projection over WorkTask facts plus minimal run control;
  no split Feature may add parallel Team task, attempt, dependency, scheduler,
  event, artifact, Context or Worktree modules.
- Permission, budget and route fallback are separate enforcement Features. A
  configuration field is not enforced until its owning Feature is accepted and
  verified.
