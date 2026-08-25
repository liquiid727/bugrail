---
id: issue-102
title: "CodeGraph index lifecycle and provider jobs"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-033
sourceSpecVersion: "0.1"
sourceSpecHash: "a47f798b27680d8bf313f602abf8d4175396780832ac6d50bddf416ea846801f"
requirements: [BUGRAIL-SPECOS-033.R02, BUGRAIL-SPECOS-033.R03]
dependsOn: [issue-085]
---

# CodeGraph index lifecycle and provider jobs

## Outcome

Complete base/Worktree index scope, full/incremental publication, coalescing and
restart recovery in existing `code_intelligence` using provider jobs.

## Scope

Keep the pinned managed `codebase-memory-mcp` Adapter and existing Worktree
lifecycle; add no second graph store.

## Verification

Cover `BUGRAIL-SPECOS-033.T02-T03` and process reuse foundations from `T05`.
