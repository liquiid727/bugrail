---
id: issue-017
title: "Derive and execute verification for integration WorkTasks"
status: draft
kind: verification
type: fullstack
priority: high
sourceSpecId: BUGRAIL-SPECOS-004
sourceSpecVersion: "0.1"
sourceSpecHash: "3ac469030262845856bb116d99471d871c13c885eb72ed60e2803e1636f32afd"
requirements: [BUGRAIL-SPECOS-004.R01, BUGRAIL-SPECOS-004.R02, BUGRAIL-SPECOS-004.R03, BUGRAIL-SPECOS-004.R04, BUGRAIL-SPECOS-004.R05]
dependsOn: [issue-014, issue-015, issue-016]
---

# Derive And Execute Verification For Integration WorkTasks

## Scope

- Derive an exact-hash Test Spec for AC01–AC08.
- Cover old payloads, handoff trust, source eligibility/staleness, deterministic
  merges, conflicts, cancellation, containment, coordinated settlement, and UI.
- Inject crashes around Git landing and database settlement.
- Run WorkTask gates, Git recovery, dependency, desktop, and server regressions.

## Acceptance Criteria

- No Agent verdict, stale plan, client mutation, or direct completion bypass lands
  or settles a source.
- Every successful settlement has Git containment evidence for all source heads.
- Failure/cancel/crash paths preserve recoverable source state.
- Every AC has independent source-bound evidence under `tests/results/`.
