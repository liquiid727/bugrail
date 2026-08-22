---
id: BUGRAIL-SPECOS-020-TEST
version: "0.1"
status: draft
sourceSpecId: BUGRAIL-SPECOS-020
sourceSpecVersion: "0.1"
sourceSpecHash: "8a9ae0653633f6aa85f533e54527d7ad0bf313eadac71789f217fa324ffa474d"
independentFromImplementation: true
---

# Test Spec: Dynamic Team Plan Proposal

## Test Cases

| ID | Covers | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-020.T01` | R01,R02 | Produce a valid bounded plan from a fake Planner Agent. | Immutable planner WorkTask generation and proposal artifact. |
| `BUGRAIL-SPECOS-020.T02` | R02,R03 | Fuzz missing fields, duplicate IDs, cycles, limits and unsupported nodes. | Deterministic validation; zero implementation WorkTasks. |
| `BUGRAIL-SPECOS-020.T03` | R04 | Attempt direct state/permission/budget mutation in Planner output. | Treated as untrusted data; no runtime mutation. |
| `BUGRAIL-SPECOS-020.T04` | R05 | Approve one hash, then edit/retry the proposal. | Old decision becomes stale; new hash needs a new decision. |
| `BUGRAIL-SPECOS-020.T05` | R06 | Materialize an approved sequential/parallel plan. | One atomic set of ordinary WorkTasks and dependency edges. |
| `BUGRAIL-SPECOS-020.T06` | R07 | Retry invalid Planner output and restart. | Earlier evidence retained; current proposal/decision restored. |

## Evidence

Use schema/property tests, fake ACP Planner fixtures, approval authorization,
SQLite transaction tests and one UI proposal/approval flow. No paid provider is
required in deterministic CI.

