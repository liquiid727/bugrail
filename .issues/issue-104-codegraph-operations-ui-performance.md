---
id: issue-104
title: "CodeGraph operations, UI and performance"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-033
sourceSpecVersion: "0.1"
sourceSpecHash: "a47f798b27680d8bf313f602abf8d4175396780832ac6d50bddf416ea846801f"
requirements: [BUGRAIL-SPECOS-033.R03, BUGRAIL-SPECOS-033.R05, BUGRAIL-SPECOS-033.R06]
dependsOn: [issue-102, issue-103]
---

# CodeGraph operations, UI and performance

## Outcome

Add index/search/relationship/impact/rebuild UI and transport operations with
last-good state and representative index/query performance budgets.

## Scope

Persist state outside events and prevent N+1 managed process spawning.

## Verification

Cover `BUGRAIL-SPECOS-033.T05-T06` and all visible degraded/error states.
