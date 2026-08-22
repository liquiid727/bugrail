---
id: issue-084
title: "Plugin configuration, health and job operations"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-028
sourceSpecVersion: "0.1"
sourceSpecHash: "f4b2884897e864cc28022536fd5b61d05eb2e7fffbee096154bd5bb308cc8c4a"
requirements: [BUGRAIL-SPECOS-028.R05, BUGRAIL-SPECOS-028.R06]
dependsOn: [issue-082, issue-083]
---

# Plugin configuration, health and job operations

## Outcome

Expose safe configuration, capability, health and job projections through
shared command-core, Tauri/Axum and existing operational UI patterns.

## Scope

Retain last-good state and reconstruct from persisted facts; never expose
credentials or make renderer-to-provider calls.

## Verification

Cover `BUGRAIL-SPECOS-028.T04-T05` and all visible empty/degraded/error states.
