---
id: BUGRAIL-SPECOS-029
version: "0.1"
title: "Managed TencentDB Memory Runtime"
status: draft
changeType: external-runtime-management
prd: ".prd/prd-memory-operating-layer-roadmap.md"
design: ".features/bugrail-specoos-memory/07-IMPLEMENTATION-工程实现与接口.md"
adr: "design/adr/ADR-004-memory-plugin-tencentdb-mvp01.md"
codeBaseline: "2ab6d5cf"
dependsOn: [BUGRAIL-SPECOS-017, BUGRAIL-SPECOS-028]
---

# BUGRAIL-SPECOS-029: Managed TencentDB Memory Runtime

## 1. Outcome

Make the exact TencentDB Agent Memory build required by `017` installable and
operable from BugRail without manual terminal setup while preserving remote
endpoint mode and the existing v3 Memory interface.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-029.R01` | A signed/checksummed runtime manifest pins upstream source, BugRail patch, image/binary digest, schema compatibility and rollback target. |
| `BUGRAIL-SPECOS-029.R02` | One process supervisor owns local install/start/stop/restart and crash recovery with bounded backoff and single-instance locking. |
| `BUGRAIL-SPECOS-029.R03` | Credentials and service identity use backend secure storage; existing environment references remain an explicit compatibility fallback. |
| `BUGRAIL-SPECOS-029.R04` | Startup verifies `/health`, exact writable version and schema compatibility before capture; recall-only degradation remains explicit. |
| `BUGRAIL-SPECOS-029.R05` | Backup, restore, migration and rollback are versioned, mutually exclusive operations with preflight checks and durable audit facts. |
| `BUGRAIL-SPECOS-029.R06` | Settings/diagnostics expose install and lifecycle state through shared Tauri/Axum command-core behavior without upstream DTO leakage. |

## 3. Existing Modules

- Reuse `memory::TencentDbMemoryAdapter`, its exact-version health gate and
  capture outbox.
- Follow the existing managed-binary cache and update transaction patterns.
- Reuse AppState lifecycle and server/desktop startup paths.
- Do not route ACP model traffic through MemoryProxy or embed MemoryPanel.

## 4. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-029.AC01` | A clean machine can install, start and pass the exact-version Memory health gate through BugRail UI/API without a terminal. |
| `BUGRAIL-SPECOS-029.AC02` | Concurrent windows/processes cannot start two local runtimes or race backup/restore/upgrade. |
| `BUGRAIL-SPECOS-029.AC03` | Crash/restart recovers the runtime and capture outbox without duplicate L0 messages. |
| `BUGRAIL-SPECOS-029.AC04` | Unsupported version/schema blocks writes, retains safe diagnostics and can roll back to the prior pin. |
| `BUGRAIL-SPECOS-029.AC05` | Backup followed by destructive fixture mutation and restore reproduces scoped Memory data and identity isolation. |
| `BUGRAIL-SPECOS-029.AC06` | Secrets are absent from config files, logs, audit rows, frontend payloads and diagnostics bundles. |

## 5. Non-Goals

No Memory governance, Wiki, CodeGraph, Skill Evolution, dynamic plugin loading
or provider-owned prompt composition is included.
