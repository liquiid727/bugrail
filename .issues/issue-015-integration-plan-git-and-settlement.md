---
id: issue-015
title: "Execute and settle integration WorkTasks from eligible sources"
status: draft
kind: implementation
type: fullstack
priority: high
sourceSpecId: BUGRAIL-SPECOS-004
sourceSpecVersion: "0.1"
sourceSpecHash: "3ac469030262845856bb116d99471d871c13c885eb72ed60e2803e1636f32afd"
requirements: [BUGRAIL-SPECOS-004.R03, BUGRAIL-SPECOS-004.R04, BUGRAIL-SPECOS-004.R05]
dependsOn: [issue-003, issue-007, issue-011, issue-014]
---

# Execute And Settle Integration WorkTasks From Eligible Sources

## Outcome

An integration WorkTask snapshots eligible source runs/heads, merges them in its
own Worktree, lands through existing gates, and settles sources only after proof.

## Scope

- Materialize ordered integration sources from `integration_source` edges.
- Implement eligibility, snapshot revision, stale-plan, and explicit refresh.
- Compose structured integration instructions and reuse current ACP/Worktree flow.
- Add safe Git helpers for ordered source merge, conflict facts, and containment.
- Atomically settle integration and source tasks after landing containment.
- Expose handoff get, integration plan, and refresh through Tauri/Axum/TS APIs.

## Acceptance Criteria

- Missing Spec/gate/handoff/branch/run facts identify the exact ineligible source.
- Source retry/rebind/head change blocks stale launch and merge until refresh.
- Conflicts remain inspectable/retryable in the integration Worktree.
- Cancel/failure leaves sources in review and preserves required Worktrees.
- Crash recovery cannot double-settle or land without all captured heads contained.

## Verification

Git fixture, conflict, stale snapshot, containment, coordinated transaction,
crash recovery, transport parity, and existing merge-recovery tests pass.
