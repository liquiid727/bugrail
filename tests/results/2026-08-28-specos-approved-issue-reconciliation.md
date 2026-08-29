# Approved Issue Reconciliation — 2026-08-28

## Scope and decision

This record covers the approved `BUGRAIL-SPECOS-001` through `017` and
`BUGRAIL-SPECOS-028` Feature/Test Spec set. Draft Features `018-027` and
`029-036` were not executed. Live Tauri/Axum journeys, browser journeys and
screenshots, external TencentDB validation, and unexpected out-of-scope
debugging were intentionally skipped.

Repository baseline: `f6fefea6753c01d1257509d3e91d17117fee8117`.
The repository requests Node `24.x`; the verification environment used Node
`25.6.0` and pnpm `11.9.0`.

## Approved artifact bindings

The hashes below are the SHA-256 values of the complete Feature and Test Spec
files used for this reconciliation.

| Feature | Version | Feature Spec SHA-256 | Test Spec SHA-256 |
|---|---:|---|---|
| `001` | `0.3` | `79160488f65ae762decaa6db4987c15a783f61c886588c1e9157fc1bb40ab0d0` | `7e3e34408f23cef03b8cf6b099864b40999ad0f151525fb16d48171dd8ee09a9` |
| `002` | `0.2` | `f9342aa89f379719d89fc8bcbcc7e612208652ec42fa9dd464c671185587fee9` | `dc82ce17bb3a21b81302207d3bc9c5dcdfd21cb0865a608bead539768b61318a` |
| `003` | `0.2` | `6dcbc16b16a05b5d98fcadd7393098558d2fbe2d334d40af4ed4672b5df38009` | `89e5bb0dbeb0cf5b9087a2794d68d5deeb2a5f725d4d3ea8643f9f95ab4bfe92` |
| `004` | `0.2` | `81ee40fe121cef77cb45120768e949a06e5883ea5b83cb64f6106a29fb8a9d4d` | `8deae207f7db73d44bd36f96fe92ab5ecc62df67254fba9a13224a93977e4b05` |
| `005` | `0.2` | `1d5ff5e900247259bab0b2ad292246fb92dbcccc7e089b5295425ecab2c47678` | `7ed765c13a5c3de7c40347221162ef2408ba3058a7e642950e06ce603be7e99d` |
| `006` | `0.2` | `9a97e6eaecf1248ec4494b17f484f687628d13d27e8bda68d164e6e9755afabb` | `dcd67c95887899be4ef0658ea9e4dc83c293c20abe63550794ce02764c869904` |
| `007` | `0.2` | `fc10e29a5c2be849a1573875aee7b3fb73f0292dcbfb5e23623c88318f6d669f` | `15e2f3c939fd0ef0847d68b6e074e6cbddb72b1a98f00e0a50660f93d0edb6cb` |
| `008` | `0.2` | `244f55f97115b3b067dc7fc031ee55bdf26879f30dcbc3a2d3b5ade24174046d` | `c437b5948e6ab4bc18fde98120011be4460153003d6098d9286f2e396f1c3412` |
| `009` | `0.2` | `0eb4d34c5db0677a58fafff40c9c963359455ea3d1750d4bb52fc53e707913a7` | `4ae27c01c675108cfeaa843a20109178a2e04cb6ed52104067bb919a0cc909c8` |
| `015` | `0.1` | `cc08a8aa814ea9dfd0658ceb8d0f4a625a960a0f2f8ffd3c4104f8363a9a783f` | `8c5de7b8c1fbe5450383cc80b2d49aefc9e810d04dfb896f963725b5ee7f0ae0` |
| `016` | `0.1` | `67a4fcb4682ca22c2994c7114376887e9e15a12c6b8daf54137ba051a47e7517` | `640a02f018d2e930069f728f000f577418e92a7469a672f5249248bd82cb7a75` |
| `017` | `0.2` | `f62824d31787ff774d962d942ffba0423fe78f28c77fa7b0e2e000d71dacfd58` | `480c2e58669ae43c29c8d119b90ac9d6e8c7abeb2ad298a0a043883311480cf6` |
| `028` | `0.1` | `f4b2884897e864cc28022536fd5b61d05eb2e7fffbee096154bd5bb308cc8c4a` | `6380e14b09921a5c77d3fcd8ce5ac96c4115683befa6dcbd6eb4410f5b9eb1a6` |

## Ledger actions

- Restored retired Issue history `022-042` and `061-070` without renumbering.
  The former range retains its historical supersession records; `061-070`
  are explicitly retired as superseded at commit `e8c6d733`.
- Promoted Issues `077-080` to `verified` using the accepted Feature 017
  evidence. Issue `081` remains verified.
- Consolidated Issues `081-085` delivery notes into their Completion Records
  and removed the duplicate `docs/issue#0081.html` through
  `docs/issue#0085.html` files.
- Kept Issues `005/045/047/049/051/053/055/058/060/073/076` at
  `pending_verification`; deterministic reruns do not replace their remaining
  live, browser, or independent Test Spec evidence.
- Final ledger: 117 total; 9 verified, 18
  `implemented_pending_verification`, 11 `pending_verification`, 32 planned,
  and 47 superseded. The next Issue number is `118`.

## Implementation and deterministic evidence

- Added the Issue ledger validator and fixture tests. `pnpm test:specos`
  passed 5/5 and `pnpm specos:validate` passed.
- Wrapped the Spec contract migration DDL in an explicit SeaORM transaction.
  The real interruption fixture now proves conflicting index creation leaves
  neither Spec contract table behind and does not advance the migration
  ledger. Targeted migration tests passed 5/5.
- Added Context tab keyboard focus coverage; the Context rerun passed 9/9
  without React `act` warnings.
- Rust deterministic suites passed: `specos_engine_gaps` 3/3,
  `specos_agent_team_context` 48/48, `specos_agent_team_015_016` 4/4,
  `memory_fake_gateway` 16/16, `memory_capture_outbox` 12/12, and
  `memory_recall_context` 8/8.
- Targeted Context/Tasks/Teams/i18n frontend tests passed 40/40.
- Full `pnpm test` passed 371 files and 5107 tests.
- `pnpm build` completed the Next.js static export successfully (32/32
  static pages).
- ESLint passed for the changed JavaScript/TypeScript files. Full
  `pnpm eslint .` remains blocked only by pre-existing/out-of-scope findings
  in untracked `agent/*.ts`, `scripts/find-free-port.mjs`,
  `scripts/sync-upstream.mjs`, `scripts/sync-upstream.test.mjs`, and the
  `tests/fixtures/**/mock-llm.mjs` fixture.

## Rust repository matrix

- Passed: desktop `cargo check`; server `cargo check` and Clippy; MCP
  `cargo check` and Clippy.
- Blocked: desktop test, desktop all-target Clippy, and server lib test all
  stop at the same pre-existing/out-of-scope compile error:
  `src/work_task/engine.rs:10171` initializes `WorkTaskDraft` without the
  newly required `task_kind` field. Per the approved scope, this unexpected
  debugging item was not modified.

## Final decision

The approved reconciliation and deterministic fixes are complete. The ledger
status promotions are limited to evidence-backed Issues `077-080`; all Issues
requiring live, browser, external-provider, or independent acceptance remain
pending. No QA acceptance is inferred from the deterministic reruns.

## Review closeout

- Review target: the scoped dirty-tree reconciliation diff at baseline
  `f6fefea6753c`; this repository uses the older flat `.issues/` adapter and has
  no owning child-package `review.md`, so this normalized record is the review
  checkpoint rather than a fabricated GoalSpec v2 artifact.
- `REVIEW-FLAT-001` (resolved): repository loading originally selected only
  already-valid three-digit Issue filenames and could miss a malformed
  `issue-*.md`. Loading now admits all Issue Markdown candidates and the
  validator reports malformed names; a repository-level regression test covers
  the case.
- Code review verdict: clean for the scoped ledger, migration, validator, CI,
  evidence and Context-test changes after resolving `REVIEW-FLAT-001`.
- Delivery evidence verdict: complete for the deterministic reconciliation;
  explicitly incomplete for the live/browser/external/independent gates listed
  above and for the out-of-scope Rust test fixture compile blocker.
- QA acceptance: unchanged and not inferred. No commit, push, PR or merge was
  performed.
