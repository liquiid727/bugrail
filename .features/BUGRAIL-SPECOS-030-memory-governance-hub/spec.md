---
id: BUGRAIL-SPECOS-030
version: "0.1"
title: "Memory Governance And Hub"
status: draft
changeType: memory-governance-ui
prd: ".prd/prd-memory-operating-layer-roadmap.md"
design: ".features/bugrail-specoos-memory/03-MEMORY-记忆模型召回与上下文.md"
codeBaseline: "2ab6d5cf"
dependsOn: [BUGRAIL-SPECOS-017, BUGRAIL-SPECOS-028, BUGRAIL-SPECOS-029]
---

# BUGRAIL-SPECOS-030: Memory Governance And Hub

## 1. Outcome

Let users inspect, search, correct, invalidate and delete scoped long-term
Memory while keeping TencentDB as durable L0-L3 storage and BugRail as policy,
evidence, injection and UI owner.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-030.R01` | Extend the Memory module with bounded search/get/mutate operations without exposing TencentDB DTOs to Context, commands or UI. |
| `BUGRAIL-SPECOS-030.R02` | A local metadata overlay records validity, supersession, correction, conflict, TTL and audit state without mirroring remote Memory content. |
| `BUGRAIL-SPECOS-030.R03` | Recall suppresses deleted/invalid/superseded/conflicting records before existing Context budget and prompt injection. |
| `BUGRAIL-SPECOS-030.R04` | Every mutation is authenticated, scoped, idempotent and linked to source evidence and remote mutation outcome. |
| `BUGRAIL-SPECOS-030.R05` | Memory Hub provides paginated search, detail/evidence drill-down, correction/delete controls and recall history with last-good/error states. |
| `BUGRAIL-SPECOS-030.R06` | Project/team/agent/user scope isolation is enforced in backend queries and mutations; frontend filters are never authorization. |

## 3. Existing Modules

- Deepen `memory` and existing Context Memory evidence.
- Extend command-core/Tauri/Axum and Context UI navigation.
- Use SQLite only for governance/evidence metadata, not a second Memory Engine.
- Reuse provider jobs for asynchronous remote mutation confirmation.

## 4. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-030.AC01` | Search and detail return bounded scoped records with L0 evidence links and no cross-project leakage. |
| `BUGRAIL-SPECOS-030.AC02` | Correction creates an auditable supersession chain and later recall includes only the effective record. |
| `BUGRAIL-SPECOS-030.AC03` | Delete/invalidate prevents automatic recall even when the upstream result is stale or retrying. |
| `BUGRAIL-SPECOS-030.AC04` | Conflict and TTL policy produce deterministic inclusion/exclusion reasons in immutable Context Packages. |
| `BUGRAIL-SPECOS-030.AC05` | Desktop/server Hub behavior survives restart, pagination and provider failure without exposing secrets or untrusted HTML. |

## 5. Non-Goals

No Wiki, CodeGraph, Skill candidate lifecycle, task offload or upstream ACL
administration is implemented by this Feature.
