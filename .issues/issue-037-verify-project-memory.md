---
id: issue-037
title: "Derive and execute verification for project memory candidates"
status: superseded
kind: verification
type: fullstack
priority: medium
sourceSpecId: BUGRAIL-SPECOS-009
replacementSpecId: BUGRAIL-SPECOS-013
supersededBy: [issue-067, issue-068]
sourceSpecVersion: "0.1"
sourceSpecHash: "aacf54806b70b677616eae03a697cebffc70c663e04d1a2806acac882745e0bd"
requirements: [BUGRAIL-SPECOS-009.R01, BUGRAIL-SPECOS-009.R02, BUGRAIL-SPECOS-009.R03, BUGRAIL-SPECOS-009.R04, BUGRAIL-SPECOS-009.R05]
dependsOn: [issue-034, issue-035, issue-036]
---

# Derive And Execute Verification For Project Memory Candidates

## Scope

- Derive exact-version Test Spec coverage for AC01–AC07.
- Verify eligible/ineligible extraction, dedup, conflict, lifecycle, path security,
  atomic CAS, concurrent edits, Context injection, transport, and review UI.
- Run Context Pack/evaluation/filesystem regressions.
- Normalize source evidence, file hashes/diffs, and visual results.

## Acceptance Criteria

- No unreviewed, rejected, stale, superseded, conflicting, or cross-project memory
  reaches Context.
- Concurrent human edits are never overwritten by stale approval.
- Source evidence and Git-trackable history survive all lifecycle transitions.
- Every AC has independent passing evidence.
