---
id: BUGRAIL-SPECOS-021
version: "0.1"
title: "Team Quality And Finalization"
status: draft
changeType: team-quality
prd: ".prd/prd-agent-team-mode-roadmap.md"
design: "design/agent-team-mode-architecture.md"
adr: "design/adr/ADR-001-embed-specos-in-work-task.md"
dependsOn: [BUGRAIL-SPECOS-001, BUGRAIL-SPECOS-003, BUGRAIL-SPECOS-005, BUGRAIL-SPECOS-019]
---

# BUGRAIL-SPECOS-021: Team Quality And Finalization

## 1. Outcome

Aggregate existing WorkTask contracts, gates, Git evidence, reviewer work and
human decisions into an explainable Team completion decision and final report.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-021.R01` | Reviewer execution is an ordinary WorkTask using a reviewer Agent Profile and an immutable run snapshot. |
| `BUGRAIL-SPECOS-021.R02` | Test, lint, build, command and human-approval requirements reuse existing WorkTask gate semantics unless a later Feature proves a new executor is necessary. |
| `BUGRAIL-SPECOS-021.R03` | Team completion is derived from required WorkTask terminal/gate/Git facts and cannot be set by Agent output or a client-only transition. |
| `BUGRAIL-SPECOS-021.R04` | Reviewer output maps every required acceptance ID to pass, fail or unknown with evidence references; deterministic failed gates cannot be overridden by review. |
| `BUGRAIL-SPECOS-021.R05` | Finalization records an immutable report referencing goal, Workflow/plan, nodes, Agent resolutions, changes, gates, review, approvals, risks and commits. |
| `BUGRAIL-SPECOS-021.R06` | The report references existing evidence and does not duplicate full transcripts, diffs or gate rows. |

## 3. Existing Modules

- Reuse WorkTask contracts, gate decisions, run projections and durable events.
- Reuse WorkTask diff/changed-file/Git integration and handoff facts.
- Extend Team projection and Teams/Task Detail with an acceptance summary.

## 4. Interface

Add Team acceptance-decision and final-report read operations. Any finalize
operation uses a state/hash precondition and returns unmet requirements rather
than forcing a status.

## 5. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-021.AC01` | A failed required WorkTask gate or missing required evidence prevents Team completion. |
| `BUGRAIL-SPECOS-021.AC02` | Reviewer omission of a required acceptance ID makes the review invalid, not implicitly passing. |
| `BUGRAIL-SPECOS-021.AC03` | A successful report can be reconstructed from persisted references after restart. |
| `BUGRAIL-SPECOS-021.AC04` | Reconnect cannot regress or fabricate the acceptance decision. |
| `BUGRAIL-SPECOS-021.AC05` | Desktop/server and accessible Teams/Task Detail views expose equivalent decision facts. |

## 6. Non-Goals

This Feature does not introduce a generic artifact database, custom gate plugin
registry, permission engine, budget, or provider fallback.

