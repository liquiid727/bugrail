---
id: issue-040
title: "Approve, activate, degrade, and rollback Skill versions safely"
status: superseded
kind: implementation
type: fullstack
priority: high
sourceSpecId: BUGRAIL-SPECOS-010
replacementSpecId: BUGRAIL-SPECOS-014
supersededBy: [issue-069, issue-070]
sourceSpecVersion: "0.1"
sourceSpecHash: "56d98d38608e6058a1a622ec4b8875c0d1a68d657e306f1f4d16388eb2321b12"
requirements: [BUGRAIL-SPECOS-010.R04, BUGRAIL-SPECOS-010.R05]
dependsOn: [issue-039]
---

# Approve, Activate, Degrade, And Rollback Skill Versions Safely

## Outcome

Only a human-approved passing version can use existing ACP Skill save/refresh;
activation preserves the prior version and rollback is explicit and auditable.

## Scope

- Implement version/hash-bound human approval separate from activation.
- Preview exact target Agent/path/content/hash and prior version before save.
- Reuse existing ACP Skill path conventions, save behavior, and refresh.
- Record activation, previous version, degradation, deprecation, and rollback.
- Restore chosen prior content via the same save path; never delete as rollback
  when a previous version exists.
- Add approve/activate/rollback commands with Tauri/Axum/TS parity.

## Acceptance Criteria

- Unsafe, failed, stale, unapproved, conflicting, or changed targets cannot activate.
- Validating Agent cannot approve its own candidate/version.
- Activation and rollback use CAS and preserve recoverable prior bytes/hash.
- Degradation never auto-rewrites, revalidates, rolls back, or deletes a Skill.
- Runtime registry refresh exposes exactly the recorded active version.

## Verification

Approval authorization, stale/hash races, existing save/refresh integration,
activation crash recovery, degradation, rollback, and transport tests pass.
