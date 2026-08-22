---
id: BUGRAIL-SPECOS-021-TEST
version: "0.1"
status: draft
sourceSpecId: BUGRAIL-SPECOS-021
sourceSpecVersion: "0.1"
sourceSpecHash: "a25f3d501f13a619eed343eeb5ab97e21d9c4e1e355ccc0422c997f0bb3531e5"
independentFromImplementation: true
---

# Test Spec: Team Quality And Finalization

## Test Cases

| ID | Covers | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-021.T01` | R01,R04 | Reviewer addresses all, omits one, and rejects one acceptance ID. | Valid mapping only when complete; rejection remains blocking. |
| `BUGRAIL-SPECOS-021.T02` | R02,R03 | Fail required test/preflight/human gates. | Team cannot finalize; existing WorkTask gate decision is unchanged. |
| `BUGRAIL-SPECOS-021.T03` | R03 | Attempt finalization from Agent output and direct stale client state. | No authoritative status change. |
| `BUGRAIL-SPECOS-021.T04` | R05,R06 | Finalize a successful run with diffs, commits, gates and review. | Immutable report contains references, not copied transcripts/blobs. |
| `BUGRAIL-SPECOS-021.T05` | R05 | Restart/reconnect and load the report. | Same decision/hash and referenced evidence. |
| `BUGRAIL-SPECOS-021.T06` | R01-R06 | Render parity and accessibility states. | Equivalent Tauri/Axum facts and keyboard-readable acceptance summary. |

## Evidence

Use WorkTask gate/Git fixtures, fake reviewer output, finalization CAS tests and
Teams/Task Detail rendering tests. Every acceptance result must reference its
persisted evidence oracle.

