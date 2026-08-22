---
id: BUGRAIL-SPECOS-018-TEST
version: "0.1"
status: draft
sourceSpecId: BUGRAIL-SPECOS-018
sourceSpecVersion: "0.1"
sourceSpecHash: "5f2cea1216b2fd82ea829e121ef30d5ee3e9eb3588952daeb4b2ac25216b2f8d"
independentFromImplementation: true
---

# Test Spec: Team Runtime Reliability Hardening

## Test Cases

| ID | Covers | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-018.T01` | R01,R02 | Race scheduler claim with pause and cancel barriers. | One valid transition; no post-barrier node launch. |
| `BUGRAIL-SPECOS-018.T02` | R01,R03 | Repeat resume/cancel and control terminal/merging runs. | Typed precondition/idempotent result; unresolved work remains visible. |
| `BUGRAIL-SPECOS-018.T03` | R04 | Start with and without TaskEngine ownership. | No false active run; explicit queued/error disposition. |
| `BUGRAIL-SPECOS-018.T04` | R05,R06 | Restart with done, running, missing-session and missing-worktree nodes. | Reconstructed WorkTask facts and durable recovery audit; no false success. |
| `BUGRAIL-SPECOS-018.T05` | R06 | Execute controls through Tauri/Axum and reconnect. | Equivalent results; persisted fetch is authoritative. |
| `BUGRAIL-SPECOS-018.T06` | R07 | List 100 runs with bounded nodes. | Recorded query-count/latency budget without N+1 growth. |

## Evidence

Use SQLite-backed command-core concurrency tests, TaskEngine recovery fixtures,
Tauri/Axum parity tests and Teams refresh tests. Record source hash, commit,
commands, query count, exit codes and result paths. Screenshots or Agent output
alone cannot satisfy acceptance.

