---
id: BUGRAIL-SPECOS-017-TEST
version: "0.2"
status: approved
sourceSpecId: BUGRAIL-SPECOS-017
sourceSpecVersion: "0.2"
sourceSpecHash: "f62824d31787ff774d962d942ffba0423fe78f28c77fa7b0e2e000d71dacfd58"
upstream: "TencentCloud/TencentDB-Agent-Memory@v2.0.0 (0aff21a2d9f2b8a0354aaa80a2e586aab4054562) + bugrail.1 patch"
---

# Test Spec: Memory Plugin MVP01

## 1. Verification Strategy

The deterministic in-memory Adapter verifies the Memory interface and all
WorkTask/Context behavior in normal test suites. A fake Gateway (recorded v3
HTTP fixture) verifies transport paths, headers, envelope/error mapping,
identity fields and the `v2.0.0+bugrail.1` patch contract. One pinned
TencentDB Agent Memory `v2.0.0+bugrail.1` integration result verifies the
async L0->L1 path end to end; release evidence must record the upstream
source commit, patch hash, image/tag/digest, exact commands and redacted
trace. Tests against a moving branch or `latest` image do not satisfy this
Spec.

The observable test surface is the Memory interface, shared command-core,
persisted delivery/package facts and rendered UI. Tests do not assert Adapter
private state.

## 2. Test Cases

| ID | Requirements | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-017.T01` | R01,R02,R07 | Validate typed configuration, strict identity resolution, static Adapter allowlist and secret redaction. Exercise missing team/agent/user/session, missing env, invalid URL/bounds, credential-bearing URLs, non-loopback HTTP and legacy no-Memory config. | No network call on invalid identity; config response/log/database contains references only; legacy behavior is unchanged and makes zero Memory network calls. |
| `BUGRAIL-SPECOS-017.T02` | R01,R02 | Exercise health over `GET /health` and v3 transport with success, 401, 429/quota, 5xx, timeout, malformed JSON, non-zero business envelope and unsupported/vanilla version reporting. | Stable error class, retryability and trace ID; exact pin recognized through `/health`; vanilla `v2.0.0` classified `memory.upstreamUnsupported` for capture; no response body or credential leakage. |
| `BUGRAIL-SPECOS-017.T03` | R03,R07 | Settle a run containing user/assistant/system/tool/terminal/attachment and secret-like content, then restart/retry delivery. Replay the same batch with identical message IDs against the patched contract; attempt the same against a vanilla `v2.0.0` fixture. | Only eligible bounded text reaches the fake/pinned Adapter; one delivery row per `(provider_id, task_id, run_seq)` with stable accepted message IDs survives restart; replay never increases L0 under the patch contract; vanilla behavior is detected as unsupported; delivered rows retain hash/IDs only, not payload body. |
| `BUGRAIL-SPECOS-017.T04` | R04,R05 | Recall ordered L1 hits and optional L3 Core with duplicates, oversized items, empty results and exact budget edges. | One immutable package records included/excluded reasons, query hash, remote ID/layer/score/content hash, fixed ordering and deterministic package hash. |
| `BUGRAIL-SPECOS-017.T05` | R04 | Fail recall for required and optional Providers before ACP prompt dispatch; fail capture after a settled run. | Required launch is blocked with no ACP dispatch; optional package/activity is degraded; capture failure leaves task/gate outcome unchanged. |
| `BUGRAIL-SPECOS-017.T06` | R03,R05,R07 | Probe redirects, credential-bearing URL, TLS bypass request, oversized payloads, log/frontend serialization and malicious remote text. | Unsafe config is rejected; redirects are never followed across schemes; output remains text-safe and bounded; remote text is wrapped as untrusted data; secrets/excluded content are absent from all local or client-visible facts. |
| `BUGRAIL-SPECOS-017.T07` | R06 | Exercise provider test, delivery list/retry and recall preview through command-core, Tauri and Axum; render every required Context UI state. | Transport payloads are equivalent; last-good state, keyboard/focus and narrow layout behavior are recorded for all ten locale catalogs. |
| `BUGRAIL-SPECOS-017.T08` | R01-R07 | Run capture then recall against pinned TencentDB `v2.0.0+bugrail.1`, restarting BugRail between phases, and inspect the later WorkTask package. Poll L1 every 2 seconds for at most 120 seconds to absorb upstream asynchronous extraction. | Upstream contains exactly one captured message set; the later package contains matching memory provenance after restart; two project `team_id` values stay isolated; source commit, patch hash, image digest, commands, exit codes and redacted request/trace evidence are retained. |

## 3. Required Evidence

- Rust unit/contract tests for config, identity, transport, filtering, bounds,
  retry classification, normalization, ordering and provenance.
- SeaORM migration/repository tests for delivery uniqueness
  `(provider_id, task_id, run_seq)`, transitions, payload clearing and restart
  recovery.
- Patch contract tests proving replayed message IDs never increase L0 and that
  vanilla `v2.0.0` is detected as unable to support reliable capture.
- WorkTask/Context integration tests proving settlement hook placement and
  pre-ACP ordering.
- Shared command-core plus Tauri/Axum parity tests.
- React tests for provider, delivery, preview and package states in all ten
  locale catalogs.
- One pinned upstream `v2.0.0+bugrail.1` integration result with source
  commit, patch hash, endpoint/image digest, exact commands, exit codes and
  redacted request/trace evidence.
- Existing `pnpm test`, build/lint and desktop/server/`codeg-mcp` Rust checks
  required by the repository handoff at the implementation commit.

## 4. Release Exclusions

The following cannot be used as proof of acceptance:

- a successful call to `/v3/tools/list` (wrong service and method) or to
  `/health` without capture and recall evidence;
- MemoryProxy prompt injection or MemoryPanel screenshots;
- tests against upstream default branch or an unpinned `latest` image;
- capture claimed idempotent without the replay/no-duplicate-L0 proof;
- frontend-only state, transient event delivery or an Agent assertion that
  memory was stored;
- payload/log snapshots containing real credentials or private transcript text.
