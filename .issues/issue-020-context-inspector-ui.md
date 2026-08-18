---
id: issue-020
title: "Implement the Run Context Inspector"
status: superseded
kind: implementation
type: frontend
priority: high
sourceSpecId: BUGRAIL-SPECOS-005
replacementSpecId: BUGRAIL-SPECOS-006
supersededBy: [issue-052, issue-053]
sourceSpecVersion: "0.1"
sourceSpecHash: "1491551bf38d5e1a56a932986c45604f97a5325ff41dbf05634a4000876993c2"
requirements: [BUGRAIL-SPECOS-005.R02, BUGRAIL-SPECOS-005.R03, BUGRAIL-SPECOS-005.R05]
dependsOn: [issue-008, issue-019]
---

# Implement The Run Context Inspector

## Outcome

A reviewer can inspect the exact pack, budgets, included/excluded items, hashes,
and reasons for a selected run without exposing excluded or unsafe content.

## Scope

- Add Run Inspector `Context` with pack header and budget summary.
- Add separate included/excluded filters and full attributable row metadata.
- Lazy-load/show capped preview only for included safe text.
- Show required missing/over-budget/stale blockers and corrective guidance.
- Extract focused context components/hooks and localize all ten locales.

## Acceptance Criteria

- Excluded content is never requested or present in the DOM.
- Empty optional is distinct from required-missing failure.
- Pack hash continuity is visible for resume/fallback cases.
- Loading, success, empty, blocked, stale, unavailable preview, refresh, and
  transport failure preserve the interaction contract.
- Narrow layouts use readable stacked rows; paths/hashes remain selectable/LTR.

## Verification

Network/request assertions, non-disclosure DOM tests, lazy preview, error states,
keyboard, responsive, i18n, light/dark, and screenshot verification pass.
