---
id: issue-086
title: "Pinned TencentDB runtime and secure installation"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-029
sourceSpecVersion: "0.1"
sourceSpecHash: "bf4396c4bec625f2501b7499c00d8ec3b97b7194c1ecdb69d8d9b1914e5a634c"
requirements: [BUGRAIL-SPECOS-029.R01, BUGRAIL-SPECOS-029.R03]
dependsOn: [issue-081, issue-085]
---

# Pinned TencentDB runtime and secure installation

## Outcome

Add the signed/checksummed Memory runtime manifest, managed cache/install flow
and backend secret references while preserving explicit remote mode.

## Scope

Pin the exact `017` Memory contract; do not bundle Wiki, CodeGraph or Skill as
Memory capabilities.

## Verification

Cover `BUGRAIL-SPECOS-029.T01` and secret scanning from `T06`.
