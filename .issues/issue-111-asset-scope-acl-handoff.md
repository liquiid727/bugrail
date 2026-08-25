---
id: issue-111
title: "Asset scope ACL, handoff and A/B isolation"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-035
sourceSpecVersion: "0.1"
sourceSpecHash: "0c2c2e143ff2dd5bbeabd2f8aeb39ff9227e605b2b71c15ef690dcd7bcd00486"
requirements: [BUGRAIL-SPECOS-035.R02, BUGRAIL-SPECOS-035.R03, BUGRAIL-SPECOS-035.R04]
dependsOn: [issue-110]
---

# Asset scope ACL, handoff and A/B isolation

## Outcome

Enforce project/user/team/agent/task/asset intersection before Adapter access,
with attributable task handoff and isolated A/B promotion decisions.

## Scope

Private Memory and unconfirmed variants never become shared implicitly.

## Verification

Cover `BUGRAIL-SPECOS-035.T02-T04`, proving denial before network/index access.
