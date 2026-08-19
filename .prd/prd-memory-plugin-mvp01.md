# PRD: Memory Plugin MVP01 With TencentDB Agent Memory

## Meta

- Status: approved implementation baseline
- Date: 2026-08-18
- Approved: 2026-08-18 (frozen for BUGRAIL-SPECOS-017 implementation)
- Product: Code: BugRail / CodeG
- Target Feature: `BUGRAIL-SPECOS-017`
- Parent PRD: `.prd/prd-specos-agent-team-context-system.md`
- Decision: `design/adr/ADR-004-memory-plugin-tencentdb-mvp01.md`
- Upstream pin: TencentDB Agent Memory `v2.0.0`

## 1. Product Decision

BugRail will add a Memory Plugin module and use TencentDB Agent Memory as the
first Memory Engine. BugRail collects eligible conversation evidence, writes
it to the Engine, retrieves relevant memory, places bounded results into the
existing immutable Context Package and exposes operational state in BugRail UI.

The Memory Engine does not own Agent execution, WorkTask state, prompt
composition or BugRail UI. Wiki, CodeGraph and Skill Evolution remain separate
modules even when the first deployment supplies them from the same upstream
project.

## 2. MVP01 Outcome

A developer can configure one TencentDB Agent Memory instance for a project,
run a WorkTask, see that eligible conversation text was delivered, and start a
later run whose Context Package contains relevant L1/L3 memory with exact
provider, query, remote ID and content-hash provenance.

The slice is successful when this path works in both desktop and server mode:

```text
settled WorkTask run
  -> filter and cap user/assistant text
  -> durable capture delivery
  -> TencentDB L0 conversation/add
  -> upstream asynchronous extraction
  -> later WorkTask prepares Context
  -> TencentDB L1 atomic/search + optional L3 core/read
  -> local budget/dedup/provenance
  -> immutable Context Package
  -> existing ACP prompt dispatch
```

## 3. Product Requirements

| ID | Requirement |
|---|---|
| `P-MEM-01` | Memory is a replaceable module behind `health`, `capture` and `recall`; TencentDB v3 is the first production Adapter. |
| `P-MEM-02` | Configuration binds one project to explicit upstream `teamId`, `userId`, default Agent mapping, Gateway endpoint, service ID reference and secret reference. Missing isolation never falls back to a shared bucket. |
| `P-MEM-03` | A settled run queues only bounded user/assistant text. System prompts, secrets, terminal output, raw tool payloads and binary attachments are excluded. |
| `P-MEM-04` | Capture uses stable session/message IDs and a durable delivery record so retry/restart is idempotent and inspectable. Capture failure does not alter the WorkTask result. |
| `P-MEM-05` | Before prompt dispatch, enabled recall queries L1 by the task goal and may read bounded L3 Core memory. Vendor responses are normalized before entering Context policy. |
| `P-MEM-06` | Remote results obey existing item/byte/token budgets, deduplication and required/optional failure rules. Included content is persisted in the immutable run package with query hash, remote ID/layer/score and content hash. |
| `P-MEM-07` | BugRail UI can configure/test the Adapter, show capture deliveries and show recall inclusion/exclusion/provenance without embedding MemoryPanel. |
| `P-MEM-08` | Tauri and Axum expose the same command-core behavior; credentials remain environment/keychain references and are absent from config responses, packages, activity and logs. |
| `P-MEM-09` | The Adapter is pinned to upstream `v2.0.0`; upgrade requires named-version contract evidence and a rollback pin. |

## 4. Configuration And Identity

The existing `.codeg/context.yaml` Provider entry is extended with typed Memory
settings. Secret values are never written to the file.

```yaml
providers:
  - id: project-memory
    kind: memory
    adapter: tencentdb-agent-memory-v3
    endpoint: http://127.0.0.1:8420
    secretEnv: TENCENTDB_AGENT_MEMORY_API_KEY
    serviceIdEnv: TENCENTDB_AGENT_MEMORY_SERVICE_ID
    teamId: team-example
    userIdEnv: TENCENTDB_AGENT_MEMORY_USER_ID
    defaultAgentId: agt-default
    captureEnabled: true
    recallEnabled: true
    recallLimit: 5
    timeoutMs: 5000
    required: false
```

An optional AgentProfile-to-upstream-Agent map may override `defaultAgentId`.
`teamId`, `agentId` and `userId` must all resolve before capture or recall.
BugRail derives stable session, task and message IDs from a project binding ID,
WorkTask generation and persisted conversation/message identities; local folder
database IDs alone are not cross-install identities.

## 5. Capture Contract

- Capture begins only after local run/conversation data is durable.
- Allowed roles are `user` and `assistant`; empty and duplicate messages are
  removed. Per-message and per-batch byte caps are applied before enqueue.
- A SQLite delivery row records provider, task/run, source IDs, payload hash,
  status, attempts, last safe error, upstream accepted IDs and timestamps.
- Delivery is at-least-once. Stable upstream message IDs make restart/retry
  idempotent. Terminal provider errors require explicit retry after config is
  corrected.
- Disable stops new capture. It does not delete upstream memory.

## 6. Recall And Context Contract

- The query is derived from bounded task title/goal text, not the full prompt or
  repository contents. Its plaintext is not stored in activity; only its hash
  and safe preview are retained.
- MVP01 calls L1 `atomic/search`. L3 `core/read` is optional and separately
  budgeted. L0 history search, L2 authoring and memory mutation are deferred.
- Every result becomes a normal Context candidate. The Context module decides
  inclusion, order, deduplication and truncation; the Adapter cannot append
  directly to a system prompt.
- Required Adapter failure blocks launch before ACP dispatch. Optional failure
  persists an explicit degraded package/activity state and launch continues.

## 7. UX

The existing Context page gains Memory-specific Provider fields and status:

- connection test with upstream version/latency and safe error details;
- capture/recall enablement, identity completeness and current pin;
- recent queued/delivered/failed capture deliveries with retry action;
- recent recall count, duration and package link;
- package detail labels remote layer, score, source ID, query hash and reason.

The page retains last-good data on transport failure and covers no workspace,
unconfigured, loading, healthy, degraded, unauthorized, timeout, empty recall,
partial budget, delivery retry and transport-error states.

## 8. Delivery Boundary

MVP01 does not include:

- routing ACP/CLI traffic through MemoryProxy;
- embedding or cloning MemoryPanel;
- automatically creating upstream Team/User/Agent metadata;
- arbitrary Memory CRUD, L2 scenario editing or upstream ACL administration;
- Wiki, CodeGraph or Skill Evolution integration;
- cross-project/global sharing or automatic memory-to-Skill promotion;
- dynamic binary/plugin installation.

These capabilities require separate Features and interfaces. Wiki, CodeGraph
and Skills may share upstream identity and the ContextItem envelope but remain
independent modules.

## 9. Release Gates

- A run captured before restart appears once upstream after recovery/retry.
- A later run includes matching L1 memory in one immutable Context Package and
  exposes exact provenance after restart.
- Missing identity, 401, business-envelope error, timeout, malformed response,
  duplicate message, optional degradation and required blocking are verified.
- No secret or excluded transcript content appears in SQLite evidence, logs,
  frontend payloads or Context Packages.
- Existing WorkTask, Context, ACP, Tauri/Axum and legacy no-Memory behavior
  remains green.

