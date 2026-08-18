# ADR-002: Resolve Expert Profiles Into WorkTask-Native Team Runs

- Status: accepted
- Date: 2026-08-12
- Sources: `docs/codeg-agent-team-orchestration-spec.md`,
  `.prd/prd-specos-agent-team-context-system.md`

## Context

CodeG already owns ACP/CLI adapters, Sessions, WorkTasks, Worktrees, streaming,
retry, cancellation, merge, and durable task transitions. Agent identity is
currently expressed mostly through adapter/task overrides. Adding an external
multi-agent runtime would duplicate exactly those stateful capabilities.

## Decision

1. `AgentProfile` is a project-scoped expert identity. It references a
   `ModelProfile`, runtime adapter, Skills, rules, tools, and Context Loadout.
2. Execution creates an immutable `ResolvedAgentRuntime` snapshot. Profiles do
   not execute directly, and model identity never defines Agent identity.
3. A Team is an expert pool. A separately versioned Workflow defines a static
   acyclic graph, node prompts/profiles, and maximum concurrency.
4. Workflow nodes materialize as existing WorkTasks. Dependency readiness and
   Team pause/concurrency are claim predicates; WorkTask status remains the
   node status and ACP/Worktree remains the execution path.
5. Project definitions use validated Git-trackable `.codeg/*.yaml`; runtime
   instances and projections use SQLite.

## Consequences

- Existing task, runtime, and transport invariants are reused and legacy tasks
  need no migration to a new state machine.
- Static DAG, sequential/parallel execution, retry and cancel are in scope.
- Dynamic planner-generated graphs, supervisor/review loops, Agent-as-Tool,
  global profile overlay, model fallback execution, and permission enforcement
  require later Specs; their fields must not be presented as enforced today.
