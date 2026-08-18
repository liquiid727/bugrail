---
id: BUGRAIL-SPECOS-008
version: "0.2"
title: "Agent Context Loadouts"
status: approved
changeType: agent-team-context-deepening
prd: ".prd/prd-specos-agent-team-context-system.md"
design: "design/specos-control-plane-design.md"
clientDesign: "design/specos-client-interaction-design.md"
dependsOn: [BUGRAIL-SPECOS-002, BUGRAIL-SPECOS-006, BUGRAIL-SPECOS-007]
---

# BUGRAIL-SPECOS-008: Agent Context Loadouts

## 1. Summary

This vertical slice adapts the two 2026-08-12 source proposals to BugRail's
existing WorkTask/ACP/Worktree/SQLite/transport architecture. It deepens those
modules rather than introducing a parallel runtime.

## 2. Requirements

| ID | Requirement |
|---|---|
| BUGRAIL-SPECOS-008.R01 | A versioned project Context configuration defines named loadouts and one default loadout. |
| BUGRAIL-SPECOS-008.R02 | A loadout declares ordered local sources, selected Provider IDs, required/optional semantics, and item/byte/token budgets. |
| BUGRAIL-SPECOS-008.R03 | Agent Profile, workflow node, and task/run override may select a loadout using the same explicit precedence model as runtime resolution. |
| BUGRAIL-SPECOS-008.R04 | Compilation canonicalizes project-local files, rejects escape, deduplicates by content hash, enforces budgets, and stores one immutable package per task/run. |
| BUGRAIL-SPECOS-008.R05 | Required source/provider failures block launch; optional unavailable or over-budget items remain attributable degradation/exclusion. |
| BUGRAIL-SPECOS-008.R06 | Prompt composition consumes only the stored package and preserves existing WorkTask safety/stage instructions. |

## 3. Architecture And Placement

Extend context::prepare_run as a deterministic compiler invoked by work_task::engine after Worktree creation and before ACP spawn. Store work_task_context_pack/item and bind its ID to work_task_run in one transaction.

### Command/API contract

Configuration uses specos_context_config_get/save; exact packages use specos_context_package_get. Package content is available only to authenticated project clients.

### Data and migration

New runtime facts use additive SeaORM migrations with foreign keys and indexed
lookup keys. Git-trackable definitions remain files; existing task rows and
legacy configuration are not rewritten. Down migration drops dependent rows
before parent projections.

## 4. Client Interaction

Context settings edit the default loadout, sources, providers, and budgets; task detail shows the exact package ID, items, hashes, status and estimates.

Backend projections are authoritative. UI disablement is never enforcement,
and live events are refresh hints only.

## 5. Failure, Security And Compatibility

Canonical project boundary, UTF-8/file-only sources, byte/item/token caps, content hashing, no secrets/full environment, immutable generation binding.

Errors are typed and leave the previous valid definition or WorkTask state
unchanged. Existing unprofiled tasks, ACP adapters, commands and routes remain
compatible.

## 6. Acceptance Criteria

| ID | Criterion |
|---|---|
| BUGRAIL-SPECOS-008.AC01 | Same ordered inputs produce the same aggregate/package item hashes. |
| BUGRAIL-SPECOS-008.AC02 | Required missing or over-budget sources block before ACP spawn. |
| BUGRAIL-SPECOS-008.AC03 | Optional missing files do not block and are not fabricated. |
| BUGRAIL-SPECOS-008.AC04 | Retry creates a new package while repeated reads of one run return its immutable package. |
| BUGRAIL-SPECOS-008.AC05 | Task detail covers empty, ready, degraded and error package states. |

## 7. Verification

The matching test-spec.md independently covers schema/validation, happy, error,
edge, concurrency/restart, security, legacy, Tauri/Axum parity, and UI states.
Completion requires persisted or command-captured evidence; implementation
output alone is not verification.

