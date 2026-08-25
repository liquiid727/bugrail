---
id: issue-098
title: "Wiki provider contract and source registry"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-032
sourceSpecVersion: "0.1"
sourceSpecHash: "44862b315a4620bb49f998eccc3083ea7d18659484f464602e1be0eedb9b64d0"
requirements: [BUGRAIL-SPECOS-032.R01, BUGRAIL-SPECOS-032.R02, BUGRAIL-SPECOS-032.R05]
dependsOn: [issue-085, issue-089]
---

# Wiki provider contract and source registry

## Outcome

Add an independent `WikiProvider`, canonical scoped source registry and pinned
production Adapter contract even when it shares the TencentDB deployment.

## Scope

Memory has no Wiki methods; source files remain authoritative and path-confined.

## Verification

Cover `BUGRAIL-SPECOS-032.T01` and source/security setup from `T04`.
