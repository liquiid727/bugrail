---
id: issue-092
title: "Memory Hub operations and recall history"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-030
sourceSpecVersion: "0.1"
sourceSpecHash: "35429d89b8f40478dd1775daa4608a1c29adf32355e3d39ac2775e6c16528eea"
requirements: [BUGRAIL-SPECOS-030.R04, BUGRAIL-SPECOS-030.R05, BUGRAIL-SPECOS-030.R06]
dependsOn: [issue-090, issue-091]
---

# Memory Hub operations and recall history

## Outcome

Add paginated Memory search/detail/evidence, safe mutation controls and recall
history through shared command-core, Tauri/Axum and UI.

## Scope

Retain last-good data, render remote text as untrusted and never use frontend
filters as authorization.

## Verification

Cover `BUGRAIL-SPECOS-030.T05` plus all mutation and provider-failure states.
