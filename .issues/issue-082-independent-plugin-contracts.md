---
id: issue-082
title: "Independent plugin contracts and shared AssetRef"
status: verified
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-028
sourceSpecVersion: "0.1"
sourceSpecHash: "f4b2884897e864cc28022536fd5b61d05eb2e7fffbee096154bd5bb308cc8c4a"
requirements: [BUGRAIL-SPECOS-028.R01, BUGRAIL-SPECOS-028.R02, BUGRAIL-SPECOS-028.R03]
dependsOn: [issue-081]
---

# Independent plugin contracts and shared AssetRef

## Outcome

Add separate Memory, Wiki, CodeGraph and Skill contracts, one vendor-neutral
asset envelope and a static validated backend registry.

## Scope

Deepen existing Context/provider modules. Do not add dynamic code loading, a
catch-all TencentDB manifest or vendor DTOs outside Adapters.

## Verification

Cover `BUGRAIL-SPECOS-028.T01-T02` and invalid configuration from `T04`.

## Completion Record

- Implemented separate Memory, Wiki, CodeGraph and Skill contracts, a
  kind-checked `AssetRef` envelope and a static validated registry.
- Deterministic adapters cover the contract while production non-Memory
  adapters remain owned by later Features; Memory retains its existing
  registry compatibility path.
- Verification and review are closed by `issue-085`; source hash matches this
  Issue and the canonical status is `verified`.
- Spec deviation: none; dynamic loading remains explicitly out of scope.
