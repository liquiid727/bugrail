---
id: BUGRAIL-SPECOS-028
version: "0.1"
title: "Independent Context Plugin Foundation"
status: approved
changeType: context-plugin-foundation
prd: ".prd/prd-memory-operating-layer-roadmap.md"
design: ".features/bugrail-specoos-memory/02-ARCH-系统架构与插件边界.md"
codeBaseline: "2ab6d5cf"
dependsOn: [BUGRAIL-SPECOS-006, BUGRAIL-SPECOS-007, BUGRAIL-SPECOS-008, BUGRAIL-SPECOS-009, BUGRAIL-SPECOS-017]
---

# BUGRAIL-SPECOS-028: Independent Context Plugin Foundation

## 1. Outcome

Establish independent Memory, Wiki, CodeGraph and Skill plugin contracts while
reusing the existing Context compiler, AppState, EventEmitter, SQLite and
Tauri/Axum command-core patterns. No dynamic code loading is introduced.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-028.R01` | Memory, Wiki, CodeGraph and Skill expose separate interfaces and manifests; no Adapter implements a catch-all asset interface. |
| `BUGRAIL-SPECOS-028.R02` | One vendor-neutral `AssetRef`/scope/provenance envelope maps plugin results into existing Context candidates without exposing vendor DTOs. |
| `BUGRAIL-SPECOS-028.R03` | A static backend registry owns enable/disable, capabilities, health and Adapter construction; renderer and Agent callers cannot select arbitrary implementations. |
| `BUGRAIL-SPECOS-028.R04` | External asynchronous plugin work uses a durable, bounded provider-job repository with idempotency, retry and restart recovery; it does not create another WorkTask or Automation state machine. |
| `BUGRAIL-SPECOS-028.R05` | Persisted configuration/job facts are authoritative and existing events are refresh hints only. |
| `BUGRAIL-SPECOS-028.R06` | Configuration validation, health and job inspection have equivalent Tauri/Axum command-core behavior and redact runtime secrets. |

## 3. Existing Modules

- Deepen `context`, `specos_control` and existing Provider configuration.
- Reuse `AppState`, `EventEmitter`, SQLite migrations and command-core handlers.
- Preserve `memory::MemoryProvider` unchanged except for shared envelope types.
- Reuse existing retry/outbox patterns; do not layer provider jobs over
  `memory_capture_delivery`.

## 4. Interface

Plugin manifests declare exactly one primary kind: `memory`, `wiki`,
`codegraph` or `skill`. Internal deterministic Adapters exercise each contract.
Production Adapter selection is a static allowlist. `AssetRef` contains stable
ID, kind, scope, provider, revision and safe provenance; content remains in the
owning plugin result until Context selection.

## 5. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-028.AC01` | Registering four deterministic Adapters yields four isolated capability/health projections and no cross-kind method access. |
| `BUGRAIL-SPECOS-028.AC02` | Wiki, CodeGraph and Skill candidates pass through existing Context budget/dedup/provenance rules using the shared envelope. |
| `BUGRAIL-SPECOS-028.AC03` | Duplicate job submission is idempotent, retry is bounded and restart recovers running jobs without duplicate external mutation. |
| `BUGRAIL-SPECOS-028.AC04` | Invalid manifests/configuration fail before Adapter construction and expose no credential value. |
| `BUGRAIL-SPECOS-028.AC05` | Desktop/server operations return equivalent persisted projections; missed events do not lose state. |

## 6. Non-Goals

No dynamic binary marketplace, arbitrary in-process library loading, new event
bus, new scheduler, Memory CRUD, Wiki indexing, graph indexing or Skill
promotion is implemented here.
