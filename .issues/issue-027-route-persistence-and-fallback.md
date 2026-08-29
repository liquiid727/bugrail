---
id: issue-027
title: "Persist routes and enforce safe pre-prompt fallback"
status: superseded
kind: implementation
type: fullstack
priority: high
sourceSpecId: BUGRAIL-SPECOS-007
replacementSpecId: BUGRAIL-SPECOS-011
supersededBy: [issue-063, issue-064]
sourceSpecVersion: "0.1"
sourceSpecHash: "13da86ab9e3289a51d07a15aad6555e80d02c1300ff0951e64929c8fb18d518b"
requirements: [BUGRAIL-SPECOS-007.R04, BUGRAIL-SPECOS-007.R05, BUGRAIL-SPECOS-007.R06]
dependsOn: [issue-006, issue-026]
---

# Persist Routes And Enforce Safe Pre-Prompt Fallback

## Outcome

Every run stores its immutable route before spawn and can follow only its
recorded pre-prompt fallback chain without changing Spec, Context, gates, or permissions.

## Scope

- Add route-decision migration/entity/repository keyed by task/run.
- Persist decision before ACP spawn and append each fallback attempt to events.
- Permit fallback only for declared pre-prompt launch/model/provider failures.
- Block permission, invalid config, Context overflow, refusal, and post-prompt fallback.
- Add preview/get commands, Tauri/Axum parity, TS DTOs, and typed API functions.

## Acceptance Criteria

- Decision survives restart and historical runs are never rewritten.
- Preview is advisory; changed candidate facts reject spawn with a typed error.
- No fallback changes bound Spec/hash, Context Pack/hash, Worktree, gate policy,
  or permission policy.
- After message/prompt ID creation, failure requires a new run.
- Wire/storage omit provider keys, environment, and raw diagnostics.

## Verification

Pre/post-prompt failure injection, decision races, restart, invariant snapshots,
transport parity, and ACP/provider regression tests pass.
