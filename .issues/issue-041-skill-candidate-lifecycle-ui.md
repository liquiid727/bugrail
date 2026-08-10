---
id: issue-041
title: "Implement the governed Skill Candidate client lifecycle"
status: draft
kind: implementation
type: frontend
priority: medium
sourceSpecId: BUGRAIL-SPECOS-010
sourceSpecVersion: "0.1"
sourceSpecHash: "56d98d38608e6058a1a622ec4b8875c0d1a68d657e306f1f4d16388eb2321b12"
requirements: [BUGRAIL-SPECOS-010.R01, BUGRAIL-SPECOS-010.R02, BUGRAIL-SPECOS-010.R03, BUGRAIL-SPECOS-010.R04, BUGRAIL-SPECOS-010.R05]
dependsOn: [issue-036, issue-039, issue-040]
---

# Implement The Governed Skill Candidate Client Lifecycle

## Outcome

Insights exposes evidence, risk, validation, approval, activation, degradation,
and rollback as separate reviewable steps rather than a one-click generator.

## Scope

- Add `Insights > Skill Candidates` filters/list/detail and source drill-in.
- Show threshold/task/run counts, version, draft as untrusted text, target Agents,
  scope, risks, conflicts, validation plan/history, approval, and active versions.
- Add validation plan review/progress dialog with durable refetch behavior.
- Add separate Approve and Activate flows; activation previews path/content/hash.
- Add explicit degraded/rollback preview and source-changed re-review behavior.
- Add safe rendering, focused hooks, pagination, i18n, and responsive layout.

## Acceptance Criteria

- Approve is available only for the exact passing version and never activates.
- Activate/Rollback require explicit preview and confirmation.
- Closing validation UI does not imply cancellation or lose progress.
- Every lifecycle/error state in Spec Section 6 is visible and source-linked.
- Long content uses safe capped scroll regions with complete keyboard/focus flow.

## Verification

Separated actions, validation progress/reconnect, stale version, activation and
rollback previews, safe text, keyboard, responsive, i18n, themes, and screenshots pass.
