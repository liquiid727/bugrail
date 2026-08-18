---
id: BUGRAIL-SPECOS-002
version: "0.2"
title: "Agent And Model Profiles"
status: approved
changeType: agent-team-context-deepening
prd: ".prd/prd-specos-agent-team-context-system.md"
design: "design/specos-control-plane-design.md"
clientDesign: "design/specos-client-interaction-design.md"
dependsOn: [BUGRAIL-SPECOS-001]
---

# BUGRAIL-SPECOS-002: Agent And Model Profiles

## 1. Summary

This vertical slice adapts the two 2026-08-12 source proposals to BugRail's
existing WorkTask/ACP/Worktree/SQLite/transport architecture. It deepens those
modules rather than introducing a parallel runtime.

## 2. Requirements

| ID | Requirement |
|---|---|
| BUGRAIL-SPECOS-002.R01 | Project-scoped Model Profiles and Agent Profiles are schema-versioned, validated, Git-trackable definitions under .codeg/agents.yaml. |
| BUGRAIL-SPECOS-002.R02 | Agent identity is independent of model and runtime adapter; multiple Agent Profiles may reference one Model Profile. |
| BUGRAIL-SPECOS-002.R03 | Resolution precedence is task/run override, workflow node override, Agent Profile, project default, then legacy WorkTask/folder behavior. |
| BUGRAIL-SPECOS-002.R04 | A resolved run stores an immutable, redacted snapshot of Agent/model/mode/reasoning/profile identifiers and reason codes before prompt dispatch. |
| BUGRAIL-SPECOS-002.R05 | Invalid references, duplicate IDs, unknown adapters, and secret-like inline values are rejected without replacing the last valid catalog. |
| BUGRAIL-SPECOS-002.R06 | Unprofiled legacy WorkTasks execute with existing ACP configuration and remain readable. |

## 3. Architecture And Placement

Add a project registry in specos_control, a pure resolver in agent_runtime, optional profile identifiers on WorkTaskConfig, and resolution columns on work_task_run. The resolver returns the existing ACP agent_type plus allowlisted config values; it never calls a model Provider.

### Command/API contract

specos_agent_catalog_get/save(folder_id) and the existing task launch path. Tauri and Axum call the same core functions.

### Data and migration

New runtime facts use additive SeaORM migrations with foreign keys and indexed
lookup keys. Git-trackable definitions remain files; existing task rows and
legacy configuration are not rewritten. Down migration drops dependent rows
before parent projections.

## 4. Client Interaction

Teams exposes empty/starter, editable profiles, invalid definitions, save success, and transport failure. The run inspector shows the immutable resolved identity rather than current mutable profile values.

Backend projections are authoritative. UI disablement is never enforcement,
and live events are refresh hints only.

## 5. Failure, Security And Compatibility

Reject symlinked .codeg configuration, path escape, duplicate IDs, missing references, and secret-like keys. Writes use validate-then-atomic-rename.

Errors are typed and leave the previous valid definition or WorkTask state
unchanged. Existing unprofiled tasks, ACP adapters, commands and routes remain
compatible.

## 6. Acceptance Criteria

| ID | Criterion |
|---|---|
| BUGRAIL-SPECOS-002.AC01 | Two Agent Profiles referencing one Model Profile remain distinct and resolve to independent profile IDs. |
| BUGRAIL-SPECOS-002.AC02 | Resolution precedence and reason codes are deterministic and persisted before ACP prompt dispatch. |
| BUGRAIL-SPECOS-002.AC03 | Invalid catalogs do not replace the last valid file and surface actionable validation errors. |
| BUGRAIL-SPECOS-002.AC04 | Legacy tasks preserve current adapter/model behavior. |
| BUGRAIL-SPECOS-002.AC05 | Desktop/server responses and all profile UI states are equivalent. |

## 7. Verification

The matching test-spec.md independently covers schema/validation, happy, error,
edge, concurrency/restart, security, legacy, Tauri/Axum parity, and UI states.
Completion requires persisted or command-captured evidence; implementation
output alone is not verification.

