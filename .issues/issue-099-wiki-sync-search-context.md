---
id: issue-099
title: "Wiki sync, search and Context integration"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-032
sourceSpecVersion: "0.1"
sourceSpecHash: "44862b315a4620bb49f998eccc3083ea7d18659484f464602e1be0eedb9b64d0"
requirements: [BUGRAIL-SPECOS-032.R03, BUGRAIL-SPECOS-032.R04]
dependsOn: [issue-098]
---

# Wiki sync, search and Context integration

## Outcome

Implement full/incremental revisioned indexing, citations, stale publication
and bounded Wiki candidate mapping into existing Context selection.

## Scope

Use durable provider jobs and explicit conflict precedence; do not copy source
snapshots into Memory or Skills.

## Verification

Cover `BUGRAIL-SPECOS-032.T02-T04` with crash, delete and conflict fixtures.
