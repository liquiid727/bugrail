---
id: issue-024
title: "Implement the Run Impact Inspector"
status: superseded
kind: implementation
type: frontend
priority: medium
sourceSpecId: BUGRAIL-SPECOS-006
replacementSpecId: BUGRAIL-SPECOS-010
supersededBy: [issue-061, issue-062]
sourceSpecVersion: "0.1"
sourceSpecHash: "ad1ac57268e3fed82c35e8c7f3b57eda1e29fca753eb6b2ad747e328ffe223f5"
requirements: [BUGRAIL-SPECOS-006.R02, BUGRAIL-SPECOS-006.R03, BUGRAIL-SPECOS-006.R05]
dependsOn: [issue-020, issue-023]
---

# Implement The Run Impact Inspector

## Outcome

Reviewers can inspect seeds, relationships, confidence, selection, omissions,
limits, and Context outcomes through an authoritative table and bounded graph.

## Scope

- Add Run Inspector `Impact` summary and relation table grouped by seed.
- Show relation, explanation, score, exact/heuristic, and Context include/exclude.
- Add optional graph only at 100 displayed nodes or fewer.
- Provide equivalent accessible list/table and existing file-open actions.
- Distinguish recorded stale revision from corruption or unavailable analysis.

## Acceptance Criteria

- Exact/heuristic states use text badges and filters, not color alone.
- Partial/truncated views enumerate omissions and active limits.
- Above the graph threshold, table remains complete and explains graph absence.
- Loading, complete, partial, truncated, unavailable, stale, empty, and failure
  states retain last-good facts and Retry.
- Graph facts are fully keyboard/screen-reader reachable through the table.

## Verification

Table/graph parity, filters, file action, threshold, accessibility, responsive,
i18n, light/dark, and screenshot checks pass.
