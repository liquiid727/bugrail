---
id: BUGRAIL-SPECOS-023-TEST
version: "0.1"
status: draft
sourceSpecId: BUGRAIL-SPECOS-023
sourceSpecVersion: "0.1"
sourceSpecHash: "8da3ccb94bb90c3edc588e9443088f4286472bffc15b70e74dbfe68084529e98"
independentFromImplementation: true
---

# Test Spec: Team Effective Permission Policy

## Test Cases

| ID | Covers | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-023.T01` | R01-R03 | Read-only reviewer attempts write, commit, merge, network and MCP actions. | Backend denies outside the effective intersection; no side effect. |
| `BUGRAIL-SPECOS-023.T02` | R03 | Attempt project/Worktree path escape and unauthorized secret/context access. | Fail-closed decision and no leaked value. |
| `BUGRAIL-SPECOS-023.T03` | R04 | Approve one bounded escalation, then reuse it elsewhere/after expiry. | Only exact action/run succeeds; later attempts denied. |
| `BUGRAIL-SPECOS-023.T04` | R05 | Inspect allow/deny/escalation audit and logs. | Actor/reason/scope present; secrets and protected content absent. |
| `BUGRAIL-SPECOS-023.T05` | R06 | Run legacy non-Team permission fixtures. | Existing behavior remains compatible. |
| `BUGRAIL-SPECOS-023.T06` | R01-R06 | Compare local and remote/Tauri/Axum paths. | Same backend decision regardless of UI control state. |

## Evidence

Use temporary filesystem, fake ACP tool requests, authenticated command-core
fixtures and secret-redaction assertions. Prompt instructions or disabled UI
controls cannot satisfy enforcement tests.

