---
id: BUGRAIL-SPECOS-015-TEST
version: "0.1"
feature: BUGRAIL-SPECOS-015
featureVersion: "0.1"
status: approved
independentFromImplementation: true
sourceSpec: ".features/BUGRAIL-SPECOS-015-static-team-workflow/spec.md"
sourceSpecId: BUGRAIL-SPECOS-015
sourceSpecVersion: "0.1"
sourceSpecHash: "cc08a8aa814ea9dfd0658ceb8d0f4a625a960a0f2f8ffd3c4104f8363a9a783f"
---

# Test Spec: Static Team Workflow

## 1. Purpose And Oracle

Verify the observable Team catalog, Workflow materialization, dependency
scheduling, concurrency bound, restart projection, and partial-launch behavior
without treating implementation output or live events as proof. Authoritative
oracles are validated Git-tracked definitions, SQLite rows after restart,
WorkTask status and dependency facts, command-core results, Tauri/Axum wire
parity, and rendered UI states.

The source Feature Spec's R06 has no dedicated acceptance criterion. `T06` is
therefore a blocking requirement-level scenario; approval evidence must either
pass it or record an explicit product decision that adds an AC or removes R06.

## 2. Fixtures And Isolation

- Temporary project root with valid and invalid `.codeg/agents.yaml` and
  `.codeg/teams.yaml` definitions, including duplicate IDs, unknown profiles,
  self edges, cycles, and invalid concurrency values.
- Migrated SQLite plus a pre-015 legacy database containing ordinary
  unprofiled WorkTasks.
- Deterministic ACP/runtime substitute that can fail one node during
  materialization or launch and can pause before a claim commits.
- Sequential and concurrent Tauri/Axum command clients using the same command
  core.
- React transport mocks for no workspace, loading, empty, valid, invalid,
  partial failure, last-good, and transport-error states.

## 3. Scenario Matrix

| ID | Requirements | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-015.T01` | R01,R02 | Save a valid Team/Workflow catalog, then submit duplicate IDs, unknown Agent Profiles/Teams, missing nodes, self edges, cycles, and concurrency values outside the limit. | Valid YAML remains readable; every invalid write returns a typed validation error and leaves the last valid file and SQLite state unchanged. |
| `BUGRAIL-SPECOS-015.T02` | R03 | Start a valid workflow with sequential and parallel nodes. | One Team run stores exact workflow version/hash; every node creates one ordinary WorkTask with profile/loadout/team identifiers and one persisted node binding. |
| `BUGRAIL-SPECOS-015.T03` | R04 | Materialized nodes contain completion edges; complete a parent and pump the folder. | Only ready children are claimed; blocked children remain queued/todo with dependency facts, and no second node state machine is consulted. |
| `BUGRAIL-SPECOS-015.T04` | R04 | Claim many ready nodes concurrently with `max_concurrent=2`. | At most two active WorkTask statuses exist at any point; the bound is enforced by the backend claim/pump transaction, not UI disablement. |
| `BUGRAIL-SPECOS-015.T05` | R05 | Restart after nodes are todo, queued, running, review, and done. | Team run and node projection reconstructed from persisted bindings and current WorkTask facts; live events are not required. |
| `BUGRAIL-SPECOS-015.T06` | R06 | Fail one node during materialization or launch after the Team run record is created. | Result is explicitly partial/failed with the failed node and reason; it is never reported as fully started, and persisted node facts are recoverable after restart. |
| `BUGRAIL-SPECOS-015.T07` | R01-R06 | Use a legacy unprofiled task and malicious/oversized catalog prompts. | Legacy task remains readable and schedulable; bounded validation rejects unsafe definitions without leaking secrets or creating partial rows. |
| `BUGRAIL-SPECOS-015.T08` | R01-R06 | Repeat catalog get/save and run start through Tauri and Axum, then render Teams and Task Detail states. | Request/result/error payloads match; UI displays loading, empty, validation, partial failure, node links, and last-good data with keyboard and narrow-layout access. |

## 4. Cross-Cutting Assertions

1. Catalog validation, Team run creation, and node materialization are atomic
   at their documented boundaries; invalid input never leaves orphan tasks or a
   new definition.
2. Workflow version/hash and Agent Profile/loadout identifiers are immutable
   run facts and survive process restart.
3. Existing WorkTask status, ACP, Worktree, Git, retry, recovery, and no-Team
   behavior remain compatible.
4. A Team operation cannot claim an unmet dependency or exceed the configured
   concurrency bound through a stale UI request or concurrent command.
5. Secrets, full environment values, uncapped prompt/output, and paths outside
   the project are absent from SQLite, DTOs, logs, and rendered UI.
6. Tauri and standalone-server adapters call the same command core and expose
   equivalent authorization, typed errors, and persisted facts.

## 5. Required Evidence

- Rust command-core and SQLite assertions for catalog validation, snapshot/hash,
  materialization, dependency readiness, bounded claims, restart projection,
  and partial launch.
- Migration fixture evidence showing legacy rows remain readable and foreign
  keys/indexes remain valid.
- Concurrent test output recording attempt count, active-status maximum, and
  failure classification.
- Tauri/Axum parity assertions and React tests for every state in T08; a
  screenshot alone is not sufficient for a scenario.
- A normalized result bound to this source hash, the BugRail commit/dirty state,
  exact commands, exit codes, fixture revision, and raw-output references.

## 6. Commands

```bash
shasum -a 256 .features/BUGRAIL-SPECOS-015-static-team-workflow/spec.md
cargo test --manifest-path src-tauri/Cargo.toml --features test-utils --test specos_agent_team_context -- --test-threads=1
cargo check --manifest-path src-tauri/Cargo.toml --features test-utils
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --features test-utils -- -D warnings
pnpm exec vitest run src/components/teams/teams-page.test.tsx src/components/tasks/specos/task-traceability-panel.test.tsx
pnpm exec tsc --noEmit
pnpm build
```

## 7. Acceptance

- [ ] Source hash matches this Test Spec.
- [ ] `BUGRAIL-SPECOS-015.T01-T08` have independent evidence or an explicitly
  reviewed non-applicability decision.
- [ ] `AC01-AC05` each have at least one passing blocking result.
- [ ] R06/T06 is resolved by passing evidence or a source-Spec decision; it may
  not disappear because the source Spec omitted an AC.
- [ ] No legacy WorkTask, dependency, concurrency, restart, transport, or UI
  bypass is found.
