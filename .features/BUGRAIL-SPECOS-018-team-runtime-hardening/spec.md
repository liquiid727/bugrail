---
id: BUGRAIL-SPECOS-018
version: "0.1"
title: "Team Runtime Reliability Hardening"
status: draft
changeType: work-task-team-hardening
prd: ".prd/prd-agent-team-mode-roadmap.md"
design: "design/agent-team-mode-architecture.md"
adr: "design/adr/ADR-002-agent-profile-team-worktask.md"
dependsOn: [BUGRAIL-SPECOS-015, BUGRAIL-SPECOS-016]
---

# BUGRAIL-SPECOS-018: Team Runtime Reliability Hardening

## 1. Outcome

Make the implemented static Team baseline safe under concurrent start, pause,
resume, cancel, process restart, and large run lists before adding new Team
capabilities.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-018.R01` | Team control transitions use explicit preconditions; terminal runs cannot resume or accept contradictory controls. |
| `BUGRAIL-SPECOS-018.R02` | Team control and concurrency eligibility participate in the same protected WorkTask claim path so pause/cancel cannot race a new launch. |
| `BUGRAIL-SPECOS-018.R03` | Cancel prevents new claims, records pending/partial outcomes, delegates to existing WorkTask cancellation, and cannot report final canceled state while active work has an unresolved disposition. |
| `BUGRAIL-SPECOS-018.R04` | Team start fails or remains explicitly queued when the current process does not own a usable TaskEngine; it never claims active execution it cannot drive. |
| `BUGRAIL-SPECOS-018.R05` | Restart first reconciles WorkTasks/Sessions/Worktrees and then reconstructs Team status without converting unknown work to success. |
| `BUGRAIL-SPECOS-018.R06` | Team control actions and recovery decisions produce durable audit facts and a refresh hint for Tauri/Web clients. |
| `BUGRAIL-SPECOS-018.R07` | Listing at least 100 recent Team runs uses bounded query count and does not issue one WorkTask query per node. |

## 3. Existing Modules

- Extend `team_run` control/projection and `specos_team_run_control` core.
- Deepen the WorkTask claim transaction and existing TaskEngine pump locks.
- Reuse WorkTask cancellation, recovery, events, Session and Worktree facts.
- Optimize the existing Team run projection query; do not add a Team store.

## 4. Interface

Existing `specos_team_run_start/list/control` commands remain compatible.
Control returns a typed projection including accepted state, pending work, and
recoverable failures. Any added version/precondition field is additive.

## 5. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-018.AC01` | A pause/cancel racing the scheduler cannot launch a node after the control barrier wins. |
| `BUGRAIL-SPECOS-018.AC02` | Repeated or invalid terminal controls are idempotent or return a typed precondition error without changing WorkTasks. |
| `BUGRAIL-SPECOS-018.AC03` | Cancel failure or a non-cancelable merge remains visible and cannot be projected as fully canceled. |
| `BUGRAIL-SPECOS-018.AC04` | Restart reconstructs the same Team node facts from WorkTask state and records interrupted/unknown work without false success. |
| `BUGRAIL-SPECOS-018.AC05` | Desktop and server command paths share the same transition logic and refresh behavior. |
| `BUGRAIL-SPECOS-018.AC06` | A 100-run fixture satisfies a documented bounded-query performance check. |

## 6. Non-Goals

No dynamic planning, new node kinds, permission policy, budget, fallback, or
remote notification behavior is introduced by this Feature.

