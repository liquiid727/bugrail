---
id: BUGRAIL-SPECOS-019-TEST
version: "0.1"
status: draft
sourceSpecId: BUGRAIL-SPECOS-019
sourceSpecVersion: "0.1"
sourceSpecHash: "2f5d9567766bf10aaae33284ff3b581d17d35ddbe3ddfbdb2d5e749b9c401bf5"
independentFromImplementation: true
---

# Test Spec: Team Goal Intake And Static Workflow Launch

## Test Cases

| ID | Covers | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-019.T01` | R01-R03 | Submit a bounded goal with a valid static Workflow. | One TeamRun snapshot, expected WorkTasks/bindings/edges in one commit. |
| `BUGRAIL-SPECOS-019.T02` | R04 | Submit the same request ID concurrently. | One run ID and no duplicate node/task rows. |
| `BUGRAIL-SPECOS-019.T03` | R02,R03 | Use missing, invalid, cyclic or changed Workflow input. | No partial TeamRun or WorkTask mutation. |
| `BUGRAIL-SPECOS-019.T04` | R05 | Launch from Teams in wide/narrow keyboard flows. | Typed call and task drill-down; no ACP mode mutation. |
| `BUGRAIL-SPECOS-019.T05` | R06 | Restart and reconnect after launch. | Exact goal/hash and current WorkTask projections restored. |
| `BUGRAIL-SPECOS-019.T06` | R01-R06 | Compare Tauri and Axum operations. | Equivalent request/result/error semantics. |

## Evidence

Use transaction/idempotency fixtures, command-core transport tests and Teams UI
tests. Record source hash, commit, commands, migration version and normalized
results under `tests/results/`.

