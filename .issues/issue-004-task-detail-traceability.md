---
id: issue-004
title: "Show Spec and gate traceability in Task Detail"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-001
sourceSpecVersion: "0.3"
sourceSpecHash: "79160488f65ae762decaa6db4987c15a783f61c886588c1e9157fc1bb40ab0d0"
requirements: [BUGRAIL-SPECOS-001.R08]
dependsOn: [issue-002, issue-003]
---

# Show Spec And Gate Traceability In Task Detail

## Scope

- Add typed preview/contract/gate APIs and stale-response-safe hooks.
- Add the Board Spec/delivery chips and Task Detail `Contract` tab.
- Implement two-step Preview -> AC/gate selection -> Bind and explicit Rebind
  compare flow using the preview hash.
- Add gate attempt history, authoritative decision reasons, human approval,
  policy-allowed waiver with required reason, and merge/complete summaries.
- Extract focused SpecOS components instead of growing one monolithic Sheet.
- Cover every state and interaction in Feature Spec Section 7, including
  reconnect, duplicate-submit prevention, keyboard/focus, narrow layout, and
  all ten locale files.
- Add i18n messages for all supported locales using current conventions.

## Existing Modules

- `src/components/tasks/task-detail-sheet.tsx`
- `src/components/tasks/specos/`
- `src/hooks/specos/`
- `src/components/tasks/task-merge-dialog.tsx`
- `src/components/tasks/task-complete-dialog.tsx`
- `src/components/tasks/task-card.tsx`
- `src/i18n/messages/`

## Acceptance

- Feature Test cases `T13`, `T15`, and `T29-T31` pass.
- Manual evidence contains light/dark and narrow/wide screenshots of Board,
  Contract tab, bind/rebind, gate decision, stale, and failure states.
- Backend enforcement remains effective if the UI is stale or bypassed.
