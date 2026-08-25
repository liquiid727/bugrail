---
id: issue-103
title: "CodeGraph queries, Context and Worktree isolation"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-033
sourceSpecVersion: "0.1"
sourceSpecHash: "a47f798b27680d8bf313f602abf8d4175396780832ac6d50bddf416ea846801f"
requirements: [BUGRAIL-SPECOS-033.R01, BUGRAIL-SPECOS-033.R04, BUGRAIL-SPECOS-033.R05]
dependsOn: [issue-102]
---

# CodeGraph queries, Context and Worktree isolation

## Outcome

Expose the closed read-only symbol/reference/call/impact/changed-file contract
with bounded revisioned evidence for planning, Agent tools and Context.

## Scope

Reject raw MCP/write/unbounded calls and expose stale/incomplete truth explicitly.

## Verification

Cover `BUGRAIL-SPECOS-033.T01`, `T04` and query budgets from `T05`.
