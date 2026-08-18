---
id: issue-030
title: "Project idempotent evidence-qualified run evaluations"
status: superseded
kind: implementation
type: backend
priority: medium
sourceSpecId: BUGRAIL-SPECOS-008
replacementSpecId: BUGRAIL-SPECOS-012
supersededBy: [issue-065, issue-066]
sourceSpecVersion: "0.1"
sourceSpecHash: "c1ea303649af37483548d4884cf90ce5a47cc8549edd4c880cf4c41c53b9ca2d"
requirements: [BUGRAIL-SPECOS-008.R01, BUGRAIL-SPECOS-008.R02, BUGRAIL-SPECOS-008.R04, BUGRAIL-SPECOS-008.R05]
dependsOn: [issue-007, issue-027]
---

# Project Idempotent Evidence-Qualified Run Evaluations

## Outcome

Each eligible settled run has one version-bound evaluation fact with explicit
unknown, exclusion, evidence-quality, outcome, and failure semantics.

## Scope

- Add evaluation migration/entity/indexes keyed by `(task_id, run_seq)`.
- Implement the deep `project(run_trace, evidence_policy)` normalizer.
- Persist source revision, Spec/route/context IDs, outcome, first-pass, gate,
  review, rework, intervention, duration, token, diff, and failure facts.
- Enforce strict/qualified/insufficient rules and the stable failure taxonomy.
- Rerun idempotently after token/review sync only on source-revision change.

## Acceptance Criteria

- Reprojection/restart cannot duplicate a run fact.
- Missing data remains null/unknown; waiver, stale Spec, legacy, cancellation,
  infrastructure, and requirement failures remain semantically distinct.
- Raw prompt/transcript/file content/provider keys/personal identifiers are absent.
- Projection never mutates task, gate, route, memory, or Skill state.
- Source changes during projection fail/retry without mixed revisions.

## Verification

Golden taxonomy/evidence fixtures, revision races, idempotency, sync updates,
privacy assertions, migration, and restart tests pass.
