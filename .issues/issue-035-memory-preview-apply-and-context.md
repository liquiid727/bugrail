---
id: issue-035
title: "Preview and apply accepted project memory safely"
status: superseded
kind: implementation
type: fullstack
priority: medium
sourceSpecId: BUGRAIL-SPECOS-009
replacementSpecId: BUGRAIL-SPECOS-013
supersededBy: [issue-067, issue-068]
sourceSpecVersion: "0.1"
sourceSpecHash: "aacf54806b70b677616eae03a697cebffc70c663e04d1a2806acac882745e0bd"
requirements: [BUGRAIL-SPECOS-009.R03, BUGRAIL-SPECOS-009.R04, BUGRAIL-SPECOS-009.R05]
dependsOn: [issue-019, issue-034]
---

# Preview And Apply Accepted Project Memory Safely

## Outcome

User-reviewed actions preview an exact governed Markdown diff, apply atomically
with file-hash CAS, and make only current accepted memory available to Context.

## Scope

- Implement canonical `.specos/memory/<type>/<slug>.md` rendering.
- Implement preview for accept/reject/supersede/narrow-scope actions.
- Apply through atomic write and expected-hash CAS; reject path/symlink escape.
- Preserve concurrent human edits and Git-trackable source references/history.
- Select accepted current memory into Context with scope/status/hash/reason.
- Add Tauri/Axum parity and exact TS preview/apply client contracts.

## Acceptance Criteria

- No file-changing action succeeds without a current preview hash.
- Conflicts require keep/supersede/narrow resolution; no last-write-wins.
- `memory.fileChanged` preserves human edits and requires a new preview.
- Only accepted, non-stale, scope-matching memory enters Context.
- Reject/stale/supersede stops future injection without deleting evidence/history.

## Verification

Filesystem atomicity, CAS races, path/symlink attacks, dirty edits, rendered file
goldens, Context selection, and transport parity tests pass.
