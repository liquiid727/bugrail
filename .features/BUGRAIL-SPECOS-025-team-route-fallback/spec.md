---
id: BUGRAIL-SPECOS-025
version: "0.1"
title: "Team Provider And Model Route Fallback"
status: draft
changeType: team-routing
prd: ".prd/prd-agent-team-mode-roadmap.md"
design: "design/agent-team-mode-architecture.md"
adr: "design/adr/ADR-002-agent-profile-team-worktask.md"
dependsOn: [BUGRAIL-SPECOS-022, BUGRAIL-SPECOS-023, BUGRAIL-SPECOS-024]
---

# BUGRAIL-SPECOS-025: Team Provider And Model Route Fallback

## 1. Outcome

Retry an eligible infrastructure/provider failure through an explicitly allowed
Agent route while preserving every prior WorkTask generation and policy reason.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-025.R01` | Fallback routes reference validated Model Profiles and remain part of Agent Profile identity rather than creating a new Agent. |
| `BUGRAIL-SPECOS-025.R02` | Only classified retryable route failures trigger automatic fallback; semantic, test, review, permission and budget failures do not unless a policy explicitly authorizes them. |
| `BUGRAIL-SPECOS-025.R03` | Every fallback traverses existing WorkTask retry/generation semantics and records old route, new route, trigger, policy and attempt provenance. |
| `BUGRAIL-SPECOS-025.R04` | Permission and budget evaluation runs again for the candidate route before dispatch. |
| `BUGRAIL-SPECOS-025.R05` | Exhausted or unavailable routes leave an actionable blocked/failed WorkTask without hiding the original error. |
| `BUGRAIL-SPECOS-025.R06` | Legacy fallback profile IDs remain inert metadata until this Feature is enabled and validated. |

## 3. Existing Modules

- Deepen Agent/Model resolution and WorkTask retry paths.
- Reuse provider registries and existing ACP adapters.
- Do not call provider SDKs from Team projection code.

## 4. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-025.AC01` | A classified provider-unavailable failure selects the next allowed route exactly once. |
| `BUGRAIL-SPECOS-025.AC02` | A failing test or reviewer rejection does not silently switch models. |
| `BUGRAIL-SPECOS-025.AC03` | The new generation shows the fallback route and retains the original generation/error. |
| `BUGRAIL-SPECOS-025.AC04` | Permission or budget denial blocks the candidate route before execution. |
| `BUGRAIL-SPECOS-025.AC05` | Restart preserves route order, consumed candidates and final disposition. |

## 5. Non-Goals

This Feature does not broker arbitrary cheapest-model selection or redefine
Agent identity from model identity.

