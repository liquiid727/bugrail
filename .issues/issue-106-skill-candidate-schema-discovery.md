---
id: issue-106
title: "Skill schema and candidate discovery"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-034
sourceSpecVersion: "0.1"
sourceSpecHash: "af88e70654e54f0f1d7b4d1ba89d92885ea9fabe31f607d06b44168644e1bcd2"
requirements: [BUGRAIL-SPECOS-034.R01, BUGRAIL-SPECOS-034.R02, BUGRAIL-SPECOS-034.R03]
dependsOn: [issue-081, issue-085]
---

# Skill schema and candidate discovery

## Outcome

Extend existing custom Skills with versioned provenance/recovery fields and
derive deduplicated candidates only from repeated attributable run evidence.

## Scope

One trace cannot publish; Memory atoms and copied asset snapshots are not Skills.

## Verification

Cover `BUGRAIL-SPECOS-034.T01` and candidate lifecycle foundations from `T02`.
