---
id: issue-100
title: "Wiki operations and browsing UI"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-032
sourceSpecVersion: "0.1"
sourceSpecHash: "44862b315a4620bb49f998eccc3083ea7d18659484f464602e1be0eedb9b64d0"
requirements: [BUGRAIL-SPECOS-032.R02, BUGRAIL-SPECOS-032.R03, BUGRAIL-SPECOS-032.R06]
dependsOn: [issue-098, issue-099]
---

# Wiki operations and browsing UI

## Outcome

Expose source management, sync/rebuild, page/search/citation and stale/error
states through shared command-core, Tauri/Axum and Wiki UI.

## Scope

Retain last-good results and safe citation navigation; renderer never reads
arbitrary paths or calls provider endpoints.

## Verification

Cover `BUGRAIL-SPECOS-032.T05` and all visible lifecycle states.
