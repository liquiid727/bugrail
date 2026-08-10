---
id: issue-009
title: "Derive and execute verification for WorkTask run evidence"
status: draft
kind: verification
type: fullstack
priority: high
sourceSpecId: BUGRAIL-SPECOS-002
sourceSpecVersion: "0.1"
sourceSpecHash: "4d2c623f35a9caea2d66aee7f716bb8ca41c451a1c1d33b92c763e6dcda87965"
requirements: [BUGRAIL-SPECOS-002.R01, BUGRAIL-SPECOS-002.R02, BUGRAIL-SPECOS-002.R03, BUGRAIL-SPECOS-002.R04, BUGRAIL-SPECOS-002.R05]
dependsOn: [issue-006, issue-007, issue-008]
---

# Derive And Execute Verification For WorkTask Run Evidence

## Scope

- Independently derive a version/hash-bound `test-spec.md` for AC01–AC07.
- Test crash/restart, concurrent claim, legacy events, pending usage, pagination,
  redaction, transport parity, and every client state.
- Run existing WorkTask, Conversation/token, desktop, and server regressions.
- Store normalized evidence under `tests/results/` with source Spec hash.

## Acceptance Criteria

- Each AC has at least one independent blocking result.
- No raw prompt, transcript, provider key, environment value, or uncapped output
  appears in database, wire snapshots, UI, screenshots, or test logs.
- Run attribution survives restart and never guesses legacy ownership.
- Failures, skipped checks, and flaky retries remain visible and blocking.
