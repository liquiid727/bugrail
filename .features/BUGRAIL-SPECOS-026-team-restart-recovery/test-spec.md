---
id: BUGRAIL-SPECOS-026-TEST
version: "0.1"
status: draft
sourceSpecId: BUGRAIL-SPECOS-026
sourceSpecVersion: "0.1"
sourceSpecHash: "ac30329d1a476ed08eaecbe7d9505195bf93f91538326d971298710086c2a0f8"
independentFromImplementation: true
---

# Test Spec: Team Backend Restart Recovery

## Test Cases

| ID | Covers | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-026.T01` | R01 | Start two processes against one data directory. | One TaskEngine owner; non-owner cannot dispatch/recover duplicates. |
| `BUGRAIL-SPECOS-026.T02` | R02,R03 | Restart with done and running nodes. | Done evidence unchanged; unknown active node never becomes success. |
| `BUGRAIL-SPECOS-026.T03` | R02,R04 | Remove Session or Worktree and create an orphan Worktree. | Bounded recovery record; no destructive/fabricated outcome. |
| `BUGRAIL-SPECOS-026.T04` | R02,R05 | Interrupt during merge and retry recovery repeatedly. | Git truth wins; idempotent valid state and no duplicate Session/task. |
| `BUGRAIL-SPECOS-026.T05` | R05,R06 | Recover then recalculate downstream readiness. | Exactly eligible nodes can claim; completed generations stay immutable. |
| `BUGRAIL-SPECOS-026.T06` | R01-R06 | Reconnect Tauri/Web clients after recovery. | Equivalent authoritative projections independent of event loss. |

## Evidence

Use process-ownership fixtures, SQLite/Git/Worktree restart integration tests and
transport reconciliation tests. Record recovery order and durable before/after
facts for every injected failure.

