---
id: issue-008
title: "Implement Task Runs and Run Inspector client surfaces"
status: draft
kind: implementation
type: frontend
priority: high
sourceSpecId: BUGRAIL-SPECOS-002
sourceSpecVersion: "0.1"
sourceSpecHash: "4d2c623f35a9caea2d66aee7f716bb8ca41c451a1c1d33b92c763e6dcda87965"
requirements: [BUGRAIL-SPECOS-002.R05]
dependsOn: [issue-007, issue-004]
---

# Implement Task Runs And Run Inspector Client Surfaces

## Outcome

A user can inspect each WorkTask generation, its durable timeline, evidence, and
unknown/legacy facts from the existing Tasks shell without reading logs.

## Scope

- Add Task Detail `Runs` with newest-first paginated summaries.
- Add the responsive `56rem` Run Inspector with Summary and Timeline tabs.
- Extract `runs-tab`, `run-list`, `run-inspector-dialog`, `run-summary`, and
  `run-timeline` under `src/components/tasks/specos/` plus focused hooks.
- Refetch only the active run after `task://changed` and after reconnect; reject
  stale responses when a dialog closes or selection changes.
- Add all ten locale message sets and keyboard/focus behavior.

## Acceptance Criteria

- First load, page load, running, settled, pending sync, legacy, not found,
  reconnect, and transport-error states match Spec Section 7.
- Selecting a row loads its trace on demand; board/detail opening does not.
- Missing facts say `Not recorded`; legacy history is never given a fake run.
- Dialog is full-screen below 768px and preserves usable timeline filters.
- Last-good content remains visible during refresh/failure with an inline Retry.

## Verification

API payload, interaction, pagination, reconnect, stale-response, keyboard,
responsive, i18n-key, light/dark, and narrow/wide visual checks pass.
