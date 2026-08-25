---
id: issue-094
title: "Task context artifact offload"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-031
sourceSpecVersion: "0.1"
sourceSpecHash: "537c3e75a4432ce8b56a87656198a990ba7b98eed55eb1ec9f1b8fb0e317ddf4"
requirements: [BUGRAIL-SPECOS-031.R01, BUGRAIL-SPECOS-031.R02, BUGRAIL-SPECOS-031.R05]
dependsOn: [issue-081, issue-085, issue-089]
---

# Task context artifact offload

## Outcome

Persist oversized tool/terminal/file evidence atomically under task/run scope
and replace active output with stable bounded references and safe summaries.

## Scope

Use explicit thresholds, hashes, retention and quota; never silently truncate
or put active task state inside Memory.

## Verification

Cover `BUGRAIL-SPECOS-031.T01-T02`, `T04` artifact security and `T05` cleanup.
