# PRD: SpecOS Agent Team And Context System In BugRail

## Meta

- Status: approved implementation baseline
- Date: 2026-08-12
- Product: Code: BugRail / CodeG
- Replaces: `.prd/prd-specos-delivery-control.md` for Features `002-016`
- Source proposals:
  - `docs/codeg-agent-team-orchestration-spec.md`
  - `docs/codeg-memory-context-system-spec.md`
- Design: `design/specos-control-plane-design.md`
- Decisions: `design/adr/ADR-002-agent-profile-team-worktask.md`,
  `design/adr/ADR-003-context-orchestrator-provider-boundary.md`

## 1. Product Decision

BugRail will add Agent Profiles, Model Profiles, static expert Teams, explicit
Workflow DAGs, and a CodeG-owned Context control plane on top of its existing
WorkTask, ACP, Session, Worktree, Skill, SQLite, event, and transport modules.

It will not embed a second multi-agent framework or let a memory vendor own
prompt composition. A workflow node materializes as a normal WorkTask; an
Agent execution resolves to the existing ACP adapter; a Context Provider
contributes assets, while CodeG owns selection, budgets, provenance, injection,
and the immutable package bound to a run generation.

## 2. Problems To Solve

- Agent identity is currently conflated with a runtime adapter and ad-hoc task
  configuration; the same model cannot be managed as several expert roles.
- Parallel WorkTasks lack a reusable Team/Workflow definition, dependency-aware
  launch controls, and a run-level orchestration view.
- Retried task generations are hard to reconstruct as immutable Agent/model,
  Session, Worktree, context, gate, and outcome facts.
- Project rules, docs, memory, wiki, code intelligence, and Skills have no
  single loadout, budget, provenance, or failure contract.
- A remote memory system could become an accidental architectural authority and
  make runs non-reproducible or silently weaken required context.

## 3. Users And Outcomes

The primary user is a developer or technical lead coordinating Spec-driven
delivery. They can:

1. define several expert Agent identities independently of model and CLI;
2. compose a static Team and versioned Workflow DAG;
3. start, pause, resume, cancel, retry, and inspect Team execution while each
   node retains normal WorkTask/Worktree behavior;
4. see the exact Agent/model resolution and Context Package for every run;
5. understand required-provider failures, degraded optional sources, budgets,
   provenance, and freshness from the UI;
6. exchange structured handoffs rather than copying entire conversations;
7. evolve context, memory, and Skills only through governed, reviewable stages.

## 4. Product Requirements

### Agent Identity And Runtime

| ID | Requirement |
|---|---|
| `P-ATC-01` | `AgentProfile` is a stable expert identity and references, rather than equals, a `ModelProfile` and runtime adapter. |
| `P-ATC-02` | Profiles declare scoped Skills, rules, tools, reasoning, context loadout, and allowlisted runtime configuration; secrets are references, never profile values. |
| `P-ATC-03` | Project profiles are Git-trackable, schema-versioned, validated before save, and resolved through an immutable run snapshot. |
| `P-ATC-04` | Resolution follows explicit run/task override, workflow node, Agent profile, project default, then legacy folder/default behavior. |
| `P-ATC-05` | Existing tasks without profile identifiers keep current ACP behavior. |

### WorkTask Run And Coordination

| ID | Requirement |
|---|---|
| `P-ATC-06` | Every claimed task generation has one durable run projection containing resolved Agent/model, Session, Worktree, Context Package, status, and timestamps. |
| `P-ATC-07` | WorkTask dependencies gate launch using persisted task facts; the current WorkTask state machine remains authoritative. |
| `P-ATC-08` | Structured handoff records summary, artifacts, risks, and open questions for a task generation. |
| `P-ATC-09` | A static Team is an expert pool; a versioned Workflow separately defines nodes, profiles, prompts, dependencies, and concurrency. |
| `P-ATC-10` | Workflow launch materializes ordinary WorkTasks and dependency edges, then uses the existing scheduler, ACP, Session, Worktree, retry, cancel, and merge paths. |
| `P-ATC-11` | Team control state supports running, paused, resumed, and canceled without inventing node statuses outside WorkTask. |

### Context Control Plane

| ID | Requirement |
|---|---|
| `P-ATC-12` | CodeG owns the Context domain, provider contract, loadout, budget, provenance, immutable package, injection, and observability. |
| `P-ATC-13` | A loadout selects required/optional project-local sources and Provider capabilities with item, byte, and token budgets. |
| `P-ATC-14` | Required missing/unhealthy context blocks launch; optional provider failure produces an explicit degraded package and activity record. |
| `P-ATC-15` | Context paths are canonicalized under the project, deduplicated by content hash, bounded, and stored against exact `task_id/run_seq`. |
| `P-ATC-16` | TencentDB Agent Memory is the first remote bootstrap target through an adapter-compatible health/tool boundary, not an Agent dependency or architecture owner. |
| `P-ATC-17` | Context UI exposes provider health, package contents, hashes, scopes, budgets, and activity, including empty/loading/degraded/blocked/error states. |

### Governance And Compatibility

| ID | Requirement |
|---|---|
| `P-ATC-18` | Agent output cannot self-authorize a human gate, permission expansion, memory promotion, Skill activation, or destructive operation. |
| `P-ATC-19` | Experience is distinct from Memory and Skill; promotion requires repeated evidence, validation, explicit approval, versioning, and rollback. |
| `P-ATC-20` | Desktop and standalone-server transports expose equivalent command-core behavior. |
| `P-ATC-21` | Existing `codeg` commands, routes, URI schemes, data files, ACP adapters, and unprofiled tasks remain compatible. |
| `P-ATC-22` | Runtime correctness is reconstructible from SQLite/Git facts; live events are refresh hints only. |

## 5. Golden Paths

### 5.1 Expert Team

```text
project Agent/Model Profiles
  -> static Team + versioned Workflow
  -> validate references and acyclic DAG
  -> materialize WorkTasks and dependencies
  -> resolve each node to an immutable Agent runtime
  -> launch ready nodes up to max concurrency
  -> persist run/context/handoff evidence
  -> inspect, pause/resume/cancel, retry, review and merge normally
```

### 5.2 Context Bootstrap

```text
project Context loadout
  -> health-check selected Providers
  -> block on unavailable required Provider
  -> retrieve/read bounded sources
  -> normalize + deduplicate + budget
  -> store immutable Context Package and provenance
  -> bind package to WorkTask run
  -> compile into prompt above existing ACP/CLI adapter
  -> expose package and activity after restart
```

## 6. Feature Sequence And Renumbering

`BUGRAIL-SPECOS-001` is preserved as the existing WorkTask contract/gate slice.
Historical Features `002-010` move as follows:

| Previous | Replacement |
|---|---|
| `002` WorkTask Run Evidence | `003` |
| `003` WorkTask Dependencies | `004` |
| `004` Integration/Handoff | `005` |
| `005` Deterministic Context Pack | `006` |
| `006` Repository Impact | `010` |
| `007` Explainable Routing | `011` |
| `008` Run Evaluation | `012` |
| `009` Project Memory | `013` |
| `010` Controlled Skills | `014` Skill Experience Lifecycle |

New slices are:

| Feature | Closure |
|---|---|
| `002` Agent And Model Profiles | Project catalogs and immutable runtime resolution |
| `007` Context Provider Bootstrap | Provider health/adapter boundary, including TencentDB bootstrap |
| `008` Agent Context Loadouts | Required/optional sources, scope, budgets, and run binding |
| `009` Context Activity And Inspector | Context navigation, health, packages, provenance, and errors |
| `015` Static Team Workflow | Team/workflow catalogs, DAG materialization, and bounded scheduling |
| `016` Team Operations And Handoff | Team run controls, node inspection, structured handoff, and parity |

## 7. UX Contract

- `Teams` and `Context` are first-level workbench routes beside `Tasks`.
- Teams uses a semantic ordered DAG/list in the MVP; no canvas dependency is
  required to understand order or status.
- Agent and Team definitions always show validation errors before save/start.
- Team nodes show Agent, model/reasoning, status, task, and run references;
  unavailable cost/token fields are shown as unknown, never fabricated.
- Context Overview shows Provider health and recent packages/activity. Task
  Detail links the exact package, run snapshot, dependencies, contract/gates,
  and handoff.
- Last successfully loaded persisted data remains visible with an explicit
  degraded/error banner during transient transport failure.
- Keyboard access, focus visibility, responsive narrow layouts, and all ten
  locale catalogs remain required.

## 8. Storage And Security

- Project definitions live in `.codeg/agents.yaml`, `.codeg/teams.yaml`, and
  `.codeg/context.yaml`; writes are validated, symlink-safe, and atomic.
- SQLite stores runtime facts only: run snapshots, dependency/handoff edges,
  Team runs/node bindings, immutable Context Packages/items, and activity.
- Profile config accepts allowlisted identifiers. Provider credentials are
  environment/keychain references and are redacted from persisted snapshots.
- Context source canonicalization rejects project escape and unsafe symlinks;
  budgets and UTF-8/file policies are enforced before persistence or prompt use.
- A required source/provider error is fail-closed. Optional failures are
  explicit degradation and cannot silently claim a complete package.

## 9. Non-Goals

- Embedding CrewAI, LangGraph, or a second Session/task/runtime framework.
- Dynamic autonomous team creation, supervisor loops, Agent-as-Tool, automatic
  model brokerage, or unbounded delegation in the first release.
- Rebuilding ACP/CLI adapters or bypassing them with direct Provider calls.
- Shipping TencentDB services inside the desktop application or treating its
  MemoryPanel as the CodeG Context UI.
- Full ContextFS, semantic CodeGraph, vector ranking, Wiki regeneration, or
  automatic Memory/Skill promotion in the bootstrap release.

## 10. Success And Release Gates

- Two profiles using the same model resolve as distinct expert identities.
- A parallel DAG never launches a child before persisted dependencies pass and
  never exceeds its configured concurrency.
- Pause prevents new claims; resume continues; cancel uses existing task
  cancellation and leaves durable outcomes.
- A restarted process reconstructs Team nodes, run resolution, package contents,
  health/activity history, and handoffs from persisted/configured facts.
- Required context failure blocks before prompt dispatch; optional failure is
  visible and reproducible.
- Legacy WorkTask, ACP, transport, migration, TypeScript, UI, and locale tests
  remain green; new Feature/Test Specs provide independent acceptance evidence.
