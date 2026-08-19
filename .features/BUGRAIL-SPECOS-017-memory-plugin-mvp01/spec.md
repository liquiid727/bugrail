---
id: BUGRAIL-SPECOS-017
version: "0.2"
title: "Memory Plugin MVP01"
status: approved
changeType: external-provider-integration
prd: ".prd/prd-memory-plugin-mvp01.md"
design: "design/adr/ADR-004-memory-plugin-tencentdb-mvp01.md"
clientDesign: "design/specos-client-interaction-design.md"
codeBaseline: "94d9d3b2"
upstream: "TencentCloud/TencentDB-Agent-Memory@v2.0.0 (0aff21a2d9f2b8a0354aaa80a2e586aab4054562) + bugrail.1 patch"
dependsOn: [BUGRAIL-SPECOS-003, BUGRAIL-SPECOS-006, BUGRAIL-SPECOS-007, BUGRAIL-SPECOS-008, BUGRAIL-SPECOS-009]
---

# BUGRAIL-SPECOS-017: Memory Plugin MVP01

## 1. Summary

Add a deep Memory module to the existing Context and WorkTask paths. TencentDB
Agent Memory v3 is the first production Adapter. BugRail owns capture policy,
identity mapping, delivery evidence, recall selection, immutable Context
Package injection and UI; the Adapter owns vendor transport and L0-L3 calls.

The full product vision (`Chat Memory`, short-term Offload, Skill, Wiki,
CodeGraph, Memory Hub, Recall Router and related capabilities) is documented in
`.features/bugrail-specoos-memory/` and is NOT this Feature. MVP01 implements
only the WorkTask capture/recall slice defined below.

### Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-017.R01` | One Memory interface exposes health, capture and recall with vendor-neutral requests/results and stable error classes. |
| `BUGRAIL-SPECOS-017.R02` | Typed project configuration resolves endpoint, service/secret references and strict team/agent/user/session/task identity without a shared fallback. |
| `BUGRAIL-SPECOS-017.R03` | Settled WorkTask runs enqueue filtered, bounded and run-unique user/assistant L0 messages with durable delivery state. |
| `BUGRAIL-SPECOS-017.R04` | Run preparation recalls bounded L1 and optional L3 memory under one deadline, then delegates deduplication, budget and required/optional behavior to existing Context compilation. |
| `BUGRAIL-SPECOS-017.R05` | Every recall candidate records included/excluded status with provider, upstream version, layer, remote ID, score when present, query hash, content hash and selection reason. |
| `BUGRAIL-SPECOS-017.R06` | Context UI exposes configuration health, capture delivery and recall provenance through equivalent Tauri/Axum command-core behavior. |
| `BUGRAIL-SPECOS-017.R07` | Credentials and excluded transcript content never enter persisted config, packages, activity, logs or frontend payloads. |

PRD coverage: `P-MEM-01` through `P-MEM-09`.

## 2. Module Placement And Interface

New implementation lives under `src-tauri/src/memory/`. Its external interface
is the test surface:

```text
health(MemoryProviderRef) -> MemoryHealth
capture(MemoryCaptureBatch) -> MemoryCaptureReceipt
recall(MemoryRecallRequest) -> MemoryRecallResult
```

`TencentDbMemoryAdapter` implements this interface over v3 HTTP. A deterministic
in-memory Adapter remains internal to contract and command-core tests. Adapter
selection is a static allowlist keyed by `adapter`; MVP01 does not load dynamic
code or expose upstream DTOs to callers.

Existing placement:

| Existing module | Extension |
|---|---|
| `specos_control` | Parse and validate typed Memory Provider settings in `.codeg/context.yaml`. |
| `context` | Call recall, normalize candidates and apply existing health, budget, dedup, package and provenance rules. |
| `work_task` | Enqueue capture after durable run settlement; never let capture rewrite WorkTask status. |
| `commands/specos_control` | Add shared Memory health/delivery/preview/retry command-core functions. |
| Context UI | Extend Provider, Activity and package inspection; do not embed MemoryPanel. |

Wiki, CodeGraph and Skill Evolution are not methods on this interface.

## 3. Provider Configuration And Identity

`ContextProviderConfig` gains optional `adapter` and typed Memory settings:

```text
service_id_env, team_id, user_id_env, default_agent_id,
agent_id_map, capture_enabled, recall_enabled, recall_limit,
include_core, timeout_ms, max_capture_message_bytes,
max_capture_batch_bytes
```

Validation rules:

1. `kind=memory` requires `adapter=tencentdb-agent-memory-v3`, an HTTP(S)
   endpoint, named environment references, `team_id` and a resolvable Agent ID.
2. Secret/service/user values are resolved only in backend runtime and are
   redacted from returned configuration and errors.
3. Agent resolution uses exact AgentProfile mapping (`agent_id_map`), then
   `default_agent_id`. Missing team/user/agent/session identity fails with
   `memory.identityMissing` and never issues a request.
4. Session ID is deterministic from project binding + task/run. Message ID is
   deterministic from project binding + persisted conversation/message ID.
   Capture carries a stable upstream `task_id` derived from the WorkTask
   identity; recall does NOT filter by `task_id`, so recall spans WorkTasks
   inside one project `team_id`.
5. Provider capabilities are `memory.capture`, `memory.recall.l1` and optional
   `memory.recall.l3`; unrelated capabilities are rejected for this Adapter.
6. `team_id` must be a project-specific upstream isolation space; local folder
   database IDs alone are never used as cross-install identity.

## 4. TencentDB v3 Transport Contract

- Pin compatibility to upstream tag `v2.0.0`, commit
  `0aff21a2d9f2b8a0354aaa80a2e586aab4054562`.
- Send `Authorization: Bearer` and `x-tdai-service-id`; JSON body carries
  `team_id`, `agent_id`, `user_id`, `session_id` when required and `task_id`.
- Health uses the public MemoryCore endpoint `GET /health`.
  `GET /v3/tools/list` is NOT a health probe: it belongs to the Knowledge
  service and is a POST there.
- Capture uses `POST /v3/conversation/add`.
- Recall uses `POST /v3/atomic/search`; `POST /v3/core/read` runs only when
  `include_core=true`.
- HTTP non-success, non-JSON, timeout and response `code != 0` are errors.
  Request/trace IDs may be retained; response bodies and credentials may not.
- Default timeout is 5 seconds and is bounded to `500..30000` ms. Recall limit
  defaults to 5 and is bounded to `1..20`.

The Adapter classifies errors as configuration, unauthorized, unavailable,
timeout, rate-limited, invalid-response, unsupported-version and upstream.
Retryability is explicit.

### 4.1 Upstream Patch Contract

Vanilla upstream `v2.0.0` types allow a caller-supplied message `id`, but the
Gateway treats it as read-only and regenerates a random ID inside
`/v3/conversation/add`. At-least-once retry would therefore duplicate L0
messages. MVP01 pins a minimal patch, versioned `v2.0.0+bugrail.1`, that makes
`conversation/add` accept the caller ID and upsert by it: a replayed request
with the same message IDs returns the same accepted IDs and creates no
additional L0 rows.

- The patch version is reported through `/health` and the build manifest.
- A Provider whose Gateway does not report the exact patched version does not
  enter writable state; capture stays disabled with a safe
  `memory.upstreamUnsupported` error while recall may still run.
- Contract tests prove both directions: replay with the same message IDs never
  increases L0 under the patched contract, and vanilla `v2.0.0` is detected as
  unable to support reliable capture.

## 5. Capture Lifecycle And Persistence

Eligible source is a settled WorkTask generation whose outcome is `review` or
`failed`, with a durable conversation and complete final assistant text.
Cancelled runs and merge-only generations are skipped. Only non-empty `user`
and `assistant` text is captured. System prompts, Context Packages, tool
inputs/results, terminal bytes, attachments are excluded; a message matching a
secret rule is excluded whole, before hashing or enqueue.

`memory_capture_delivery` stores:

```text
id, provider_id, folder_id, task_id, run_seq, conversation_id,
source_message_ids, payload_hash, status, attempts, retryable,
upstream_accepted_ids, safe_error_code, safe_error_message,
created_at, updated_at, delivered_at
```

The unique key is `(provider_id, task_id, run_seq)`: one delivery per provider
per settled run generation. `payload_hash` is an integrity check over the
staged filtered payload; it is never a cross-task dedup key.

The outbox stages the filtered send payload. After successful delivery the row
clears the payload body and retains only the hash, source IDs, upstream
accepted IDs and safe error fields. State transitions are
`queued -> sending -> delivered | failed`; a retryable failure returns to
`queued` through explicit retry or restart recovery. The worker:

- runs independently from the settlement transaction (capture enqueue survives
  settle, and settle never waits on the network);
- retries automatically at most 5 times with exponential backoff;
- recovers `sending` rows on startup;
- reconciles settled runs missing a delivery row, covering the crash window
  after settlement.

Stable upstream message IDs plus the patch upsert protect at-least-once
retries. Capture failure records Context Activity and never changes a settled
WorkTask or gate outcome.

Default caps are 100 messages, 8 KiB per message and 256 KiB per batch. A
message over its cap is excluded with an explicit reason, never silently
truncated.

## 6. Recall And Context Package

`context::prepare_run` asks selected Memory Providers for recall before package
persistence. The request contains bounded task title/goal text only (never the
compiled prompt or repository contents), resolved identity, task/run IDs, limit
and remaining budget. The Adapter cannot receive the whole compiled prompt or
append directly to it.

L1 and optional L3 requests run in parallel under one unified recall deadline:
target budget 1.5 seconds, absolute bound 5 seconds. An empty result is a
successful recall with zero candidates; the production path never polls the
upstream asynchronous extraction pipeline.

Normalized candidates use `kind=memory` and carry:

```json
{
  "provider": "project-memory",
  "adapter": "tencentdb-agent-memory-v3",
  "upstreamVersion": "v2.0.0+bugrail.1",
  "layer": "L1",
  "remoteId": "...",
  "score": 0.0,
  "queryHash": "sha256:...",
  "capturedAt": "...",
  "selectionReason": "memory.l1.semanticMatch"
}
```

Package order is fixed: local required items, local optional items in
configuration order, then L3 items, then L1 items ordered by score descending
with remote ID as tie-break. Local content wins deduplication against remote
content with the same content hash. Existing item, byte and token limits remain
authoritative. Required failure returns before ACP prompt dispatch; optional
failure persists degradation.

Every candidate records evidence: included/excluded status, reason, layer,
remote ID, score when present, query hash and content hash. Only included
content enters the immutable package and prompt. Remote text is wrapped as
untrusted data, never as instructions.

## 7. Commands And UI

Shared command-core functions and matching Tauri/Axum calls:

```text
specos_memory_provider_test(folder_id, provider_id) -> MemoryHealth
specos_memory_delivery_list(folder_id, cursor) -> MemoryDeliveryPage
specos_memory_delivery_retry(delivery_id) -> MemoryDelivery
specos_memory_recall_preview(folder_id, provider_id, query) -> MemoryRecallPreview
```

Provider test first calls the public `GET /health` to verify the exact pinned
version, then validates credentials and isolation through `POST /v3/core/read`
whose response body is discarded. The client only receives version, latency,
status, error class and trace ID.

The Context Overview becomes batch-loaded and returns package summaries only;
package detail is fetched on demand through a `package_get` command. This
removes the N+1 package loading and large-body list responses.

UI extends the existing Context page; it does not build a Memory Hub. It adds
typed identity/config fields, connection test, capture/recall toggles, delivery
states with retry, recall preview and package provenance. It retains last-good
persisted data and covers no-workspace, unconfigured, loading, healthy, empty,
degraded, unauthorized, timeout, invalid-response, budget-excluded, retrying
and transport error states across all ten locale catalogs.

## 8. Error Contract

| Error key | Condition |
|---|---|
| `memory.configInvalid` | Adapter, endpoint, bounds or environment references are invalid. |
| `memory.identityMissing` | team/agent/user/session identity cannot be resolved. |
| `memory.unauthorized` | Gateway rejects credentials or service identity. |
| `memory.unavailable` | Connection failure or upstream 5xx. |
| `memory.timeout` | Bounded request deadline expires. |
| `memory.rateLimited` | Upstream rate or quota limit. |
| `memory.invalidResponse` | Non-JSON, malformed envelope or unsupported fields. |
| `memory.upstreamUnsupported` | Gateway version is not the exact patched pin; capture cannot be writable. |
| `memory.deliveryNotRetryable` | Retry requested for terminal failure/delivered row. |

Errors expose safe reason, provider ID, retryability and trace ID only.

## 9. Security And Compatibility

- Remote endpoints must use HTTPS; plain HTTP is allowed only for loopback
  hosts. URLs containing credentials are rejected, as are redirects to
  disallowed schemes and any TLS-verification bypass. Oversized upstream
  responses are aborted at their bound.
- Logs contain IDs, counts, latency and error classes, not payloads or secrets.
- Legacy projects without a Memory Provider behave exactly as before and do not
  create delivery rows or make network calls.
- Desktop and server use the same Rust Adapter and command-core logic.
- `codeg`, `CODEG_*`, ACP and existing Context command names remain unchanged.
- Disabling capture/recall is prospective; remote deletion and ACL management
  are outside this Feature.

## 10. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-017.AC01` | Valid v3 configuration resolves strict identities and health through `GET /health` without exposing runtime values; incomplete identity makes no request. |
| `BUGRAIL-SPECOS-017.AC02` | Eligible run text is filtered, capped, delivered once per `(provider_id, task_id, run_seq)` and not duplicated across retry/restart under the patched upsert contract. |
| `BUGRAIL-SPECOS-017.AC03` | Tool/system/terminal/attachment and secret-bearing content is absent from payload and local evidence; delivered rows drop the payload body after success. |
| `BUGRAIL-SPECOS-017.AC04` | Matching L1 and optional L3 results enter the exact run package in the fixed order only through existing budget/dedup policy with full safe provenance. |
| `BUGRAIL-SPECOS-017.AC05` | Required recall failure blocks before ACP; optional failure degrades explicitly; capture failure never changes WorkTask outcome. |
| `BUGRAIL-SPECOS-017.AC06` | Timeout, 401, rate/quota, business error, malformed response, unsupported version and empty result retain distinct observable behavior. |
| `BUGRAIL-SPECOS-017.AC07` | Context Provider, delivery, preview and package UI states are equivalent across Tauri/Axum and survive restart. |
| `BUGRAIL-SPECOS-017.AC08` | No-Memory projects and existing WorkTask/Context/ACP behavior remain compatible and make zero Memory network calls. |

## 11. Implementation Order

1. Typed config, identity resolver, Memory interface and deterministic test
   Adapter.
2. TencentDB v3 transport and pinned contract fixtures, including the
   `v2.0.0+bugrail.1` patch contract.
3. Capture delivery migration/repository/worker and run-settlement hook.
4. Recall normalization inside Context compilation and immutable provenance.
5. Tauri/Axum commands, Context UI and package inspection.
6. Exact-version Test Spec evidence against fake Adapter and a pinned upstream
   `v2.0.0+bugrail.1` integration fixture.
