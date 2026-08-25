---
id: issue-107
title: "Skill validation, publication and routing"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-034
sourceSpecVersion: "0.1"
sourceSpecHash: "af88e70654e54f0f1d7b4d1ba89d92885ea9fabe31f607d06b44168644e1bcd2"
requirements: [BUGRAIL-SPECOS-034.R03, BUGRAIL-SPECOS-034.R04, BUGRAIL-SPECOS-034.R05, BUGRAIL-SPECOS-034.R06]
dependsOn: [issue-106]
---

# Skill validation, publication and routing

## Outcome

Implement constrained generation-safe validation, review/publish/disable/
rollback and explainable scoped Top-K routing with `AssetRef` dependencies.

## Scope

Validation cannot publish; rejected/disabled/unauthorized Skills never enter a
Context Package.

## Verification

Cover `BUGRAIL-SPECOS-034.T02-T05` across crash, malicious content and rollback.
