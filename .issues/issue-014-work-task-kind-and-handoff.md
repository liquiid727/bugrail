---
id: issue-014
title: "Add WorkTask kinds and trusted structured handoffs"
status: draft
kind: implementation
type: backend
priority: high
sourceSpecId: BUGRAIL-SPECOS-004
sourceSpecVersion: "0.1"
sourceSpecHash: "3ac469030262845856bb116d99471d871c13c885eb72ed60e2803e1636f32afd"
requirements: [BUGRAIL-SPECOS-004.R01, BUGRAIL-SPECOS-004.R02]
dependsOn: [issue-006, issue-010]
---

# Add WorkTask Kinds And Trusted Structured Handoffs

## Outcome

WorkTasks are explicitly implementation or integration tasks, and a correlated
run can publish one bounded, versioned handoff without opening a generic write path.

## Scope

- Add `task_kind` defaulting existing rows to `implementation`.
- Add handoff migration/entity/DTO with source branch/head, paths, decisions,
  risks, verification, unresolved items, summary, schema version, and actor.
- Extend the existing correlated `task_complete` reporting path compatibly.
- Permit separately attributed human revisions without rewriting Agent history.
- Enforce all item/size/path limits from Spec Section 3.

## Acceptance Criteria

- Old rows and old `task_complete` payloads remain valid.
- Only the live `(connection_id, run_seq)` path can create an Agent handoff.
- Handoff branch/head matches persisted Git/run facts at write time.
- Oversized, absolute, escaping, or malformed handoffs are rejected atomically.
- Agent and human revisions remain distinguishable and auditable after restart.

## Verification

Migration, backward compatibility, correlation, bounds, attribution, rollback,
and malicious payload tests pass.
