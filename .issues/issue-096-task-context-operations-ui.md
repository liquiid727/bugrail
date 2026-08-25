---
id: issue-096
title: "Task context operations and UI"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-031
sourceSpecVersion: "0.1"
sourceSpecHash: "537c3e75a4432ce8b56a87656198a990ba7b98eed55eb1ec9f1b8fb0e317ddf4"
requirements: [BUGRAIL-SPECOS-031.R05, BUGRAIL-SPECOS-031.R06]
dependsOn: [issue-094, issue-095]
---

# Task context operations and UI

## Outcome

Expose offload volume, refs, canvas, checkpoints, retention and resume states
through task command-core, Tauri/Axum and existing Tasks UI.

## Scope

Support Memory degradation and retain last-good task facts; renderer cannot
delete protected evidence or authorize resume.

## Verification

Cover `BUGRAIL-SPECOS-031.T05-T06` and visible error/loading/empty states.
