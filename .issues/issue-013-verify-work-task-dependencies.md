---
id: issue-013
title: "Derive and execute verification for WorkTask dependencies"
status: draft
kind: verification
type: fullstack
priority: high
sourceSpecId: BUGRAIL-SPECOS-003
sourceSpecVersion: "0.1"
sourceSpecHash: "301702c3135f0f70700ce7475c6a9b0292d41ca9d4a9eb711c82e5b37e9d801d"
requirements: [BUGRAIL-SPECOS-003.R01, BUGRAIL-SPECOS-003.R02, BUGRAIL-SPECOS-003.R03, BUGRAIL-SPECOS-003.R04, BUGRAIL-SPECOS-003.R05]
dependsOn: [issue-010, issue-011, issue-012]
---

# Derive And Execute Verification For WorkTask Dependencies

## Scope

- Derive the exact-version Test Spec for AC01–AC07 before evidence execution.
- Verify graph invariants, CAS races, every claim path, deletion, limits,
  desktop/server parity, UI states, and accessible graph/list equivalence.
- Run WorkTask scheduler, recovery, retry, and concurrency regressions.
- Normalize source-bound evidence under `tests/results/`.

## Acceptance Criteria

- No invalid/cyclic/stale edit changes persisted graph state.
- No manual, bulk, automatic, concurrent, or stale-client path starts an unmet
  task.
- Existing no-edge projects retain behavior and performance.
- Every AC has independent passing evidence; gaps remain blocking.
