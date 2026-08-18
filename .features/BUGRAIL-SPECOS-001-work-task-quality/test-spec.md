---
testSpecId: "BUGRAIL-SPECOS-001.test"
testSpecVersion: "0.3"
status: draft
sourceSpec: ".features/BUGRAIL-SPECOS-001-work-task-quality/spec.md"
sourceSpecId: "BUGRAIL-SPECOS-001"
sourceSpecVersion: "0.3"
sourceSpecHash: "81b9aff1353243855173525f5a9111200f00a201674a338871f1b344084d657d"
approvalEvidence: "pending-independent-review"
riskTier: P1
---

# Test Spec: Spec-Linked WorkTask Quality

## Purpose

Independently verify that exact Spec binding and structured quality gates deepen
the existing WorkTask flow without creating a bypass, breaking legacy tasks, or
diverging between desktop and standalone-server transports.

This document specifies evidence to produce. Checkboxes and commands do not
claim that execution has occurred.

## Source Verification

Before running any test, compute SHA-256 for the source Feature Spec. A mismatch
with the front matter marks this Test Spec stale and blocks all evidence from
satisfying review.

## Coverage Matrix

| Acceptance | Test cases | Evidence | Gate |
|---|---|---|---|
| `BUGRAIL-SPECOS-001.AC01` | `BUGRAIL-SPECOS-001.T01`, `T02` | DB assertions, command result, timeline | blocking |
| `BUGRAIL-SPECOS-001.AC02` | `BUGRAIL-SPECOS-001.T03-T09` | structured errors, unchanged DB state | blocking |
| `BUGRAIL-SPECOS-001.AC03` | `BUGRAIL-SPECOS-001.T10` | existing WorkTask regression suite | blocking |
| `BUGRAIL-SPECOS-001.AC04` | `BUGRAIL-SPECOS-001.T11-T13` | ordered gate rows and UI state | blocking |
| `BUGRAIL-SPECOS-001.AC05` | `BUGRAIL-SPECOS-001.T14`, `T15` | actor authorization assertions | blocking |
| `BUGRAIL-SPECOS-001.AC06` | `BUGRAIL-SPECOS-001.T16-T20` | merge/complete error and unchanged review state | blocking |
| `BUGRAIL-SPECOS-001.AC07` | `BUGRAIL-SPECOS-001.T21`, `T22` | existing merge/complete success path | blocking |
| `BUGRAIL-SPECOS-001.AC08` | `BUGRAIL-SPECOS-001.T23`, `T24` | retry generation evidence | blocking |
| `BUGRAIL-SPECOS-001.AC09` | `BUGRAIL-SPECOS-001.T25-T27` | stale/rebind timeline and decision | blocking |
| `BUGRAIL-SPECOS-001.AC10` | `BUGRAIL-SPECOS-001.T28-T31` | Tauri/Axum parity, UI interaction, and wire snapshots | blocking |

## Scenarios

### Spec Binding

| ID | Branch | Scenario | Expected result |
|---|---|---|---|
| `T01` | happy | Preview a valid repository-local Feature Spec, select AC IDs/gates, and bind with the preview hash. | Preview returns exact parsed metadata; bind stores the server-resolved snapshot and a `spec_contract_bound` event in one transaction. |
| `T02` | flow | Read the contract after process restart. | Exact reference and snapshots are unchanged. |
| `T03` | error | Relative path escapes the project root. | `InvalidInput/workTask.specContract.invalid`; no contract/event change. |
| `T04` | edge | Symlink resolves outside the project root. | Same rejection as `T03`. |
| `T05` | error | Source file is missing or exceeds 1 MiB. | Rejected before DB mutation. |
| `T06` | error | Feature Spec identity/version is absent, malformed, or differs from the requested contract expectations. | Preview/bind is rejected with mismatch detail. |
| `T07` | concurrency | File SHA-256 changes after preview but before bind. | Expected-hash comparison rejects bind; no implicit rebind. |
| `T08` | error | Selected AC is absent from source. | Rejected with unknown AC ID. |
| `T09` | limit | Duplicate gate ID, unsupported type, reusable human approval, more than 32 gates, or oversized snapshot/policy. | Rejected with bounded structured error. |

### Compatibility And Gate Attempts

| ID | Branch | Scenario | Expected result |
|---|---|---|---|
| `T10` | compatibility | Create and complete an unbound legacy task through existing paths. | Behavior and wire fields remain compatible. |
| `T11` | happy | The correlated WorkTask engine records running then passed preflight for current `run_seq`. | Both attempts remain ordered and latest applicable result passes. |
| `T12` | error | An internal producer attempts failed/blocked/waived without a reason or without run correlation. | Rejected without a gate row. |
| `T13` | flow | Load Task Detail with several attempts. | Latest decision and auditable history are distinguishable. |
| `T14` | security | Agent output or an arbitrary direct client payload attempts to record/pass preflight or human approval. | No public generic gate-record path exists; trusted command rejects the attempt. |
| `T15` | happy | Authenticated user invokes `work_task_gate_human_decide` with a non-empty reason; repeat against a gate whose policy forbids waiver. | Approval or policy-allowed waiver is recorded with derived actor/time/reason; forbidden waiver is rejected without a row. |

### Merge And Completion Enforcement

| ID | Branch | Scenario | Expected result |
|---|---|---|---|
| `T16` | error | Required gate has no attempt. | Merge rejected; task remains `review`; unmet ID returned. |
| `T17` | error | Required gate is running, or required preflight has no configured producer. | Same invariant as `T16`; missing producer is reported as blocked. |
| `T18` | error | Required gate failed. | Same invariant; evidence summary is available. |
| `T19` | error | Required gate blocked. | Same invariant. |
| `T20` | flow | No-change task requests complete with unmet gate. | Completion rejected with no cleanup/state side effect. |
| `T21` | happy | All required gates pass and task has changes. | Existing merge flow proceeds and still verifies Git truth. |
| `T22` | happy | All required gates pass and task has no changes. | Existing completion flow proceeds. |

### Retry, Staleness, And Rebind

| ID | Branch | Scenario | Expected result |
|---|---|---|---|
| `T23` | edge | Retry starts a new `run_seq` after a passing non-reusable gate. | New run is ineligible until a new attempt passes. |
| `T24` | edge | Retry with a reusable passing preflight whose `verified_head` is unchanged, then change Worktree `HEAD`. | It remains eligible only while both verified/current heads and Spec hash match; the head change invalidates it. |
| `T25` | error | Source Spec changes after binding. | Decision reports stale and merge/complete is rejected. |
| `T26` | flow | User explicitly rebinds in `review`, then attempts the same in every forbidden state. | Review rebind succeeds and preserves old hash in the timeline; queued/preparing/running/awaiting_input/merging/done reject without mutation. |
| `T27` | concurrency | Spec changes between validation and persistence/decision. | Operation detects mismatch or fails atomically; no mixed reference. |

### Transport And UI

| ID | Branch | Scenario | Expected result |
|---|---|---|---|
| `T28` | compatibility | Call preview, bind, read, gate-list/decision, and human-decision operations through Tauri and Axum adapters. | Same request/result/error wire contract. |
| `T29` | flow | Use the Board chip and Task Detail Contract tab to preview, bind, inspect, approve/waive, and rebind. | Every action calls the typed API, renders authoritative reasons, and reaches the expected panel/dialog state. |
| `T30` | security | Error/evidence includes secret-like environment content. | UI/wire only exposes capped redacted references and summaries. |
| `T31` | accessibility | Complete preview/bind and approval/waiver dialogs by keyboard in narrow and wide layouts. | Focus is trapped/restored, labels and errors are announced, status has text/icon semantics, and no action depends on hover or color. |

## Migration Verification

- Apply migrations from a database created at the pre-Feature schema.
- Verify existing WorkTask rows and settings are unchanged.
- Verify foreign-key cascade for contract and gate results.
- Verify expected indexes through SQLite metadata.
- Apply down migration on a fixture and confirm existing WorkTask tables remain
  readable by the previous schema.
- Interrupt a transaction that writes contract/gate data plus timeline event;
  confirm neither side commits alone.

## Commands

Commands are execution inputs only:

```bash
shasum -a 256 .features/BUGRAIL-SPECOS-001-work-task-quality/spec.md
pnpm lint
pnpm test
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml --features test-utils
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --features test-utils -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --no-default-features --bin codeg-server --lib
cargo clippy --manifest-path src-tauri/Cargo.toml --no-default-features --bin codeg-server --lib -- -D warnings
cargo check --manifest-path src-tauri/Cargo.toml --no-default-features --bin codeg-mcp
```

Focused tests should be added for the new Rust modules, command cores,
migrations, handlers, TypeScript wire types, and Task UI states before the full
matrix runs.

## Evidence Contract

Normalized results under `tests/results/` must include:

- `sourceSpecId`, `sourceSpecVersion`, and `sourceSpecHash`;
- BugRail commit and dirty-state metadata;
- test case and acceptance IDs;
- owner, command, environment, attempt count, outcome, and duration;
- raw-output reference;
- flake classification where a retry occurred;
- migration fixture version and desktop/server transport when applicable.

Local developer output without source binding cannot satisfy the gate.

## Acceptance

- [ ] Source hash matches this Test Spec.
- [ ] `BUGRAIL-SPECOS-001.T01-T31` have evidence or an approved, documented non-applicability
  decision.
- [ ] Every `BUGRAIL-SPECOS-001.AC01-AC10` has at least one passing blocking result.
- [ ] No merge/complete bypass is found through Tauri, Axum, retry, rebind,
  stale source, Agent actor, or legacy preflight paths.
- [ ] Legacy WorkTask regression tests pass.
- [ ] Independent review consumes normalized results before release status can
  become ready.
