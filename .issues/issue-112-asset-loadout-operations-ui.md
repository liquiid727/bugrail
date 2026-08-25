---
id: issue-112
title: "Asset loadout operations and UI"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-035
sourceSpecVersion: "0.1"
sourceSpecHash: "0c2c2e143ff2dd5bbeabd2f8aeb39ff9227e605b2b71c15ef690dcd7bcd00486"
requirements: [BUGRAIL-SPECOS-035.R05, BUGRAIL-SPECOS-035.R06]
dependsOn: [issue-110, issue-111]
---

# Asset loadout operations and UI

## Outcome

Add preset editing and effective loadout/denial inspection through shared
command-core, Tauri/Axum and Agent/Context UI.

## Scope

Frontend controls are explanatory only; tampered requests must receive the
same backend-enforced policy.

## Verification

Cover `BUGRAIL-SPECOS-035.T05-T06` and all visible resolution/error states.
