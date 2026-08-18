---
id: issue-028
title: "Implement route preview and persisted Route Inspector"
status: superseded
kind: implementation
type: frontend
priority: high
sourceSpecId: BUGRAIL-SPECOS-007
replacementSpecId: BUGRAIL-SPECOS-011
supersededBy: [issue-063, issue-064]
sourceSpecVersion: "0.1"
sourceSpecHash: "13da86ab9e3289a51d07a15aad6555e80d02c1300ff0951e64929c8fb18d518b"
requirements: [BUGRAIL-SPECOS-007.R01, BUGRAIL-SPECOS-007.R02, BUGRAIL-SPECOS-007.R03, BUGRAIL-SPECOS-007.R04, BUGRAIL-SPECOS-007.R05]
dependsOn: [issue-008, issue-027]
---

# Implement Route Preview And Persisted Route Inspector

## Outcome

Users can understand the proposed Agent/model choice before Start and the exact
persisted choice/fallbacks afterward, including every candidate reason.

## Scope

- Add explicit override/Automatic summary and Preview route action.
- Add candidate table with score, qualifications, disqualifications, reason codes,
  chosen route, policy/hash, and ordered fallbacks.
- Repeat advisory summary in Start confirmation and handle changed-before-spawn
  by retaining the dialog and offering a new preview.
- Add immutable Run Inspector `Route` and fallback-attempt history.
- Add narrow stacked-card and screen-reader semantics.

## Acceptance Criteria

- Advisory preview and persisted decision are visibly distinct.
- No-candidate/explicit-unavailable states are actionable but secret-free.
- Loading, explicit, automatic, fallback-used, invalid policy, stale catalog,
  changed, no-candidate, and transport failure are covered.
- Candidate score/reason semantics survive narrow layout and screen readers.
- UI cannot silently start a newly resolved route after stale preview.

## Verification

Payload, stale preview, Start dialog, fallback history, keyboard, accessibility,
responsive, i18n, light/dark, and screenshot tests pass.
