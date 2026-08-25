---
id: issue-090
title: "Memory search and governance overlay"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-030
sourceSpecVersion: "0.1"
sourceSpecHash: "35429d89b8f40478dd1775daa4608a1c29adf32355e3d39ac2775e6c16528eea"
requirements: [BUGRAIL-SPECOS-030.R01, BUGRAIL-SPECOS-030.R02, BUGRAIL-SPECOS-030.R06]
dependsOn: [issue-081, issue-085, issue-089]
---

# Memory search and governance overlay

## Outcome

Extend the deep Memory interface with bounded scoped search/get and persist a
local validity/supersession/conflict/TTL overlay without mirroring content.

## Scope

Backend scope enforcement precedes Adapter access; SQLite stores governance
and evidence metadata only.

## Verification

Cover `BUGRAIL-SPECOS-030.T01` and policy foundations from `T04`.
