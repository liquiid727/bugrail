---
id: BUGRAIL-SPECOS-024-TEST
version: "0.1"
status: draft
sourceSpecId: BUGRAIL-SPECOS-024
sourceSpecVersion: "0.1"
sourceSpecHash: "987875354254a53f097d4ac5f63b47844f8440b7a8f3017ef1d036725932841c"
independentFromImplementation: true
---

# Test Spec: Team Usage Accounting And Budget

## Test Cases

| ID | Covers | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-024.T01` | R01,R02 | Aggregate reported tokens/cost across nodes and retries. | Exact once-only total with provider/model/source provenance. |
| `BUGRAIL-SPECOS-024.T02` | R03,R04 | Cross warning then enforceable hard threshold. | One warning; next protected claim blocked before dispatch. |
| `BUGRAIL-SPECOS-024.T03` | R04,R06 | Provider reports delayed, partial or no usage. | Explicit unknown/stale measurement; no fabricated zero/strict guarantee. |
| `BUGRAIL-SPECOS-024.T04` | R05 | Stop with active outputs/Worktrees, adjust budget and resume. | Artifacts preserved; authenticated audited decision. |
| `BUGRAIL-SPECOS-024.T05` | R01,R02 | Replay duplicate provider usage records. | No double count. |
| `BUGRAIL-SPECOS-024.T06` | R03-R06 | Restart/reconnect at a budget blocker. | Same total, blocker and adjustment history. |

## Evidence

Use deterministic usage fixtures, claim-predicate tests, uniqueness assertions
and UI unknown/warning/blocked states. Record pricing/currency fixture revision.

