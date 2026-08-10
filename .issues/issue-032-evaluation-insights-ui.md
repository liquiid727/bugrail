---
id: issue-032
title: "Implement run Evaluation and project Evaluation Insights"
status: draft
kind: implementation
type: frontend
priority: medium
sourceSpecId: BUGRAIL-SPECOS-008
sourceSpecVersion: "0.1"
sourceSpecHash: "c1ea303649af37483548d4884cf90ce5a47cc8549edd4c880cf4c41c53b9ca2d"
requirements: [BUGRAIL-SPECOS-008.R02, BUGRAIL-SPECOS-008.R03, BUGRAIL-SPECOS-008.R04, BUGRAIL-SPECOS-008.R05]
dependsOn: [issue-028, issue-031]
---

# Implement Run Evaluation And Project Evaluation Insights

## Outcome

Reviewers can inspect one run's quality and project cohorts with transparent
sample/exclusion math and no unsupported ranking or automatic action.

## Scope

- Add Run Inspector `Evaluation` with source/evidence/unknown facts.
- Add Tasks `Insights > Evaluation` with all declared filters and pagination.
- Show numerator, denominator, exclusions, sample size, cohort, and range on
  every metric; provide equivalent tables for any chart.
- Apply sample thresholds: no rate under 5, no recommendation/winner under 20.
- Add confirmed Reproject, filter persistence, drill-in/back behavior, and i18n.

## Acceptance Criteria

- Empty, loading, strict, qualified, insufficient evidence/sample, pending sync,
  source changed, invalid filter, and transport failure are covered.
- Unknown/excluded rows are inspectable and never visually folded into zero.
- Charts, if present, have semantic tables with identical values.
- Reproject remains run-scoped and cannot trigger execution/learning mutations.
- Filters survive run drill-in and responsive/narrow layouts.

## Verification

Cohort display math, thresholds/copy, filter state, pagination, Reproject,
accessibility tables, i18n, light/dark, responsive, and screenshot tests pass.
