---
id: issue-083
title: "Durable provider jobs and restart recovery"
status: verified
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-028
sourceSpecVersion: "0.1"
sourceSpecHash: "f4b2884897e864cc28022536fd5b61d05eb2e7fffbee096154bd5bb308cc8c4a"
requirements: [BUGRAIL-SPECOS-028.R04, BUGRAIL-SPECOS-028.R05]
dependsOn: [issue-082]
---

# Durable provider jobs and restart recovery

## Outcome

Persist bounded idempotent external provider jobs with leasing, retry and
restart recovery using SQLite facts and existing refresh events.

## Scope

Do not replace WorkTask, Automation, EventEmitter or Memory capture delivery.

## Verification

Cover `BUGRAIL-SPECOS-028.T03`, including duplicate submission, crash and
retry exhaustion.
