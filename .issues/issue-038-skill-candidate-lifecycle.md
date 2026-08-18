---
id: issue-038
title: "Extract and persist governed Skill candidates"
status: superseded
kind: implementation
type: backend
priority: medium
sourceSpecId: BUGRAIL-SPECOS-010
replacementSpecId: BUGRAIL-SPECOS-014
supersededBy: [issue-069, issue-070]
sourceSpecVersion: "0.1"
sourceSpecHash: "56d98d38608e6058a1a622ec4b8875c0d1a68d657e306f1f4d16388eb2321b12"
requirements: [BUGRAIL-SPECOS-010.R01, BUGRAIL-SPECOS-010.R02]
dependsOn: [issue-030, issue-034]
---

# Extract And Persist Governed Skill Candidates

## Outcome

Repeated independent evidence proposes a versioned Skill candidate with exact
sources, scope, risks, targets, conflicts, and lifecycle—never an active Skill.

## Scope

- Add candidate/version migration, entity, indexes, lifecycle, and audit events.
- Group by exact normalized pattern/scope and enforce the default threshold of
  three successful Spec-bound runs from at least two WorkTasks.
- Persist draft, target Agent IDs, source refs, evidence revision, risks,
  validation plan, candidate hash/version, conflicts, and actor/times.
- Add bounded list/get commands and typed desktop/server/TS contracts.
- Reject similarity merging, single-task thresholding, and cross-project evidence.

## Acceptance Criteria

- One run or repeated runs from one task cannot meet proposal threshold.
- Same evidence revision creates one idempotent candidate/version.
- Every source is independently inspectable and tied to an eligible evaluation.
- Proposal does not validate, approve, save, refresh, or activate a Skill.
- Invalid lifecycle/version transitions fail atomically and remain auditable.

## Verification

Threshold, grouping, independence, idempotency, project isolation, lifecycle,
pagination, restart, and transport tests pass.
