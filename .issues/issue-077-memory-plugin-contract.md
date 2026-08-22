---
id: issue-077
title: "Memory Plugin contract and TencentDB v3 transport"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-017
sourceSpecVersion: "0.2"
sourceSpecHash: "f62824d31787ff774d962d942ffba0423fe78f28c77fa7b0e2e000d71dacfd58"
requirements: [BUGRAIL-SPECOS-017.R01, BUGRAIL-SPECOS-017.R02, BUGRAIL-SPECOS-017.R07]
dependsOn: [issue-054]
---

# Memory Plugin contract and TencentDB v3 transport

## Outcome

Add the deep Memory interface, typed/redacted Provider configuration, strict
identity resolver, deterministic test Adapter and TencentDB `v2.0.0` v3 HTTP
Adapter with stable error classes.

## Scope

Keep Adapter selection static. Do not add Proxy routing, MemoryPanel, dynamic
plugin loading or Wiki/CodeGraph/Skill methods.

## Verification

Cover `T01-T02` and the transport/security portions of `T06`; record the pinned
upstream contract fixture before changing implementation status.


### 2026-08-22 verification

- Evidence: `tests/results/2026-08-22-specos-017-memory-verification.md`
  (spec hash rematched, pinned upstream `v2.0.0` + `bugrail.1` patch).
- T01/T02 plus the T06 transport/security oracles pass against the fake
  Gateway and the pinned T08 fixture, including vanilla-version detection
  and the replay idempotency patch contract.

### 2026-08-23 status

- Code committed through `feat(memory): recall integration into context
  packages`; acceptance of the evidence record is pending review.
