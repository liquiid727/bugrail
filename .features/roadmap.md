# BugRail SpecOS Roadmap

## Requirement Source

- `.prd/prd-specos-agent-team-context-system.md` — current product baseline.
- `docs/codeg-agent-team-orchestration-spec.md` — approved source proposal for
  profiles, Teams, Workflows, runtime resolution, DAGs, and controls.
- `docs/codeg-memory-context-system-spec.md` — approved source proposal for the
  CodeG-owned Context control plane and TencentDB bootstrap boundary.
- `design/specos-control-plane-design.md` and ADRs `001-003` — code placement
  and architecture decisions.
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
| 10 | `BUGRAIL-SPECOS-010` Repository Impact Snapshot | draft | 006 | Bounded repository relationship context |
| 11 | `BUGRAIL-SPECOS-011` Explainable Routing | draft | 003,006 | Policy-scored Agent/model routing |
| 12 | `BUGRAIL-SPECOS-012` Run Evaluation Projection | draft | 003,011 | Qualified outcome comparison |
| 13 | `BUGRAIL-SPECOS-013` Project Memory Candidates | draft | 006,012 | Reviewable Git-tracked memory |
| 14 | `BUGRAIL-SPECOS-014` Skill Experience Lifecycle | draft | 012-013 | Experience-to-Skill governance |
| 15 | `BUGRAIL-SPECOS-015` Static Team Workflow | implementation baseline | 002-004,008 | Static Team DAG execution |
| 16 | `BUGRAIL-SPECOS-016` Team Operations And Handoff | implementation baseline | 005,009,015 | Team controls, node trace and handoff |

## Migration Notes

- `BUGRAIL-SPECOS-001` remains the original accepted slice and is not
  renumbered or rewritten.
- Previous Features `002-010` were moved to `003-006` and `010-014` according
  to the table in the current PRD. Their old Issues `006-042` are retained as
  historical planning records and marked superseded/stale; replacement Issues
  start at `043` so no Issue identity is reused.
- Feature Specs `002-009` carry matching Test Specs. A code change can be
  implemented without claiming independent verification until its Test Spec
  evidence is executed and recorded.

## Delivery Boundaries

- Workflow nodes are WorkTasks; WorkTask remains the state machine.
- Agent execution always traverses existing ACP/CLI adapters.
- Context Providers contribute assets; CodeG owns loadout, budget, package,
  provenance, injection, and observability.
- Live events trigger refresh only. SQLite, Git, and validated `.codeg/*.yaml`
  are the reconstructible truth.
- Dynamic orchestration, Agent-as-Tool, autonomous Memory/Skill promotion,
  full ContextFS, and semantic CodeGraph remain later work.
