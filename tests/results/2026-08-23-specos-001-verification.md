# BUGRAIL-SPECOS-001 Verification Resume

## Decision

**NOT VERIFIED / BLOCKED.** The compatibility loop resumed from the first
dependency-ready, non-verified item (`issue-001`) and evaluated the existing
implementation through `issue-005`. No issue status was promoted and no local
ship was attempted.

## Binding

| Field | Value |
|---|---|
| Feature | `BUGRAIL-SPECOS-001` |
| Spec version | `0.3` |
| Spec SHA-256 | `79160488f65ae762decaa6db4987c15a783f61c886588c1e9157fc1bb40ab0d0` |
| Test Spec SHA-256 | `7e3e34408f23cef03b8cf6b099864b40999ad0f151525fb16d48171dd8ee09a9` |
| Branch / HEAD | `main` / `3c7240c08184c28658330f15e5bcd08d35ee8c4d` |
| Worktree | dirty before verification; preserved in place |
| Approval | explicit user approval, `2026-08-23` |

All five local issue records carry the same current Spec hash. Historical
`2026-08-21` evidence is bound to an earlier document state and is superseded
for this verification decision; it was not rewritten.

## Executed Matrix

| Command | Result |
|---|---|
| `pnpm test` | pass: 319 files, 4,227 tests |
| `pnpm build` | pass: Next build, 32 static pages |
| `cargo test --manifest-path src-tauri/Cargo.toml --features test-utils` | pass: library 2,731 passed / 1 ignored; integration suites pass; declared external fixtures remain ignored |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --features test-utils -- -D warnings` | pass |
| `cargo test --manifest-path src-tauri/Cargo.toml --no-default-features --bin codeg-server --lib` | pass: 2,707 passed / 1 ignored |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --no-default-features --bin codeg-server --lib -- -D warnings` | pass |
| `cargo check --manifest-path src-tauri/Cargo.toml --no-default-features --bin codeg-mcp` | pass |
| focused `specos_spec_binding` | pass: 12 tests |
| focused `task-traceability-panel` | pass: 6 tests |
| focused ESLint and Rust clippy for changed verification files | pass |
| `pnpm lint` | fail: 42 errors and 1 warning in pre-existing unrelated `agent/`, `scripts/`, and `tests/fixtures/` changes |
| `cargo fmt --all -- --check` | fail: pre-existing import ordering in untracked `src-tauri/src/commands/memory.rs`; changed Rust verification file is formatted |

The package requests Node `24.x`; this run used Node `25.6.0` with pnpm
`11.9.0`, producing an engine warning.

## Direct Evidence

- Axum integration coverage passes for Spec preview/bind validation (`T01-T09`),
  human gate actor/policy/reason enforcement (`T14-T15`), stale binding and
  rebind restrictions (`T25-T26`), and the shared-core portion of `T28`.
- Merge rejection preserves `review`, returns exact unmet gate IDs, and emits
  auditable block events for missing, running, failed, and blocked gate states.
  Failed-gate evidence is persisted and read back (`T16`, the running branch of
  `T17`, and `T18-T19`).
- UI tests exercise exact AC/gate selection in preview-to-bind and keyboard
  entry/roving focus in the Contract tab.
- Review-it first identified six assertion gaps. Generic-writer status and
  no-write behavior, allowed waiver, exact unmet IDs, persisted failed evidence,
  merge blocking, and UI selection/keyboard paths were added. The follow-up
  review confirmed the focused suites pass and identified the remaining T17
  producer-path gap below.

## Blocking Gaps

- `T17`: no engine-level fixture executes required preflight with no configured
  producer and asserts the persisted `producer_unavailable` blocked result.
- `T20`: no real unchanged Git worktree fixture observes rejected completion
  with `deleteWorktree=true` and proves there is no cleanup side effect.
- `T27-T28`: atomic Spec-change behavior and an actual Tauri invocation were not
  independently executed; Axum and shared-core parity alone are insufficient.
- `T29-T31`: full bind/rebind/waive UI journeys, secret-redaction checks, and
  required light/dark plus narrow/wide screenshot evidence are incomplete.
- Migration verification does not yet retain the required pre-feature rollback
  fixture and interrupted-transaction evidence for this exact Spec hash.
- Raw command logs were not persisted to a stable artifact reference, and this
  verifier strengthened tests during the run. That does not satisfy the Test
  Spec's independent-verifier separation and raw-evidence retention contract.
- Repository-level lint and format gates remain red because of unrelated dirty
  files listed above. They were not modified or hidden.

Resume only after these gaps have direct evidence. Keep `issue-001` through
`issue-005` non-verified until the full blocking matrix passes against the bound
Spec hash.
