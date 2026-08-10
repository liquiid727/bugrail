---
id: issue-042
title: "Security-review and verify controlled Skill candidates"
status: draft
kind: verification
type: fullstack
priority: high
sourceSpecId: BUGRAIL-SPECOS-010
sourceSpecVersion: "0.1"
sourceSpecHash: "56d98d38608e6058a1a622ec4b8875c0d1a68d657e306f1f4d16388eb2321b12"
requirements: [BUGRAIL-SPECOS-010.R01, BUGRAIL-SPECOS-010.R02, BUGRAIL-SPECOS-010.R03, BUGRAIL-SPECOS-010.R04, BUGRAIL-SPECOS-010.R05]
dependsOn: [issue-038, issue-039, issue-040, issue-041]
---

# Security-Review And Verify Controlled Skill Candidates

## Scope

- Independently derive exact-version Test Spec coverage for AC01–AC08.
- Attack thresholds, source eligibility, unsafe draft content, paths, secrets,
  permissions, network, sandbox escape, self-approval, version/hash races,
  activation, degradation, and rollback.
- Verify existing ACP Skill compatibility plus all lifecycle client states.
- Normalize raw security outputs, baselines, hashes, actors, and visual evidence.

## Acceptance Criteria

- No single-task/unsafe/failing/stale/unapproved/self-approved path activates.
- Validation cannot push, release, widen permissions, leak secrets, or escape.
- Every activation/rollback proves exact content/hash and recoverable prior version.
- Degradation has no silent content or policy side effect.
- Every AC has independent passing evidence; any bypass remains release-blocking.
