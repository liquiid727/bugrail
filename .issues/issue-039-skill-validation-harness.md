---
id: issue-039
title: "Build isolated baseline-comparing Skill candidate validation"
status: superseded
kind: implementation
type: backend
priority: high
sourceSpecId: BUGRAIL-SPECOS-010
replacementSpecId: BUGRAIL-SPECOS-014
supersededBy: [issue-069, issue-070]
sourceSpecVersion: "0.1"
sourceSpecHash: "56d98d38608e6058a1a622ec4b8875c0d1a68d657e306f1f4d16388eb2321b12"
requirements: [BUGRAIL-SPECOS-010.R02, BUGRAIL-SPECOS-010.R03]
dependsOn: [issue-038]
---

# Build Isolated Baseline-Comparing Skill Candidate Validation

## Outcome

A fixed candidate version can be validated in a disposable Worktree/fixture
against a recorded baseline with bounded, append-only, independently reviewable evidence.

## Scope

- Add append-only validation storage bound to candidate version/hash.
- Validate draft content for secrets, absolute paths, credential files, permission
  or gate bypass instructions before execution.
- Run declared fixtures/replay inputs, commands, criteria, time/resources, and
  permission policy in a disposable Worktree.
- Deny network by default; require plan declaration and user approval to allow.
- Compare baseline/candidate outcomes and prevent validator self-approval.
- Expose `skill_candidate_validate` with durable progress/result facts.

## Acceptance Criteria

- Stale candidate/evidence/plan cannot start or complete validation.
- Passing requires every criterion and no baseline regression.
- No validation path pushes/releases, widens permissions, or escapes its Worktree.
- Close/disconnect does not lose durable validation state.
- Failed/unsafe validation cannot advance to approved/active.

## Verification

Sandbox/path/network/secret/timeout attacks, baseline regressions, cancellation,
crash/restart, stale-version, self-approval, and durable progress tests pass.
