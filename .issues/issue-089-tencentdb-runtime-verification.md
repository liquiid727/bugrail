---
id: issue-089
title: "Independently verify managed TencentDB runtime"
status: planned
kind: verification
sourceSpecId: BUGRAIL-SPECOS-029
sourceSpecVersion: "0.1"
sourceSpecHash: "bf4396c4bec625f2501b7499c00d8ec3b97b7194c1ecdb69d8d9b1914e5a634c"
requirements: []
dependsOn: [issue-086, issue-087, issue-088]
---

# Independently verify managed TencentDB runtime

## Outcome

Execute exact Test Spec `BUGRAIL-SPECOS-029-TEST` against the signed local
fixture and remote mode, retaining pin, process, migration and secret evidence.

## Scope

Manual startup, `/health` alone and moving upstream builds are invalid proof.
