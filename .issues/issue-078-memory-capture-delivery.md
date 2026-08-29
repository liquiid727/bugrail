---
id: issue-078
title: "WorkTask memory capture delivery"
status: verified
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-017
sourceSpecVersion: "0.2"
sourceSpecHash: "f62824d31787ff774d962d942ffba0423fe78f28c77fa7b0e2e000d71dacfd58"
requirements: [BUGRAIL-SPECOS-017.R03, BUGRAIL-SPECOS-017.R07]
dependsOn: [issue-077]
---

# WorkTask memory capture delivery

## Outcome

Persist idempotent capture deliveries and send filtered, capped user/assistant
messages after durable WorkTask run settlement with restart-safe retry.

## Scope

Capture evidence only. Do not change WorkTask/gate outcomes, mirror remote
memory locally or add remote delete/ACL operations.

## Verification

Cover `T03`, capture failure in `T05`, and capture privacy/bounds in `T06` with
migration, restart and duplicate-delivery oracles.


### 2026-08-22 verification

- Evidence: `tests/results/2026-08-22-specos-017-memory-verification.md`.
- T03 filtering, caps, uniqueness, restart recovery, reconciliation and
  payload clearing pass; the pinned T08 run proves replay never increases
  L0 and legacy projects stage nothing.

## Completion Record

- Verified by `issue-081` against Feature `017` v0.2 and the exact source hash
  in this Issue.
- Evidence: `tests/results/2026-08-22-specos-017-memory-verification.md`.
- Restart recovery, idempotency, privacy bounds, payload clearing and legacy
  zero-behavior passed the accepted T03/T05/T06/T08 matrix.
- Spec deviation: none.
