---
id: issue-108
title: "Skill Evolution operations and candidate UI"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-034
sourceSpecVersion: "0.1"
sourceSpecHash: "af88e70654e54f0f1d7b4d1ba89d92885ea9fabe31f607d06b44168644e1bcd2"
requirements: [BUGRAIL-SPECOS-034.R03, BUGRAIL-SPECOS-034.R07]
dependsOn: [issue-106, issue-107]
---

# Skill Evolution operations and candidate UI

## Outcome

Expose inbox, diff, evidence, validation, publish, disable and rollback through
shared command-core, Tauri/Axum and existing Skill UI.

## Scope

Retain last-good version/lifecycle facts and require backend authorization for
all state transitions.

## Verification

Cover `BUGRAIL-SPECOS-034.T06` and visible validation/security/error states.
