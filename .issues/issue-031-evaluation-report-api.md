---
id: issue-031
title: "Expose bounded evaluation facts and cohort reports"
status: draft
kind: implementation
type: fullstack
priority: medium
sourceSpecId: BUGRAIL-SPECOS-008
sourceSpecVersion: "0.1"
sourceSpecHash: "c1ea303649af37483548d4884cf90ce5a47cc8549edd4c880cf4c41c53b9ca2d"
requirements: [BUGRAIL-SPECOS-008.R03, BUGRAIL-SPECOS-008.R04, BUGRAIL-SPECOS-008.R05]
dependsOn: [issue-030]
---

# Expose Bounded Evaluation Facts And Cohort Reports

## Outcome

Clients can inspect one fact or aggregate bounded cohorts with honest sample,
unknown, numerator, denominator, and exclusion accounting.

## Scope

- Implement `work_task_evaluation_get`, `evaluation_report`, and idempotent
  `evaluation_reproject` command cores.
- Add Agent/model/task-kind/risk/context/route/time/evidence filters.
- Return numerator, denominator, excluded count, policy, range, and sample size.
- Enforce cursor pagination, default 30 days, maximum one-year range, minimum
  display sample 5, and recommendation sample 20.
- Add Tauri/Axum parity, exact TypeScript DTOs, and client functions.

## Acceptance Criteria

- Unknown/excluded facts never enter denominator or become zero/success.
- Invalid/unbounded cohort requests return stable typed errors.
- Reproject requires explicit action and cannot mutate adjacent domains.
- Query plans are indexed/bounded on representative fixtures.
- Desktop/server aggregation and pagination results match exactly.

## Verification

Aggregation math, cohort boundaries, pagination, query plan/performance,
authorization, transport parity, and mutation-isolation tests pass.
