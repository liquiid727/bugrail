---
id: BUGRAIL-SPECOS-007
version: "0.2"
title: "Context Provider Bootstrap"
status: approved
changeType: agent-team-context-deepening
prd: ".prd/prd-specos-agent-team-context-system.md"
design: "design/specos-control-plane-design.md"
clientDesign: "design/specos-client-interaction-design.md"
dependsOn: [BUGRAIL-SPECOS-006]
---

# BUGRAIL-SPECOS-007: Context Provider Bootstrap

## 1. Summary

This vertical slice adapts the two 2026-08-12 source proposals to BugRail's
existing WorkTask/ACP/Worktree/SQLite/transport architecture. It deepens those
modules rather than introducing a parallel runtime.

## 2. Requirements

| ID | Requirement |
|---|---|
| BUGRAIL-SPECOS-007.R01 | Context Providers are project definitions with stable ID, kind, endpoint reference, credential reference, capabilities, enabled and required flags. |
| BUGRAIL-SPECOS-007.R02 | CodeG performs bounded provider health/capability discovery through an adapter boundary before context compilation. |
| BUGRAIL-SPECOS-007.R03 | TencentDB Agent Memory is represented as the first remote bootstrap provider without leaking its API into Agent Profiles, ACP adapters, or prompt code. |
| BUGRAIL-SPECOS-007.R04 | Unavailable required providers block before prompt dispatch; unavailable optional providers produce explicit degraded health/activity. |
| BUGRAIL-SPECOS-007.R05 | Provider credentials are resolved from environment/keychain references and never persisted in definitions, run snapshots, activity, logs, or UI. |

## 3. Architecture And Placement

The context module owns health and capability normalization; Provider-specific protocol behavior stays behind that module. Phase 1 uses local sources plus a Tencent-compatible /v3/tools/list health seam. Remote search/write is not claimed by this slice.

### Command/API contract

specos_context_config_get/save, specos_context_overview, and Provider health in the Context DTO. Provider failures use typed validation/transport errors.

### Data and migration

New runtime facts use additive SeaORM migrations with foreign keys and indexed
lookup keys. Git-trackable definitions remain files; existing task rows and
legacy configuration are not rewritten. Down migration drops dependent rows
before parent projections.

## 4. Client Interaction

Context Overview renders healthy, disabled, degraded and required-blocked Provider cards, retry/refresh, and last-good data during transient failure.

Backend projections are authoritative. UI disablement is never enforcement,
and live events are refresh hints only.

## 5. Failure, Security And Compatibility

Five-second network timeout, credential references only, redacted errors, no arbitrary response persistence, and no direct Agent-to-Provider call.

Errors are typed and leave the previous valid definition or WorkTask state
unchanged. Existing unprofiled tasks, ACP adapters, commands and routes remain
compatible.

## 6. Acceptance Criteria

| ID | Criterion |
|---|---|
| BUGRAIL-SPECOS-007.AC01 | Local provider is healthy without network access. |
| BUGRAIL-SPECOS-007.AC02 | A Tencent-compatible healthy endpoint is normalized without exposing credentials. |
| BUGRAIL-SPECOS-007.AC03 | Required provider failure prevents launch before ACP prompt dispatch. |
| BUGRAIL-SPECOS-007.AC04 | Optional provider failure creates degraded package/activity rather than silent omission. |
| BUGRAIL-SPECOS-007.AC05 | Restart and Tauri/Axum reads report the same persisted/configured facts. |

## 7. Verification

The matching test-spec.md independently covers schema/validation, happy, error,
edge, concurrency/restart, security, legacy, Tauri/Axum parity, and UI states.
Completion requires persisted or command-captured evidence; implementation
output alone is not verification.

