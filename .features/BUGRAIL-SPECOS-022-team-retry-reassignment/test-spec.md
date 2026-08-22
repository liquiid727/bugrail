---
id: BUGRAIL-SPECOS-022-TEST
version: "0.1"
status: draft
sourceSpecId: BUGRAIL-SPECOS-022
sourceSpecVersion: "0.1"
sourceSpecHash: "971ee1bb505ad662c40f8570e44f13a1b16740e398c8e6628cf62076733af38e"
independentFromImplementation: true
---

# Test Spec: Team Retry And Agent Reassignment

## Test Cases

| ID | Covers | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-022.T01` | R01,R05 | Retry one failed node beside a successful sibling. | One new `run_seq`; sibling generation unchanged. |
| `BUGRAIL-SPECOS-022.T02` | R02,R06 | Reassign next generation to another enabled Team member. | Old/new immutable resolution and reason retained. |
| `BUGRAIL-SPECOS-022.T03` | R03 | Race duplicate retry/reassign requests. | At most one active generation and one effective decision. |
| `BUGRAIL-SPECOS-022.T04` | R04 | Retry an upstream node after a downstream node consumed it. | Downstream becomes explicit stale/blocked under the declared policy. |
| `BUGRAIL-SPECOS-022.T05` | R02 | Reassign to missing, disabled or non-member profile. | Typed rejection and no generation/config mutation. |
| `BUGRAIL-SPECOS-022.T06` | R01-R06 | Restart after retry/reassignment. | Generation provenance and invalidation projection are unchanged. |

## Evidence

Use WorkTask claim concurrency tests, dependency-generation fixtures, Agent
resolution assertions and transport/UI tests. A Team-specific attempt table is
not an acceptable oracle.

