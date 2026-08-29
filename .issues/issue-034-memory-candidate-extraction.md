---
id: issue-034
title: "Extract and govern project memory candidates"
status: superseded
kind: implementation
type: backend
priority: medium
sourceSpecId: BUGRAIL-SPECOS-009
replacementSpecId: BUGRAIL-SPECOS-013
supersededBy: [issue-067, issue-068]
sourceSpecVersion: "0.1"
sourceSpecHash: "aacf54806b70b677616eae03a697cebffc70c663e04d1a2806acac882745e0bd"
requirements: [BUGRAIL-SPECOS-009.R01, BUGRAIL-SPECOS-009.R02, BUGRAIL-SPECOS-009.R05]
dependsOn: [issue-030]
---

# Extract And Govern Project Memory Candidates

## Outcome

Eligible evidence creates deterministic, typed, project-scoped proposals with
exact provenance, conflicts, confidence inputs, and an auditable lifecycle.

## Scope

- Add candidate migration/entity/indexes and lifecycle transitions.
- Extract supported types only from eligible evaluation/human/handoff/repeated
  failure facts; exclude Agent final text alone.
- Implement exact normalized-key deduplication and source-reference merging.
- Detect conflicts with accepted memory and require explicit resolution.
- Add extract/list/get commands with bounded pagination and filters.

## Acceptance Criteria

- Same evidence revision is idempotent by project/type/key/source revision.
- Cross-project, unbound, insufficient, or Agent-text-only sources cannot become
  acceptable candidates.
- Exact duplicates merge provenance without semantic guessing.
- Rejected/stale/superseded records stay auditable and ineligible for injection.
- Status transitions outside the declared lifecycle fail atomically.

## Verification

Extraction fixtures, lifecycle table tests, dedup/conflict, project isolation,
pagination, restart, and invalid-transition tests pass.
