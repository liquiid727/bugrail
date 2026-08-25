---
id: issue-110
title: "Agent asset loadout resolution"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-035
sourceSpecVersion: "0.1"
sourceSpecHash: "0c2c2e143ff2dd5bbeabd2f8aeb39ff9227e605b2b71c15ef690dcd7bcd00486"
requirements: [BUGRAIL-SPECOS-035.R01, BUGRAIL-SPECOS-035.R05]
dependsOn: [issue-045, issue-058, issue-085, issue-093, issue-101, issue-105, issue-109]
---

# Agent asset loadout resolution

## Outcome

Resolve named per-Agent plugin enablement, budgets and query policy once per
WorkTask generation and persist exact revisions/exclusions in Context evidence.

## Scope

Deepen existing Agent Profiles/Context Loadouts; add no identity service or
mutable historical policy.

## Verification

Cover `BUGRAIL-SPECOS-035.T01` and restart/config revision from `T05`.
