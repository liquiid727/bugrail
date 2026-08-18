---
id: BUGRAIL-SPECOS-001
version: "0.3"
title: "Spec-Linked WorkTask Quality"
status: draft
changeType: work-task-deepening
prd: ".prd/prd-specos-delivery-control.md"
slicePrd: ".prd/prd-specos-work-task-quality.md"
design: "design/specos-control-plane-design.md"
clientDesign: "design/specos-client-interaction-design.md"
adr: "design/adr/ADR-001-embed-specos-in-work-task.md"
codeBaseline: "55545d43"
tests:
  requiredBranches:
    - happy
    - error
    - edge
    - flow
    - compatibility
---

# BUGRAIL-SPECOS-001: Spec-Linked WorkTask Quality

## 1. Summary

### 1.1 Scope

Extend the existing CodeG-derived WorkTask delivery module so a task can bind to
one exact Feature Spec version and acceptance snapshot, record structured gate
attempts, and reject merge or no-change completion until all required gates are
eligible.

This Feature uses the existing WorkTask state machine, ACP runtime, Git
Worktrees, SQLite database, command-core functions, Tauri/Axum transports, live
event adapters, and Tasks UI.

### 1.2 Requirement Sources

| Product requirement | Feature requirement |
|---|---|
| `P-DC-01`, `P-WTQ-01`, `P-WTQ-02` | `R01`, `R02`, `R03` |
| `P-DC-02`, `P-WTQ-03` | `R04` |
| `P-DC-03`, `P-DC-04`, `P-WTQ-04`, `P-WTQ-05` | `R05`, `R06`, `R07` |
| `P-DC-18`, `P-WTQ-07` | `R08` |
| `P-WTQ-08` | `R09`, `R10` |
| `P-DC-16`, `P-DC-17` | `R04`, `R07` |

### 1.3 Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-001.R01` | A WorkTask may bind to one repository-local Feature Spec using its ID, version, relative path, and SHA-256. |
| `BUGRAIL-SPECOS-001.R02` | Binding validates that the file exists under the active project root and that its declared ID/version and computed hash match the request. |
| `BUGRAIL-SPECOS-001.R03` | The binding stores a snapshot of selected acceptance criteria and the required gate policy. |
| `BUGRAIL-SPECOS-001.R04` | WorkTasks without a Spec contract preserve current create, run, preflight, review, merge, complete, retry, and cleanup behavior. |
| `BUGRAIL-SPECOS-001.R05` | A contract-bound task records append-only gate attempts scoped to `task_id` and `run_seq`, including status, actor, evidence, reason, and timestamps. |
| `BUGRAIL-SPECOS-001.R06` | The first Feature supports engine-owned `preflight` gates and trusted-user `human_approval` gates only; Agent verdicts and arbitrary client payloads cannot record or pass either gate. |
| `BUGRAIL-SPECOS-001.R07` | Existing `work_task_merge` and `work_task_complete` commands return an explainable typed error and leave task state unchanged when a required gate is unmet. |
| `BUGRAIL-SPECOS-001.R08` | Task Detail renders the exact Spec reference, acceptance criteria, latest required gate states, evidence summaries, and merge block reason. |
| `BUGRAIL-SPECOS-001.R09` | A retry starts a new `run_seq`; previous gate attempts remain auditable but do not pass the new run unless the gate policy marks them reusable. |
| `BUGRAIL-SPECOS-001.R10` | If the current Spec file hash differs from the bound hash, the task is marked stale for merge eligibility until a user explicitly rebinds it. |

## 2. Architecture

### 2.1 Existing Modules

| Decision | Existing path | Change |
|---|---|---|
| WorkTask orchestration | `src-tauri/src/work_task/engine.rs` | Extend settlement/preflight integration; retain current state machine. |
| State and transactional audit | `src-tauri/src/db/service/work_task_service.rs` | Add contract/gate repository operations and gate-decision transaction records. |
| Entity types | `src-tauri/src/db/entities/work_task*.rs` | Add contract and gate-result entities; do not replace `work_task`. |
| Migration | `src-tauri/src/db/migration/` | Add one ordered migration for new tables and indexes. |
| Commands | `src-tauri/src/commands/work_task.rs` | Add bind/read/list/decision and explicit human approval commands; keep engine gate writes internal; enforce the decision in existing merge/complete core functions. |
| Server transport | `src-tauri/src/web/handlers/work_task.rs`, `web/router.rs` | Mirror new commands; preserve existing routes. |
| Rust/TS wire types | `src-tauri/src/models/work_task.rs`, `src/lib/types.ts` | Add matching contract, gate, and decision DTOs. |
| Frontend client | `src/lib/api.ts` | Add transport-independent calls using the existing command transport. |
| Tasks UI | `src/components/tasks/task-detail-sheet.tsx` | Add Spec/AC/Gate sections and blocked/stale states. |
| Existing preflight UI | `src/components/tasks/task-card.tsx`, `task-detail-sheet.tsx` | Preserve projection while structured gate results become authoritative for bound tasks. |

### 2.2 Interaction Flow

```text
user selects a Feature Spec
  -> preview command parses project-relative path and returns ID/version/hash/AC
  -> user selects AC and gate policy from the preview
  -> bind command revalidates file/hash and selected AC
  -> transaction upserts work_task_contract + records contract_bound event

agent run settles to review
  -> existing preflight runs
  -> bound task writes structured preflight gate attempt
  -> existing preflight projection remains available to old UI code

user requests merge/complete
  -> revalidate bound Spec hash
  -> load latest gate attempts for current run_seq
  -> evaluate gate policy
  -> eligible: continue existing merge/complete CAS flow
  -> unmet/stale: record gate_decision event, return typed error, keep review state
```

### 2.3 Module Interface

The external interface remains the WorkTask command family. The implementation
adds these command-core operations:

```rust
work_task_contract_preview_core(task_id, source_spec_path) -> WorkTaskContractPreview
work_task_contract_bind_core(task_id, draft) -> WorkTaskContractInfo
work_task_contract_get_core(task_id) -> Option<WorkTaskContractInfo>
record_preflight_gate_internal(task_id, run_seq, preflight) -> WorkTaskGateResultInfo
work_task_gate_human_decide_core(task_id, gate_id, decision, reason, actor_context) -> WorkTaskGateResultInfo
work_task_gate_list_core(task_id, run_seq) -> Vec<WorkTaskGateResultInfo>
work_task_gate_decision_core(task_id) -> WorkTaskGateDecision
```

Gate evaluation and Spec parsing remain internal functions/modules. They are not
registered as public plugins.

## 3. Data Model

### 3.1 `work_task_contract`

```text
task_id              INTEGER PRIMARY KEY FK work_task(id) ON DELETE CASCADE
source_spec_id       TEXT NOT NULL
source_spec_version  TEXT NOT NULL
source_spec_path     TEXT NOT NULL
source_spec_hash     TEXT NOT NULL
acceptance_criteria  TEXT NOT NULL  -- JSON array snapshot
gate_policy          TEXT NOT NULL  -- JSON policy snapshot
created_at           TIMESTAMP NOT NULL
updated_at           TIMESTAMP NOT NULL
```

Indexes:

- `idx_work_task_contract_spec_id(source_spec_id)`

### 3.2 `work_task_gate_result`

```text
id          INTEGER PRIMARY KEY AUTOINCREMENT
task_id     INTEGER NOT NULL FK work_task(id) ON DELETE CASCADE
run_seq     INTEGER NOT NULL
gate_id     TEXT NOT NULL
gate_type   TEXT NOT NULL
status      TEXT NOT NULL
required    BOOLEAN NOT NULL
reusable    BOOLEAN NOT NULL DEFAULT false
actor       TEXT NOT NULL
evidence    TEXT NULL      -- JSON references and capped summaries
reason      TEXT NULL
started_at  TIMESTAMP NOT NULL
finished_at TIMESTAMP NULL
```

Indexes:

- `idx_work_task_gate_task_run(task_id, run_seq, gate_id, id)`

### 3.3 DTOs

```text
WorkTaskContractDraft
  source_spec_path: string
  expected_source_spec_hash: string
  selected_acceptance_criteria_ids: string[]
  gate_policy: WorkTaskGatePolicy

WorkTaskContractPreview
  source_spec_id: string
  source_spec_version: string
  source_spec_path: string
  source_spec_hash: string
  acceptance_criteria: AcceptanceCriterionSnapshot[]
  current_binding_hash: string | null

WorkTaskGatePolicy
  gates: GateRequirement[]

GateRequirement
  id: string
  type: preflight | human_approval
  required: boolean
  reusable: boolean
  allow_waiver: boolean

WorkTaskGateDecision
  eligible: boolean
  stale_spec: boolean
  required: GateDecisionItem[]
  unmet: GateDecisionItem[]
  waived: GateDecisionItem[]
```

Rust owns serialization names. TypeScript mirrors the exact snake_case wire
shape used by current WorkTask DTOs.

### 3.4 Migration And Rollback

- Migration adds two tables and indexes; no existing row is rewritten.
- Existing tasks have no contract row and follow the legacy path.
- Down migration drops gate results before contracts.
- A rollback binary ignores the new tables. No current column or state value is
  changed, so application downgrade remains possible.

## 4. Command And Error Contract

### 4.1 New Commands

| Command | Request | Result |
|---|---|---|
| `work_task_contract_preview` | task ID + repository-relative Spec path | parsed identity, hash, available AC, and current binding hash |
| `work_task_contract_bind` | task ID + contract draft | validated contract |
| `work_task_contract_get` | task ID | contract or `null` |
| `work_task_gate_list` | task ID + optional run sequence | gate attempts |
| `work_task_gate_decision` | task ID | current explainable decision |
| `work_task_gate_human_decide` | task ID + gate ID + approve/waive + reason | trusted-user approval or policy-allowed waiver |

Desktop and server modes expose the same behavior through existing Tauri and
Axum conventions.

### 4.2 Existing Commands

`work_task_merge` and `work_task_complete` keep their current names and request
shapes. A contract-bound task adds a precondition: current Spec hash and gate
decision must be eligible before the existing state transition begins.

### 4.3 Errors

| App error | i18n key | Condition | State effect |
|---|---|---|---|
| `InvalidInput` | `workTask.specContract.invalid` | invalid path, metadata, hash, AC, or gate policy | none |
| `TaskExecutionFailed` | `workTask.specContract.stale` | current source hash differs | none |
| `TaskExecutionFailed` | `workTask.qualityGate.unmet` | required gate is not passed/validly waived | none |
| `PermissionDenied` | `workTask.qualityGate.invalidWaiver` | Agent/non-human or reasonless waiver | none |

The detail payload includes task ID, source Spec ID/version when relevant, and
unmet gate IDs. It must not include full command output or secrets.

## 5. Business Rules

### 5.1 Spec Binding

1. Resolve `source_spec_path` relative to the task's live project folder.
2. Canonicalize the path and reject escape outside the project root.
3. Read the Feature Spec and validate declared ID/version.
4. Compute SHA-256 and compare with `expected_source_spec_hash` from the preview.
5. Resolve selected AC identifiers from the file; the client cannot submit AC text.
6. Validate gate IDs are unique and every required gate has a supported type.
7. Upsert the contract and record `spec_contract_bound` in one transaction.

Binding an already contracted task is an explicit rebind. It preserves old gate
attempts, records old/new hashes in the timeline, and invalidates non-reusable
eligibility for the current run.

Rebind is allowed only in `todo`, `review`, `failed`, or `canceled`. It is
rejected in `queued`, `preparing`, `running`, `awaiting_input`, `merging`, or
`done`, so one execution generation cannot change its acceptance contract while
work or landing is in flight.

### 5.2 Gate Status

Persisted attempt writes:

```text
running, then passed | failed | blocked  (engine-owned preflight attempts)
passed | waived                         (trusted-user decision attempt)
```

- `passed` requires evidence appropriate to the gate type.
- `failed`, `blocked`, and `waived` require a reason.
- `human_approval` can pass or waive only through
  `work_task_gate_human_decide`; a `preflight` gate may only be waived through
  that command when its snapshotted policy has `allow_waiver: true`.
- Actor identity is derived from the authenticated command context, never
  accepted from request JSON.
- `human_approval` gates must set `reusable: false`. A reusable preflight result
  is applicable only when its evidence `verified_head` equals current Worktree
  `HEAD` and the bound Spec hash is unchanged.
- A required preflight gate with no configured preflight command is `blocked`
  with reason `producer_unavailable`; it is never treated as passed.
- Agent verdict may be referenced as evidence but cannot satisfy either
  supported gate. Test, review, and security-review gates are introduced only
  by later Feature Specs with a trusted producer path.

### 5.3 Eligibility

A contract-bound task is eligible when:

- source Spec ID/version/hash still match the bound file;
- every required gate has a latest applicable attempt for the current
  `run_seq`, or a reusable passing attempt allowed by policy;
- every applicable attempt is `passed` or validly `waived`;
- no required gate is `running`, `failed`, or `blocked`.

The decision is calculated from persisted facts. Live event delivery does not
participate in correctness.

### 5.4 Compatibility

- No-contract task: retain current behavior.
- Bound task: structured gate decision is authoritative.
- Existing `preflight` field: retained as a compatibility projection.
- Existing WorkTask status values: unchanged.
- Existing `codeg` command, route, URI, database-file, and environment names:
  unchanged.

## 6. Security And Limits

- Spec file maximum size: 1 MiB for binding validation.
- Acceptance criteria snapshot maximum: 64 items and 64 KiB serialized.
- Gate policy maximum: 32 gates and 32 KiB serialized.
- Evidence maximum: 64 KiB serialized per attempt; command output uses the
  existing capped tail rather than full stdout/stderr.
- Symlink/canonical path escape is rejected.
- Preflight results can be written only by the correlated WorkTask engine path.
- Human gate actors come from the authenticated Tauri/server command context;
  request JSON contains no actor field.

## 7. Client Interaction Contract

This Feature implements the `Contract` tab and Contract stop of the Delivery
Rail defined in `design/specos-client-interaction-design.md`.

### 7.1 Surfaces And Actions

- Board card shows the short Spec ID/version and the highest-priority gate or
  stale state; either chip opens Task Detail on `Contract`.
- An unbound task shows a repository-relative path picker, not an unrestricted
  filesystem picker. `Preview` calls `work_task_contract_preview`.
- The preview shows exact ID, version, path, SHA-256 and all available AC. The
  user selects AC IDs, configures supported gates, then confirms `Bind`.
- Bind submits the preview hash as an optimistic-concurrency token. A changed
  file returns the stale error and keeps the preview for comparison/retry.
- A bound task shows full selected AC text, gate attempts, actor, reason,
  evidence summary, verified head, timestamps, and the computed decision.
- `Approve` and policy-allowed `Waive` use a confirmation dialog. Waive always
  requires a non-empty reason; actor identity is never editable.
- `Rebind` repeats preview/compare and explicitly lists the old/new hash and AC
  selection before confirmation.
- Merge/Complete dialogs show every unmet reason. The backend remains the final
  enforcement point; disabled buttons are only guidance.

### 7.2 Client Boundary And Placement

`src/lib/types.ts` mirrors preview, contract, gate-attempt, and decision DTOs.
`src/lib/api.ts` exposes `workTaskContractPreview`, `workTaskContractBind`,
`workTaskContractGet`, `workTaskGateList`, `workTaskGateDecision`, and
`workTaskGateHumanDecide`; components do not call a transport directly.

The feature UI is extracted under `src/components/tasks/specos/` as
`contract-tab`, `contract-bind-dialog`, `gate-attempt-list`, and
`gate-decision-summary`. `task-detail-sheet.tsx` owns tab selection and passes
the selected task ID only.

### 7.3 Required States

| State | Visible behavior | Allowed action |
|---|---|---|
| unbound | Explanation and bind control | Preview Spec |
| previewing | Existing task remains visible; form is busy | Cancel |
| previewed | Identity/hash/AC/gates review | Bind or cancel |
| eligible | All required gates and evidence shown | Merge/Complete |
| pending | Pending/running gate and reason | Inspect evidence |
| failed/blocked | Failure reason and capped evidence | Retry run or waive if policy permits |
| stale | Old/new hash comparison | Re-preview/Rebind |
| waived | Human actor, reason, time, policy | Inspect history |
| transport error | Last good facts remain visible | Retry fetch |

All strings use `next-intl`; focus returns to the originating control after a
dialog closes, and status is expressed with icon plus text, never color alone.

## 8. Acceptance Criteria

| ID | Criterion | Requirements |
|---|---|---|
| `BUGRAIL-SPECOS-001.AC01` | Previewing then binding a valid repository-local Feature Spec stores its exact ID, version, path, preview-matched hash, selected AC snapshot, and gate policy. | `R01-R03` |
| `BUGRAIL-SPECOS-001.AC02` | Path escape, missing/oversized file, metadata mismatch, hash mismatch, unknown AC, duplicate gate ID, and oversized policy are rejected without changing the task. | `R02`, `R03` |
| `BUGRAIL-SPECOS-001.AC03` | A legacy task without a contract passes existing WorkTask regression behavior. | `R04` |
| `BUGRAIL-SPECOS-001.AC04` | Gate attempts are append-only, scoped to `run_seq`, and displayed with actor, evidence summary, reason, and time. | `R05`, `R08`, `R09` |
| `BUGRAIL-SPECOS-001.AC05` | Agent verdicts and direct/arbitrary client gate payloads cannot satisfy preflight or human approval; only the correlated engine and trusted human command paths can write them. | `R06` |
| `BUGRAIL-SPECOS-001.AC06` | Merge and no-change completion remain in `review` and return unmet gate IDs when any required gate is ineligible. | `R07` |
| `BUGRAIL-SPECOS-001.AC07` | All eligible gates allow the unchanged existing merge/complete flow to proceed. | `R07` |
| `BUGRAIL-SPECOS-001.AC08` | Retry does not reuse non-reusable results, while an explicitly reusable passing result remains eligible according to policy. | `R09` |
| `BUGRAIL-SPECOS-001.AC09` | A changed source Spec hash blocks merge until explicit rebind; old attempts remain auditable. | `R10` |
| `BUGRAIL-SPECOS-001.AC10` | Desktop and standalone-server transports expose equivalent preview/contract/gate behavior and error semantics, and the client covers every state in Section 7. | `R01-R10` |

## 9. Testing Strategy

- Pure Rust tests: Spec path/hash validation, gate evaluation, actor rules,
  reusable attempts, stale binding.
- SeaORM tests: migration up/down, FK cascade, indexes, append-only attempts,
  transaction rollback.
- WorkTask integration tests: legacy compatibility, blocked merge/complete,
  eligible merge/complete, retry generation, preflight projection.
- Transport tests: Tauri command registration and Axum route/handler parity.
- React/Vitest: preview/bind/rebind, gate decisions, every state in Section 7,
  keyboard/focus behavior, and merge-dialog decision display.
- Build gates: `pnpm lint`, `pnpm test`, `pnpm build`, Rust desktop/server/MCP
  check/test/clippy commands declared in `AGENTS.md`.

The matching `test-spec.md` is independently derived and bound to this exact
Spec version and content hash.

## 10. Implementation Order

1. Migration, entities, DTOs, and pure gate decision.
2. Spec binding and gate repositories with command-core tests.
3. Merge/complete enforcement and preflight compatibility.
4. Tauri/Axum/TypeScript preview and command parity.
5. Contract tab, bind/rebind dialogs, card chips, and merge decision states.
6. Independent verification and normalized evidence.

Implementation Issues live under `.issues/` and carry this Spec's exact hash.

## 11. Risks And Assumptions

- Assumption: the active project has one repository root available through the
  current folder record.
- Risk: existing command cores return `DbError` in several paths. The
  implementation must preserve current callers while mapping new domain
  conditions to `AppCommandError`; do not stringify structured errors away.
- Risk: Spec front matter parsing must use an existing dependency or a small
  deterministic parser. Adding a dependency requires explicit justification and
  lockfile update.
- Risk: remote users need a trustworthy human actor identity. The first slice
  records the identity currently available; stronger multi-user authorization is
  deferred and must not be implied.

## 12. Definition Of Ready

- [ ] ADR-001 is accepted.
- [x] Rebind-state policy is fixed to `todo/review/failed/canceled`.
- [ ] Feature Spec `0.3` is approved at an exact SHA-256.
- [ ] Matching Test Spec is independently approved at the same source hash.
- [ ] Implementation and verification Issues reference that hash.

## 13. Definition Of Done

- [ ] `AC01-AC10` have passing, version-bound evidence.
- [ ] Legacy WorkTask regression suite passes.
- [ ] Desktop/server behavior and error contracts match.
- [ ] Migration and rollback evidence is recorded.
- [ ] Independent review finds no blocking gate-bypass path.
- [ ] Release decision references normalized results for this Spec version.
