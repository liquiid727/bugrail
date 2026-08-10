---
id: issue-019
title: "Persist and consume Context Packs in WorkTask runs"
status: draft
kind: implementation
type: fullstack
priority: high
sourceSpecId: BUGRAIL-SPECOS-005
sourceSpecVersion: "0.1"
sourceSpecHash: "1491551bf38d5e1a56a932986c45604f97a5325ff41dbf05634a4000876993c2"
requirements: [BUGRAIL-SPECOS-005.R04, BUGRAIL-SPECOS-005.R05]
dependsOn: [issue-006, issue-018]
---

# Persist And Consume Context Packs In WorkTask Runs

## Outcome

Each fresh run persists one immutable pack before prompt dispatch, reuses it
correctly for resume/fallback, and exposes safe inspection after restart.

## Scope

- Add pack migration/entity/repository keyed by `(task_id, run_seq)`.
- Compile/persist before ACP prompt and append the bounded context block before
  the existing Worktree guard.
- Preserve same-session resume, same-run fallback, and new-run recompilation rules.
- Record `context_compiled` count/hash event and bind hash to run facts.
- Add `work_task_context_get` to Tauri/Axum plus exact TS DTO/client function.

## Acceptance Criteria

- Required compile failure prevents prompt dispatch and leaves an explicit event.
- Same-session resume does not resend unchanged content.
- Pre-prompt fallback uses the exact stored pack; retry gets a new pack/hash.
- Restart returns identical pack metadata/content within security caps.
- Existing read-only/merge/retry prompt guards retain their ordering and behavior.

## Verification

Atomic persistence, prompt snapshots, resume/fallback/retry, restart, transport
parity, and existing ACP/WorkTask prompt regression tests pass.
