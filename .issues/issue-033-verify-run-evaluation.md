---
id: issue-033
title: "Derive and execute verification for run evaluation projection"
status: draft
kind: verification
type: fullstack
priority: medium
sourceSpecId: BUGRAIL-SPECOS-008
sourceSpecVersion: "0.1"
sourceSpecHash: "c1ea303649af37483548d4884cf90ce5a47cc8549edd4c880cf4c41c53b9ca2d"
requirements: [BUGRAIL-SPECOS-008.R01, BUGRAIL-SPECOS-008.R02, BUGRAIL-SPECOS-008.R03, BUGRAIL-SPECOS-008.R04, BUGRAIL-SPECOS-008.R05]
dependsOn: [issue-030, issue-031, issue-032]
---

# Derive And Execute Verification For Run Evaluation Projection

## Scope

- Derive exact-hash Test Spec scenarios for AC01–AC07.
- Verify taxonomy, idempotency, source revisions, unknowns, exclusions, cohort
  math, thresholds, query bounds, privacy, transport parity, and every UI state.
- Run run/gate/route/token/review regression suites.
- Capture normalized facts, queries, screenshots, and source-bound results.

## Acceptance Criteria

- Independent fixtures reproduce exact expected fact/report values after restart.
- Sparse/qualified data never produces an unsupported recommendation.
- Evaluation surfaces and commands cannot mutate delivery or learning state.
- Every AC has independent passing evidence.
