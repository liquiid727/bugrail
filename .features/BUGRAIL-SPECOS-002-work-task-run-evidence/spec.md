---
id: BUGRAIL-SPECOS-002
version: "0.1"
title: "WorkTask Run Evidence"
status: draft
changeType: work-task-deepening
prd: ".prd/prd-specos-delivery-control.md"
design: "design/specos-control-plane-design.md"
clientDesign: "design/specos-client-interaction-design.md"
codeBaseline: "55545d43"
dependsOn: [BUGRAIL-SPECOS-001]
---

# BUGRAIL-SPECOS-002: WorkTask Run Evidence

## 1. Summary

Give every WorkTask execution generation a durable, restart-safe run record and
an inspectable trace assembled from existing WorkTask events, Conversation,
token usage, gate results, and Git facts. This Feature deepens WorkTask; it does
not introduce a second event store or copy full transcripts into SQLite.

### Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-002.R01` | Each claimed `run_seq` creates exactly one durable WorkTask run record before launch side effects. |
| `BUGRAIL-SPECOS-002.R02` | Events created for new runs carry `run_seq`; legacy unscoped events remain readable. |
| `BUGRAIL-SPECOS-002.R03` | A run trace projects its Spec reference, effective Agent/model config, Session, Worktree/Git coordinates, timeline, gates, token use, diff, outcome, and failure category. |
| `BUGRAIL-SPECOS-002.R04` | Trace queries use references and capped summaries; they do not persist raw prompts, transcripts, secrets, or full command output. |
| `BUGRAIL-SPECOS-002.R05` | Task Detail lists run generations and opens one run without changing existing task lifecycle behavior. |

PRD coverage: `P-DC-05`, `P-DC-13`, `P-DC-16`, `P-DC-18`.

## 2. Architecture

| Existing module | Change |
|---|---|
| `db/service/work_task_service.rs` | Create and settle run records in the same transactions as current claims/transitions. |
| `work_task/engine.rs` | Attach effective config, Session, Worktree, diff, and settlement facts to the current run. |
| `work_task_event` | Add nullable `run_seq`; current-run writers must populate it. |
| `token_usage_turn` + `conversation` | Read-only source for usage projection; no duplicated counters. |
| `commands/work_task.rs` + Axum handlers | Add list/get trace operations using existing transport conventions. |
| `components/tasks/task-detail-sheet.tsx` | Add Runs/Trace presentation with existing timeline patterns. |

The WorkTask module presents two new caller-facing operations:

```text
work_task_run_list(task_id, cursor?, limit) -> RunSummaryPage
work_task_run_trace(task_id, run_seq) -> WorkTaskRunTrace
```

## 3. Data Contract

### `work_task_run`

```text
task_id              INTEGER NOT NULL FK work_task(id) ON DELETE CASCADE
run_seq              INTEGER NOT NULL
round_kind           TEXT NOT NULL
conversation_id      INTEGER NULL
worktree_folder_id   INTEGER NULL
agent_type           TEXT NULL
model                 TEXT NULL
mode_id               TEXT NULL
base_branch           TEXT NULL
base_sha              TEXT NULL
work_branch           TEXT NULL
source_spec_id        TEXT NULL
source_spec_version   TEXT NULL
source_spec_hash      TEXT NULL
config_snapshot       TEXT NOT NULL  -- redacted effective values
outcome               TEXT NULL      -- review|done|failed|canceled|interrupted
failure_category      TEXT NULL
verdict               TEXT NULL
files_changed         INTEGER NULL
additions             INTEGER NULL
deletions             INTEGER NULL
merge_commit          TEXT NULL
started_at            TIMESTAMP NOT NULL
settled_at            TIMESTAMP NULL
PRIMARY KEY(task_id, run_seq)
```

`work_task_event.run_seq` is nullable for compatibility and indexed by
`(task_id, run_seq, id)`. Existing rows are not guessed or backfilled.

`WorkTaskRunTrace` contains the run row plus ordered event DTOs, latest gate
attempts, token totals queried through `conversation_id`, and repository-relative
evidence references. It is a projection, not a stored JSON blob.

## 4. Business Rules

1. `claim_for_run*` inserts the run row in the claim transaction. Duplicate
   `(task_id, run_seq)` is a consistency error, not an upsert.
2. Effective Agent/model/config is attached after resolution and before the ACP
   prompt. Secret-bearing config keys are replaced with a redaction marker.
3. Resume keeps the current `run_seq`; retry/return/merge generations follow
   existing increment rules and receive their own row.
4. Settlement writes outcome/failure/diff facts with the task transition.
5. Token totals are calculated from existing usage facts at read time and may
   be marked `pending_sync`; missing usage is not treated as zero.
6. Event ordering uses autoincrement `id`, never wall-clock timestamps.
7. Legacy tasks and pre-Feature runs show an explicit `legacy_unscoped` trace
   state instead of fabricated attribution.

## 5. Errors, Security, And Performance

| Error key | Condition |
|---|---|
| `workTask.run.notFound` | Task/run pair does not exist. |
| `workTask.run.inconsistent` | Claim or settlement facts violate one-run-per-generation invariants. |

- Trace access follows existing folder/task authorization.
- `config_snapshot` uses an allowlist (`agent`, `model`, `mode`, policy IDs),
  not a denylist over environment values.
- Run list is cursor-paginated by `run_seq DESC`, default 20, maximum 100.
- Trace events are cursor-paginated after 500 rows; UI does not fetch full
  history on task-list rendering.

## 6. UI States

- empty: no scoped run exists;
- loading: run list or selected trace pending;
- running: live run with durable events and possibly pending token sync;
- settled: outcome, gates, diff, cost/usage, and references visible;
- legacy: unscoped historical facts labeled as incomplete;
- failure: trace query error with existing task content preserved and retry.

## 7. Client Interaction Contract

This Feature implements Task Detail `Runs` and Run Inspector `Summary` and
`Timeline` from `design/specos-client-interaction-design.md`.

- `Runs` loads summaries only, newest `run_seq` first, 20 rows per page. Each
  row shows round kind, outcome, Agent/model, duration, token-sync state, gate
  count, changed-file count, and evidence quality.
- Selecting a row opens a `56rem` Run Inspector dialog and fetches the trace on
  demand. Board and Task Detail initial queries must not fetch event history.
- Summary shows the Delivery Rail plus Spec, Session, Worktree, Git coordinates,
  failure category, diff, usage, and gates. Missing facts render `Not recorded`,
  never zero or success.
- Timeline orders by durable event ID, supports event-kind filtering and cursor
  pagination, and links back to gate/evidence sections without copying raw
  transcript or command output.
- `task://changed` only invalidates/refetches the open run. After reconnect, the
  client refetches list and trace and does not reconstruct truth from missed
  live events.
- Legacy events appear in a separate `Unscoped history` group and cannot open a
  fabricated generation.

Client types and calls live in `src/lib/types.ts` and `src/lib/api.ts` as
`workTaskRunList` and `workTaskRunTrace`. UI modules are extracted as
`runs-tab`, `run-list`, `run-inspector-dialog`, `run-summary`, and
`run-timeline` under `src/components/tasks/specos/`.

Required render states are empty, first-load skeleton, incremental page load,
running/pending sync, settled, legacy, trace-not-found, reconnect refresh, and
transport failure with last-good content plus retry. The dialog traps focus,
supports Escape/close, becomes full-screen on narrow screens, and exposes event
filters and pagination to keyboard users.

## 8. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-002.AC01` | Start, retry, return, and merge generations create distinct run rows without changing existing status transitions. |
| `BUGRAIL-SPECOS-002.AC02` | A process restart can reconstruct the same scoped trace from durable facts. |
| `BUGRAIL-SPECOS-002.AC03` | Trace fields match Conversation, gate, token, event, and Git sources; missing sync is explicit. |
| `BUGRAIL-SPECOS-002.AC04` | Raw prompts, transcripts, provider keys, environment values, and uncapped outputs are absent from the stored/run wire contract. |
| `BUGRAIL-SPECOS-002.AC05` | Legacy events remain readable and are never attributed to an invented run. |
| `BUGRAIL-SPECOS-002.AC06` | Tauri and Axum return equivalent pagination, trace shapes, and error semantics. |
| `BUGRAIL-SPECOS-002.AC07` | Task Detail covers all states in Section 6 without loading trace history on board queries. |

## 9. Testing And Implementation Order

1. Migration/entity and claim/settlement transaction tests.
2. Projection tests with gate, Conversation, usage-sync, and legacy fixtures.
3. Command/transport pagination and redaction tests.
4. Task Detail run-list/Run Inspector tests for all UI, pagination, reconnect,
   responsive, and keyboard states.
5. Full existing WorkTask, desktop, and server regression suites.

No implementation Issue may claim `P-DC-13` evaluation aggregation; that is
owned by `BUGRAIL-SPECOS-008`.
