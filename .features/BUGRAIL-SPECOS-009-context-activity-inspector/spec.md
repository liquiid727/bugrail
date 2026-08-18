---
id: BUGRAIL-SPECOS-009
version: "0.2"
title: "Context Activity And Inspector"
status: approved
changeType: agent-team-context-deepening
prd: ".prd/prd-specos-agent-team-context-system.md"
design: "design/specos-control-plane-design.md"
clientDesign: "design/specos-client-interaction-design.md"
dependsOn: [BUGRAIL-SPECOS-006, BUGRAIL-SPECOS-007, BUGRAIL-SPECOS-008]
---

# BUGRAIL-SPECOS-009: Context Activity And Inspector

## 1. Summary

This vertical slice adapts the two 2026-08-12 source proposals to BugRail's
existing WorkTask/ACP/Worktree/SQLite/transport architecture. It deepens those
modules rather than introducing a parallel runtime.

## 2. Requirements

| ID | Requirement |
|---|---|
| BUGRAIL-SPECOS-009.R01 | Context is a first-level workbench route showing configuration, Provider health, recent immutable packages, and activity. |
| BUGRAIL-SPECOS-009.R02 | Every package and activity entry is attributable to project, task/run/package/provider where applicable, status, timestamp, and bounded message. |
| BUGRAIL-SPECOS-009.R03 | Task Detail joins the selected run to its exact Context Package and displays source, required flag, content hash, budget and status. |
| BUGRAIL-SPECOS-009.R04 | Refresh failure preserves last successfully loaded data with an explicit stale/degraded banner and retry action. |
| BUGRAIL-SPECOS-009.R05 | The route and inspector cover no workspace, loading, empty, ready, degraded, blocked, stale and transport-error states across all locales. |

## 3. Architecture And Placement

Expose read projections from context_activity and immutable package tables. UI uses the existing typed transport and workbench route registry; live events may refresh but do not create evidence.

### Command/API contract

specos_context_overview(folder_id) returns validated config, Provider health, recent packages and bounded activity. specos_context_package_get(id) returns exact items.

### Data and migration

New runtime facts use additive SeaORM migrations with foreign keys and indexed
lookup keys. Git-trackable definitions remain files; existing task rows and
legacy configuration are not rewritten. Down migration drops dependent rows
before parent projections.

## 4. Client Interaction

The page uses compact health cards, loadout controls, package/activity lists and progressive disclosure. It remains a normal responsive workbench page, not the external Provider panel.

Backend projections are authoritative. UI disablement is never enforcement,
and live events are refresh hints only.

## 5. Failure, Security And Compatibility

Bound list sizes, authenticated folder ownership checks, redacted messages, hash/path metadata first, and no cross-project package lookup.

Errors are typed and leave the previous valid definition or WorkTask state
unchanged. Existing unprofiled tasks, ACP adapters, commands and routes remain
compatible.

## 6. Acceptance Criteria

| ID | Criterion |
|---|---|
| BUGRAIL-SPECOS-009.AC01 | Context route is keyboard reachable and works with narrow layout. |
| BUGRAIL-SPECOS-009.AC02 | Health, packages and activity survive process restart. |
| BUGRAIL-SPECOS-009.AC03 | Task detail opens the package attached to the selected run rather than latest mutable configuration. |
| BUGRAIL-SPECOS-009.AC04 | Transient refresh failure preserves last-good content with an explicit warning. |
| BUGRAIL-SPECOS-009.AC05 | All ten locale catalogs and empty/loading/error states render without missing keys. |

## 7. Verification

The matching test-spec.md independently covers schema/validation, happy, error,
edge, concurrency/restart, security, legacy, Tauri/Axum parity, and UI states.
Completion requires persisted or command-captured evidence; implementation
output alone is not verification.

