---
id: issue-088
title: "TencentDB runtime migration and operations UI"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-029
sourceSpecVersion: "0.1"
sourceSpecHash: "bf4396c4bec625f2501b7499c00d8ec3b97b7194c1ecdb69d8d9b1914e5a634c"
requirements: [BUGRAIL-SPECOS-029.R05, BUGRAIL-SPECOS-029.R06]
dependsOn: [issue-087]
---

# TencentDB runtime migration and operations UI

## Outcome

Add mutually exclusive backup/restore/migrate/rollback operations and safe
settings/diagnostic projections across desktop and server modes.

## Scope

Every operation requires preflight, durable audit facts and rollback state;
the UI is never operation authority.

## Verification

Cover `BUGRAIL-SPECOS-029.T04-T06`, including race and last-good states.
