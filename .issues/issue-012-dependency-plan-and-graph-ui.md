---
id: issue-012
title: "Implement dependency Plan, blockers, and Graph view"
status: superseded
kind: implementation
type: frontend
priority: high
sourceSpecId: BUGRAIL-SPECOS-003
replacementSpecId: BUGRAIL-SPECOS-004
supersededBy: [issue-048, issue-049]
sourceSpecVersion: "0.1"
sourceSpecHash: "301702c3135f0f70700ce7475c6a9b0292d41ca9d4a9eb711c82e5b37e9d801d"
requirements: [BUGRAIL-SPECOS-003.R05]
dependsOn: [issue-011, issue-008]
---

# Implement Dependency Plan, Blockers, And Graph View

## Outcome

Users can see why a task is waiting, edit allowed dependencies with CAS safety,
and inspect folder topology in Graph or an equivalent accessible list.

## Scope

- Add the Board blocker chip and Task Detail Plan readiness/dependency sections.
- Add compare-and-save dependency editor with task search, kind, ordering, and
  full proposed edge-set submission.
- Add Tasks `Board / Graph / Insights` switch and folder-scoped Graph filters.
- Provide keyboard navigation and a virtualized topology-list fallback above
  100 nodes and on narrow screens.
- Keep cycle/stale edits intact and render backend cycle/revision facts inline.

## Acceptance Criteria

- Card chips open the exact blocked Plan section.
- Ordinary drag never mutates edges; explicit Edit mode uses the same CAS dialog.
- Empty, loading, ready, waiting, terminal blocker, cycle, stale edit, over-limit,
  and transport-error states are test-covered.
- Graph/list expose equivalent labels for both dependency kinds and readiness.
- All text is localized and status never relies on color alone.

## Verification

Interaction, payload, CAS conflict, keyboard, virtualized fallback, responsive,
i18n, reconnect, light/dark, and visual screenshot checks pass.
