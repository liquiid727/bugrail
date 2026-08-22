---
id: BUGRAIL-SPECOS-023
version: "0.1"
title: "Team Effective Permission Policy"
status: draft
changeType: team-permission
prd: ".prd/prd-agent-team-mode-roadmap.md"
design: "design/agent-team-mode-architecture.md"
adr: "design/adr/ADR-002-agent-profile-team-worktask.md"
dependsOn: [BUGRAIL-SPECOS-002, BUGRAIL-SPECOS-018]
---

# BUGRAIL-SPECOS-023: Team Effective Permission Policy

## 1. Outcome

Enforce a least-privilege permission decision for every Team WorkTask action at
the existing ACP/tool/command seam and record any human escalation.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-023.R01` | Effective permission is the intersection of runtime capability, existing BugRail policy, project policy, Agent Profile policy, Workflow/node restriction and bounded human escalation. |
| `BUGRAIL-SPECOS-023.R02` | Enforcement occurs in backend execution paths; UI disablement and prompt text are not security controls. |
| `BUGRAIL-SPECOS-023.R03` | Filesystem roots, shell classes, network, MCP tools, secret references, commit, merge and destructive operations have explicit default-deny or inherited semantics. |
| `BUGRAIL-SPECOS-023.R04` | Escalation binds an actor, reason, capability, scope, expiry and exact task/run; it cannot become a silent permanent project grant. |
| `BUGRAIL-SPECOS-023.R05` | Allowed, denied and escalated decisions are attributable without persisting secret values or protected context. |
| `BUGRAIL-SPECOS-023.R06` | Legacy non-Team WorkTasks preserve current permission behavior unless explicitly migrated. |

## 3. Existing Modules

- Deepen existing ACP permission requests and command authorization.
- Resolve Agent Profile identity from existing immutable WorkTask run facts.
- Add only the policy configuration and audit facts required for enforcement.

The project policy file/location and migration contract must be finalized before
this Feature can move from draft to approved.

## 4. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-023.AC01` | A read-only reviewer cannot write, commit or merge even when its prompt requests it. |
| `BUGRAIL-SPECOS-023.AC02` | A destructive action is denied or requests the exact configured escalation before execution. |
| `BUGRAIL-SPECOS-023.AC03` | An expired or differently scoped escalation cannot authorize another action/run. |
| `BUGRAIL-SPECOS-023.AC04` | Project/Worktree path escape and unauthorized knowledge/secret access fail closed. |
| `BUGRAIL-SPECOS-023.AC05` | Tauri and server paths produce the same backend permission outcome and audit facts. |

## 5. Non-Goals

This Feature is not a generic policy plugin framework and does not define cost
budgeting or provider fallback.

