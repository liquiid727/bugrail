---
id: issue-011
title: "Enforce explainable readiness in every WorkTask claim path"
status: superseded
kind: implementation
type: fullstack
priority: high
sourceSpecId: BUGRAIL-SPECOS-003
replacementSpecId: BUGRAIL-SPECOS-004
supersededBy: [issue-048, issue-049]
sourceSpecVersion: "0.1"
sourceSpecHash: "301702c3135f0f70700ce7475c6a9b0292d41ca9d4a9eb711c82e5b37e9d801d"
requirements: [BUGRAIL-SPECOS-003.R03, BUGRAIL-SPECOS-003.R04]
dependsOn: [issue-010]
---

# Enforce Explainable Readiness In Every WorkTask Claim Path

## Outcome

Manual start, Start all, auto-process, and concurrent claims consume one backend
readiness decision and cannot launch an unmet task.

## Scope

- Implement readiness projection with revision, unmet, and terminal blockers.
- Recheck readiness in the same transaction as every existing claim CAS.
- Add set/get-graph/get-readiness command cores, Tauri/Axum parity, typed errors,
  TypeScript DTOs, and `src/lib/api.ts` methods.
- Preserve status, concurrency, retry, recovery, and no-edge behavior.

## Acceptance Criteria

- `completion` becomes satisfied only when the prerequisite is `done`.
- `integration_source` remains explicitly unsupported until Feature 004 exists.
- Failed/canceled prerequisites report terminal blockers without mutating the
  dependent task.
- Concurrent/stale UI requests cannot race past the transactional check.
- Desktop and server return the same revision and reason payloads.

## Verification

Manual, bulk, automatic, concurrent, retry, recovery, and legacy scheduling
tests pass with stable structured reason assertions.
