---
id: issue-079
title: "Memory recall and Context Package integration"
status: verified
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


## Completion Record

- Implementation landed in `feat(memory): recall integration into context
  packages` (Context recall normalization, fixed package order, budget and
  provenance evidence, required/optional failure semantics).
- T04/T05 oracles pass in `tests/memory_recall_context.rs` and the pinned
  T08 run (`tests/results/2026-08-22-specos-017-memory-verification.md`)
  demonstrates capture-to-later-recall after restart.
- `issue-081` accepted this exact Feature hash; required/optional failure,
  immutable provenance and budget behavior are verified.
- Spec deviation: none.
