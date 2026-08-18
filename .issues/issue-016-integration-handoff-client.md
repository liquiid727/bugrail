---
id: issue-016
title: "Implement integration source and handoff client experience"
status: superseded
kind: implementation
type: frontend
priority: high
sourceSpecId: BUGRAIL-SPECOS-004
replacementSpecId: BUGRAIL-SPECOS-005
supersededBy: [issue-050, issue-051]
sourceSpecVersion: "0.1"
sourceSpecHash: "3ac469030262845856bb116d99471d871c13c885eb72ed60e2803e1636f32afd"
requirements: [BUGRAIL-SPECOS-004.R01, BUGRAIL-SPECOS-004.R03, BUGRAIL-SPECOS-004.R04, BUGRAIL-SPECOS-004.R05]
dependsOn: [issue-012, issue-015]
---

# Implement Integration Source And Handoff Client Experience

## Outcome

Task creation, Plan, Graph, and Run Inspector expose integration type, source
eligibility, handoffs, merge order, conflicts, and containment without Git/log use.

## Scope

- Add task-kind selection/editing with legacy Implementation projection.
- Add ordered source rows, captured run/head, branch, handoff, Spec/gate state,
  and source task/session/diff actions to Plan.
- Render sanitized, attributed handoff summaries and disclosure details.
- Add explicit stale-plan compare/refresh dialog; never refresh during Start/Merge.
- Add conflict, verification failure, and landed containment presentations.
- Extend Graph with labeled integration edges and task-kind badge.

## Acceptance Criteria

- All states in Spec Section 6 are visible and actionable.
- Source facts stack into usable disclosure cards below tablet width.
- Agent-authored Markdown is sanitized and clearly attributed.
- Landed source tasks link to integration task and shared commit proof.
- Refresh/stale/transport errors retain last-good plan and user context.

## Verification

Typed API, interaction, stale refresh, sanitizer, keyboard, responsive, i18n,
Graph/list parity, light/dark, and screenshot checks pass.
