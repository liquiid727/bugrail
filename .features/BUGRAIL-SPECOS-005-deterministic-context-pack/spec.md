---
id: BUGRAIL-SPECOS-005
version: "0.1"
title: "Deterministic Context Pack"
status: draft
changeType: work-task-deepening
prd: ".prd/prd-specos-delivery-control.md"
design: "design/specos-control-plane-design.md"
clientDesign: "design/specos-client-interaction-design.md"
codeBaseline: "55545d43"
dependsOn: [BUGRAIL-SPECOS-001, BUGRAIL-SPECOS-002, BUGRAIL-SPECOS-004]
---

# BUGRAIL-SPECOS-005: Deterministic Context Pack

## 1. Summary

Insert a deterministic context-compilation step into existing WorkTask prompt
composition. Every included or excluded item is attributable and bounded, and
the exact pack is stored against `run_seq`. This is an internal WorkTask module
with a local filesystem test substitute, not an external Context Provider seam.

### Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-005.R01` | Before ACP prompt dispatch, a run compiles one immutable Context Pack from declared deterministic sources. |
| `BUGRAIL-SPECOS-005.R02` | Every candidate records source kind, repository-relative location, revision/hash, reason, priority, size, and include/exclude decision. |
| `BUGRAIL-SPECOS-005.R03` | Byte/item/token-estimate budgets are deterministic; required contract items cannot be silently dropped. |
| `BUGRAIL-SPECOS-005.R04` | Prompt composition consumes the stored pack and preserves retry/resume/read-only/merge guards. |
| `BUGRAIL-SPECOS-005.R05` | The user can inspect the exact pack after restart without exposing secrets or files outside the project. |

PRD coverage: `P-DC-09`, `P-DC-16`, `P-DC-18`.

## 2. Source Policy

Initial sources, in priority order:

1. bound Feature Spec metadata and selected acceptance criteria;
2. original WorkTask prompt blocks and current round instruction;
3. explicit repository-relative references captured by the task composer;
4. applicable project instructions (`AGENTS.md` from root to working path and
   paths declared by the active SpecOS manifest/rules index);
5. eligible dependency/integration handoffs;
6. retry facts: previous run outcome, unresolved items, and current Git diff
   summary.

The compiler does not scan the whole repository, perform embeddings, query the
network, or infer symbol relations. Repository expansion belongs to
`BUGRAIL-SPECOS-006`.

## 3. Interface And Storage

Internal interface:

```text
compile(ContextRequest, FileReader) -> ContextPack | ContextCompileError
```

`FileReader` is a private local-substitutable seam used by filesystem fixtures.
The WorkTask caller passes task/run/spec/dependency facts; it does not assemble
items itself.

`work_task_context_pack` has primary key `(task_id, run_seq)` and stores policy
version, source Spec hash, byte/item/token estimates, included/excluded item
JSON, pack SHA-256, and creation time. Individual file contents are stored only
when included and capped; excluded items retain metadata/reason, not content.

Default limits: 64 items, 512 KiB total content, 128 KiB per item, and a
configurable token estimate defaulting to 32,000. Required Spec/AC and user
instruction items fail compilation if they cannot fit; optional items are
excluded in stable priority/path order.

## 4. WorkTask Integration

| Existing path | Change |
|---|---|
| `work_task/engine.rs::compose_prompt` | Load/compile pack and append one bounded structured context block before the standing Worktree guard. |
| WorkTask claim/run record | Snapshot context policy version and pack hash. |
| WorkTask events | Record `context_compiled` with counts/hash only. |
| WorkTask commands/transports | Add `work_task_context_get(task_id, run_seq)`. |
| Task Detail run trace | Add included/excluded Context inspector. |

Resume in the same ACP Session does not resend unchanged content. A fresh
fallback for the same run resends the exact stored pack. A new `run_seq`
recompiles and receives a new hash.

## 5. Errors And Security

| Error key | Condition |
|---|---|
| `workTask.context.requiredMissing` | Required Spec/rule/reference cannot be read. |
| `workTask.context.outsideProject` | Canonical or symlinked path escapes root. |
| `workTask.context.overBudget` | Required items exceed a hard limit. |
| `workTask.context.staleSource` | Required source changes between read and persisted pack. |
| `workTask.context.invalidEncoding` | Required text is not supported UTF-8 text. |

- Reject `.env`, credential/key stores, Git internals, binary files, device
  files, and configured secret globs regardless of explicit mention.
- Persist SHA-256 and capped text, never absolute paths or environment values.
- Files are opened without following an unchecked path; hash/metadata are
  revalidated before the pack transaction commits.

## 6. Client Interaction Contract

This Feature implements Run Inspector `Context`. It is read-only and always
scoped to the selected `(task_id, run_seq)`.

- Header shows pack hash, policy version, source Spec hash, item/byte/token
  totals, hard limits, and compile time.
- Included and Excluded are separate filterable lists. Every row shows source
  kind, repository-relative path/location, revision/hash, priority, size,
  include/exclude decision, and stable reason.
- Selecting an included safe-text item opens a capped content preview. Excluded
  content is never fetched or rendered; only metadata and reason are visible.
- Required-over-budget/missing/stale failures show which required item blocked
  launch and which safe corrective action is available. The UI cannot override
  the compiler from the Inspector.
- Empty optional context is a successful state distinct from missing required
  context. Resumed and fallback runs visibly retain the same pack hash.

`src/lib/api.ts` exposes `workTaskContextGet`; DTOs live in
`src/lib/types.ts`. `context-tab`, `context-budget-summary`,
`context-item-list`, and `context-content-preview` live under
`src/components/tasks/specos/`. Content preview is lazy and never part of Task
Detail or board queries.

Required states are loading, included/excluded success, empty optional,
blocked required item, stale source, safe-preview unavailable, and transport
failure with last-good metadata. Paths/hashes use selectable monospace text;
lists become stacked cards on narrow screens.

## 7. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-005.AC01` | Identical repository/task facts and policy produce byte-identical ordered packs and hashes. |
| `BUGRAIL-SPECOS-005.AC02` | Required Spec/AC, prompt, rules, handoff, and retry items are included according to Section 2 with attributable reasons. |
| `BUGRAIL-SPECOS-005.AC03` | Optional over-budget items are excluded deterministically; required over-budget items block launch. |
| `BUGRAIL-SPECOS-005.AC04` | Path escape, symlink escape, secret globs, binaries, oversized files, and read/hash races cannot leak content. |
| `BUGRAIL-SPECOS-005.AC05` | Resume/fallback/retry behavior follows Section 4 and preserves all existing prompt guards. |
| `BUGRAIL-SPECOS-005.AC06` | Context Inspector covers loading, included, excluded, blocked, stale, empty optional, and transport-error states. |
| `BUGRAIL-SPECOS-005.AC07` | Tauri/Axum and post-restart reads return the same pack metadata/content. |

## 8. Testing And Implementation Order

1. Pure compiler and filesystem-fixture tests for order, budgets, hashing, and
   security exclusions.
2. Migration/repository atomicity and race tests.
3. WorkTask prompt/resume/fallback/retry integration tests.
4. Transport and Context Inspector tests including lazy preview, excluded-item
   non-disclosure, responsive layout, and every required state.
5. Existing ACP prompt, WorkTask, and desktop/server regression suites.
