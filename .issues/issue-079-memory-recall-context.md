---
id: issue-079
title: "Memory recall and Context Package integration"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-017
sourceSpecVersion: "0.2"
sourceSpecHash: "f62824d31787ff774d962d942ffba0423fe78f28c77fa7b0e2e000d71dacfd58"
requirements: [BUGRAIL-SPECOS-017.R04, BUGRAIL-SPECOS-017.R05]
dependsOn: [issue-057, issue-077]
---

# Memory recall and Context Package integration

## Outcome

Recall L1 and optional L3 memory before ACP prompt dispatch, normalize it into
existing Context candidates and persist exact safe provenance in the immutable
run package.

## Scope

Existing Context budget/dedup/failure behavior remains authoritative. Do not
allow the Adapter to compose prompts or implement L0/L2 authoring.

## Verification

Cover `T04-T05` and remote-content security in `T06`, including deterministic
package hashes, empty recall and restart inspection.

