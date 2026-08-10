---
id: issue-036
title: "Implement Memory Candidate review and diff workflow"
status: draft
kind: implementation
type: frontend
priority: medium
sourceSpecId: BUGRAIL-SPECOS-009
sourceSpecVersion: "0.1"
sourceSpecHash: "aacf54806b70b677616eae03a697cebffc70c663e04d1a2806acac882745e0bd"
requirements: [BUGRAIL-SPECOS-009.R01, BUGRAIL-SPECOS-009.R02, BUGRAIL-SPECOS-009.R03, BUGRAIL-SPECOS-009.R04, BUGRAIL-SPECOS-009.R05]
dependsOn: [issue-032, issue-035]
---

# Implement Memory Candidate Review And Diff Workflow

## Outcome

Insights provides a complete proposal-to-file review flow with source evidence,
conflict resolution, exact diff preview, and Context eligibility visibility.

## Scope

- Add `Insights > Memory Candidates` filters/list and detail panel.
- Show type, scope, status, evidence/confidence, normalized key, source links,
  conflicts, lifecycle, proposed text, accepted path/hash, and Context use.
- Implement accept/reject/supersede/narrow actions with mandatory preview dialog.
- Render text-safe unified diff and handle file-changed by retaining user intent.
- Add safe hooks, pagination, reconnect refresh, i18n, and responsive layout.

## Acceptance Criteria

- Conflict rows never expose a generic unqualified Accept action.
- File-changing Apply always uses the previewed expected hash.
- Added/removed diff semantics use labels as well as color.
- Every lifecycle/error state in Spec Section 6 is inspectable.
- Keyboard and narrow layouts can complete all review actions.

## Verification

Filters, source drill-in, preview-before-apply, CAS conflict, safe diff DOM,
keyboard/focus, responsive, i18n, light/dark, and screenshots pass.
