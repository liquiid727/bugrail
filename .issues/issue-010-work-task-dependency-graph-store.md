---
id: issue-010
title: "Add transactional WorkTask dependency graph storage"
status: superseded
kind: implementation
type: backend
priority: high
sourceSpecId: BUGRAIL-SPECOS-003
replacementSpecId: BUGRAIL-SPECOS-004
supersededBy: [issue-048, issue-049]
sourceSpecVersion: "0.1"
sourceSpecHash: "301702c3135f0f70700ce7475c6a9b0292d41ca9d4a9eb711c82e5b37e9d801d"
requirements: [BUGRAIL-SPECOS-003.R01, BUGRAIL-SPECOS-003.R02]
dependsOn: [issue-001]
---

# Add Transactional WorkTask Dependency Graph Storage

## Outcome

WorkTasks can persist ordered, same-project dependency edges with atomic cycle,
revision, deletion, and size-limit enforcement.

## Scope

- Add migration/entity/indexes for `work_task_dependency`.
- Implement full-edge-set replacement using `expected_revision` CAS.
- Validate self, duplicate, cross-project, unsupported-kind, cycle, and limits.
- Record one `dependencies_changed` event per successful replacement.
- Reject prerequisite deletion while live dependents exist.

## Acceptance Criteria

- Invalid or stale edits leave the previous graph and event stream unchanged.
- Recursive cycle checks return a stable cycle path within 500 tasks/2,000 edges.
- Worktree child folders resolve to their root project for edge validation.
- Edges are immutable while a task is queued or later per Spec Section 4.
- Migration up/down and FK behavior preserve existing WorkTasks.

## Verification

Repository, recursive CTE, transaction rollback, CAS race, delete protection,
limit, and restart tests pass.
