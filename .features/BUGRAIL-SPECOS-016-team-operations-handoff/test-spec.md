---
id: BUGRAIL-SPECOS-016-TEST
version: "0.1"
feature: BUGRAIL-SPECOS-016
featureVersion: "0.1"
status: approved
independentFromImplementation: true
sourceSpec: ".features/BUGRAIL-SPECOS-016-team-operations-handoff/spec.md"
sourceSpecId: BUGRAIL-SPECOS-016
sourceSpecVersion: "0.1"
sourceSpecHash: "67a4fcb4682ca22c2994c7114376887e9e15a12c6b8daf54137ba051a47e7517"
---

# Test Spec: Team Operations And Handoff

## 1. Purpose And Oracle

Verify pause/resume/cancel controls, node traceability, bounded generation-
scoped handoff, WorkTask gate/merge compatibility, restart recovery, and
desktop/server/UI parity. The oracle is persisted Team/WorkTask/handoff/event
state after each operation; event delivery and frontend disablement are only
refresh hints.

## 2. Fixtures And Isolation

- Migrated SQLite plus a legacy database with ordinary WorkTasks, a paused and
  running Team run, and nodes in every relevant WorkTask status.
- A deterministic TaskEngine/ACP substitute that can hold active nodes,
  reject one cancellation, and stop between control-state and task updates.
- Source tasks with exact `run_seq`, Worktree/Session, Context Package, Spec
  contract/gates, dependency edges, and valid/invalid handoffs.
- Sequential/concurrent Tauri/Axum clients and React mocks for loading,
  partial failure, last-good, stale, localized, keyboard, and narrow layouts.

## 3. Scenario Matrix

| ID | Requirements | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-016.T01` | R01 | Pause a running Team, attempt new claims, then resume. | Control state is `paused`; active WorkTasks keep their existing semantics; no new node claim commits while paused; resume returns to `running` and pumps ready work. |
| `BUGRAIL-SPECOS-016.T02` | R02 | Cancel a Team with queued and active nodes, repeat cancel, and restart during cancellation. | Existing WorkTask cancellation path is used; each node has a durable outcome; repeat/cross-process cancel is idempotent; restart finishes recovery without resurrecting a claim. |
| `BUGRAIL-SPECOS-016.T03` | R03 | Open a Team node from the run projection and inspect its task/run facts. | Every node links to the ordinary Task Detail and preserves task ID, run sequence, Session/Worktree, Context Package, contract/gate, dependency, and handoff references when present. |
| `BUGRAIL-SPECOS-016.T04` | R04,R05 | Save valid, empty, oversized, and cross-generation handoffs; reload after restart. | Only bounded summary/artifacts/risks/open questions for the exact `(task_id, run_seq)` persist; invalid writes leave the previous row unchanged; transcript history is never copied implicitly. |
| `BUGRAIL-SPECOS-016.T05` | R04 | Attempt Team control, handoff, retry, merge, and complete operations while a source gate is unmet or an integration head is stale. | No Team operation bypasses the existing WorkTask contract, gate, dependency, Git-truth, or merge CAS; source remains review/blocked with an explainable reason. |
| `BUGRAIL-SPECOS-016.T06` | R06 | Invoke each operation through Tauri and Axum with authorized and unauthorized contexts. | Authorization, error keys, recovery behavior, and persisted results match exactly; no actor or folder ownership can be supplied by untrusted request JSON. |
| `BUGRAIL-SPECOS-016.T07` | R01-R06 | Drop a live event or make a refresh fail after a successful control/handoff write. | Reload reconstructs the last durable facts; UI retains last-good data and offers retry without deriving readiness locally. |
| `BUGRAIL-SPECOS-016.T08` | R03-R06 | Render no-node, loading, paused, running, partial failure, canceled, handoff, stale, conflict, and transport-error states in all supported locales and viewport widths. | Actions are keyboard reachable, statuses have text plus icon semantics, tables stack at narrow widths, and no required state depends on color, hover, or transient events. |

## 4. Cross-Cutting Assertions

1. Pause, resume, cancel, and handoff writes are atomic at their documented
   boundary and leave the previous valid projection on failure.
2. Cancel never deletes a source Worktree or changes a source out of `review`
   when integration has not proved Git containment.
3. Handoff is bounded, untrusted text is rendered through the existing safe
   path, and secrets/full transcript/environment values do not enter DTOs,
   SQLite, logs, or UI.
4. Existing unprofiled WorkTasks and the current ACP/Session/Worktree behavior
   remain readable and operable.
5. Desktop and standalone-server control and handoff operations use equivalent
   command-core results, errors, authorization, and restart projection.

## 5. Required Evidence

- Rust/SQLite tests for control CAS/idempotency, cancellation recovery, exact
  handoff generation, bounds, and gate/merge non-bypass.
- Restart evidence for each durable control outcome and handoff row.
- Shared command-core plus Tauri/Axum parity and authorization evidence.
- React tests for T08 including last-good transport failure, localization,
  keyboard focus, and narrow-layout behavior.
- A normalized result bound to this source hash, commit/dirty state, exact
  commands, exit codes, fixture revision, raw-output references, and any retry
  classification.

## 6. Commands

```bash
shasum -a 256 .features/BUGRAIL-SPECOS-016-team-operations-handoff/spec.md
cargo test --manifest-path src-tauri/Cargo.toml --features test-utils --test specos_agent_team_context -- --test-threads=1
cargo check --manifest-path src-tauri/Cargo.toml --features test-utils
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --features test-utils -- -D warnings
pnpm exec vitest run src/components/tasks/specos/task-traceability-panel.test.tsx src/contexts/tasks-view-context.test.tsx
pnpm exec tsc --noEmit
pnpm build
```

## 7. Acceptance

- [ ] Source hash matches this Test Spec.
- [ ] `BUGRAIL-SPECOS-016.T01-T08` have independent evidence or an explicitly
  reviewed non-applicability decision.
- [ ] `AC01-AC05` each have at least one passing blocking result.
- [ ] No pause/resume/cancel/handoff/merge/restart or transport authorization
  bypass is found.
