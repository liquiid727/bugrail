---
id: BUGRAIL-SPECOS-013
version: "0.1"
title: "Project Memory Candidates"
status: draft
changeType: governed-learning
prd: ".prd/prd-specos-agent-team-context-system.md"
design: "design/specos-control-plane-design.md"
clientDesign: "design/specos-client-interaction-design.md"
codeBaseline: "55545d43"
dependsOn: [BUGRAIL-SPECOS-006, BUGRAIL-SPECOS-012]
---

# BUGRAIL-SPECOS-013: Project Memory Candidates

## 1. Summary

Extract reviewable project-memory candidates from strict/qualified run evidence
and explicit user corrections. Candidates remain runtime proposals in SQLite;
accepted memory is written as project-local, Git-trackable Markdown and can be
selected by the existing Context Pack policy. Nothing is silently injected or
shared across projects.

### Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-013.R01` | Extraction creates typed candidates with exact source run/evidence references, confidence inputs, scope, and proposed text. |
| `BUGRAIL-SPECOS-013.R02` | Candidates are deduplicated/conflict-checked and move through explicit proposed, accepted, rejected, stale, and superseded states. |
| `BUGRAIL-SPECOS-013.R03` | Accepting a candidate writes or updates one canonical project Markdown memory file through a previewed user action. |
| `BUGRAIL-SPECOS-013.R04` | Context Pack may include accepted, non-stale memory only when scope matches and records the selection reason/hash. |
| `BUGRAIL-SPECOS-013.R05` | Rejection/staleness stops future injection without deleting source evidence or Git history. |

PRD coverage: `P-DC-14`, `P-DC-09`, `P-DC-16`, `P-DC-18`.

## 2. Memory Types And Sources

Initial types: `fact`, `rule`, `decision`, `preference`, `failure_lesson`, and
`pattern_candidate`. Skill candidates are excluded and owned by
`BUGRAIL-SPECOS-014`.

Allowed sources:

- strict or qualified evaluation facts with bound run evidence;
- explicit human review/waiver/correction events;
- accepted handoff decisions and unresolved items;
- repeated failures with the same normalized category and scope.

Agent final text alone is never an eligible source.

## 3. Storage And File Contract

`project_memory_candidate` stores candidate UUID, folder/project ID, type,
scope paths/modules, normalized key, proposed text, source references, evidence
count, confidence inputs, conflicts, status, actor/timestamps, and accepted file
hash/path when applicable.

Accepted files live under `.specos/memory/<type>/<slug>.md` in the active
project and contain stable ID, status, scope, source run refs, reviewed text,
created/updated time, and supersedes relation. There is no generated YAML twin.
Maximum candidate/file text is 32 KiB; maximum 100 source refs.

## 4. Lifecycle And Rules

```text
proposed -> accepted | rejected
accepted -> stale | superseded
stale -> accepted (new reviewed revision) | superseded
```

1. Extraction is deterministic for the same evidence revision and idempotent by
   `(project, type, normalized_key, source_revision)`.
2. Exact duplicates merge source refs; semantic guessing is not used for
   deduplication in the first implementation.
3. Conflicting accepted memory blocks acceptance until the user chooses keep,
   supersede, or narrow scope. No silent last-write-wins.
4. Acceptance shows the exact target path/diff and uses atomic write plus
   compare-and-swap against the previously read file hash.
5. Files outside `.specos/memory/`, symlink escapes, and dirty concurrent edits
   are rejected.
6. Context selection uses type/scope/status/hash and the same security/budget
   rules as other Context items.
7. Cross-project/global promotion is outside this Feature.

## 5. Commands, Errors, And UI

```text
memory_candidate_extract(task_id, run_seq) -> Candidate[]
memory_candidate_list(folder_id, filters, cursor) -> CandidatePage
memory_candidate_get(id) -> CandidateDetail
memory_candidate_preview(id, action) -> FileDiff
memory_candidate_apply(id, action, expected_hash) -> Candidate
```

| Error key | Condition |
|---|---|
| `memory.sourceIneligible` | Evidence quality/source is insufficient. |
| `memory.conflict` | Accepted memory conflicts and needs a user resolution. |
| `memory.fileChanged` | Target changed after preview. |
| `memory.pathInvalid` | Target escapes governed memory path. |

UI covers empty, loading, proposed, duplicate, conflict, preview, accepted,
rejected, stale, superseded, file-changed, and transport-error states.

## 6. Client Interaction Contract

This Feature implements Tasks `Insights > Memory Candidates`.

- The list is project/folder-scoped and filterable by type, lifecycle, scope,
  conflict, evidence quality, and source run. Each row shows proposed title,
  type, scope, status, evidence count, confidence inputs, conflict state, and
  updated time.
- Selecting a row opens a detail panel with proposed text, exact source refs,
  source evidence links, normalized key, conflicts, lifecycle history, and the
  current accepted file/path/hash when present.
- Accept, supersede, narrow-scope, and reject are explicit actions. Any
  file-changing action first calls `memory_candidate_preview` and shows target
  path plus unified diff; Apply submits the previewed expected hash.
- A conflict cannot expose a generic Accept button. The user chooses keep
  existing, supersede, or edit/narrow scope, then previews the resulting diff.
- `memory.fileChanged` keeps the proposed action, displays the concurrent-change
  explanation, and requires a new preview. The client never overwrites.
- Accepted/stale/superseded entries show whether they are currently eligible
  for Context Pack and link to the recorded selection reason when used.

`src/lib/api.ts` exposes extract/list/get/preview/apply methods mirroring the
commands; DTOs live in `src/lib/types.ts`. `memory-candidate-list`,
`memory-candidate-detail`, `memory-diff-dialog`, and
`memory-conflict-resolution` live under `src/components/tasks/specos/`.

Required states are empty, loading, proposed, duplicate, conflict, preview,
accepted, rejected, stale, superseded, file changed, invalid path, and
transport failure. Diff rendering is text-safe, keyboard scrollable, and uses
added/removed labels in addition to color.

## 7. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-013.AC01` | Eligible evidence produces deterministic, idempotent, typed candidates with exact sources. |
| `BUGRAIL-SPECOS-013.AC02` | Agent text, insufficient evidence, cross-project sources, and unbound runs cannot create an acceptable candidate. |
| `BUGRAIL-SPECOS-013.AC03` | Duplicate/conflicting candidates follow Section 4 without losing source provenance. |
| `BUGRAIL-SPECOS-013.AC04` | Preview/apply uses governed paths, atomic CAS writes, and preserves concurrent human edits. |
| `BUGRAIL-SPECOS-013.AC05` | Only accepted current memory can enter Context Pack, with file hash and reason visible. |
| `BUGRAIL-SPECOS-013.AC06` | Rejected/stale/superseded memory is not injected and remains auditable. |
| `BUGRAIL-SPECOS-013.AC07` | Desktop/server and all lifecycle UI states are equivalent. |

## 8. Testing And Implementation Order

1. Pure extraction/key/scope/conflict/lifecycle tests.
2. Candidate migration/repository/idempotency tests.
3. Filesystem fixture tests for preview, atomic CAS, paths, symlinks, and dirty edits.
4. Context Pack selection tests.
5. Transport and candidate-review UI tests covering filters, detail sources,
   preview-before-apply, CAS conflict, safe diff rendering, and every state.
