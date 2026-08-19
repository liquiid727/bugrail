# IMPLEMENTATION — 工程实现与接口规范

# 1. 推荐代码结构

```text
apps/
  desktop/
  web/

packages/
  agent-runtime/
  lifecycle/
  context-router/
  asset-domain/
  memory/
  knowledge/
  skills/
  codegraph/
  plugin-runtime/
  provider-tencentdb/
  observability/
  config/
  jobs/

src-tauri/
  sidecars/
  processes/
  secure-store/
  filesystem/
```

---

# 2. Provider Contracts

## Memory

```ts
interface MemoryProvider {
  health(): Promise<ProviderHealth>;

  capture(input: CaptureInput): Promise<CaptureResult>;
  recall(input: MemoryRecallInput): Promise<MemoryRecallResult>;
  search(input: MemorySearchInput): Promise<MemorySearchResult>;

  get(id: string): Promise<MemoryRecord>;
  delete(id: string): Promise<void>;
  updateMetadata(input: MemoryMetadataUpdate): Promise<void>;

  pipelineStatus(scope: Scope): Promise<PipelineStatus>;
}
```

## Skill

```ts
interface SkillProvider {
  search(input: SkillSearchInput): Promise<Skill[]>;
  get(id: string, version?: string): Promise<Skill>;
  createDraft(input: CreateSkillInput): Promise<Skill>;
  validate(id: string): Promise<ValidationResult>;
  publish(id: string): Promise<Skill>;
  disable(id: string): Promise<void>;
}
```

## Wiki

```ts
interface KnowledgeProvider {
  registerSource(input: KnowledgeSourceInput): Promise<KnowledgeSource>;
  sync(sourceId: string): Promise<JobRef>;
  search(input: KnowledgeSearchInput): Promise<KnowledgeHit[]>;
  getPage(id: string): Promise<KnowledgePage>;
}
```

## CodeGraph

```ts
interface CodeGraphProvider {
  index(input: IndexRepositoryInput): Promise<JobRef>;
  searchSymbol(query: string): Promise<SymbolHit[]>;
  references(symbolId: string): Promise<GraphRef[]>;
  impact(input: ImpactInput): Promise<ImpactResult>;
}
```

---

# 3. TencentDB Adapter

```text
provider-tencentdb/
  client/
    gateway-client.ts
    knowledge-client.ts
  mappers/
    memory.ts
    skill.ts
    metadata.ts
    knowledge.ts
  runtime/
    version.ts
    migrations.ts
  plugin.ts
```

所有 upstream DTO 在 mapper 终止。

---

# 4. Gateway Connection

推荐优先官方 SDK，裸 HTTP 仅作为 fallback/test fixture。

配置：

```yaml
provider:
  id: tencentdb
  endpoint: http://127.0.0.1:8420
  authRef: secure://tencentdb-gateway
  serviceId: <instance>
```

远程模式必须 TLS。

---

# 5. Sidecar Install

不要要求用户预装 Node 环境。

Desktop distribution 应：

- bundle compatible runtime/sidecar
- or download signed runtime package
- verify checksum/signature
- unpack into app-managed version directory

结构：

```text
runtime/tencentdb/<version>/
```

---

# 6. Runtime Manifest

```json
{
  "provider": "tencentdb",
  "runtimeVersion": "pinned",
  "schemaVersion": 3,
  "binaryHash": "...",
  "installedAt": "...",
  "lastSuccessfulStart": "..."
}
```

---

# 7. Config

SpecOS-level：

```yaml
memorySystem:
  enabled: true

  provider:
    id: tencentdb
    mode: local

  capture:
    enabled: true
    redactSecrets: true

  recall:
    enabled: true
    timeoutMs: 1500

  offload:
    enabled: true

  skills:
    enabled: true
    discovery: auto-validate

  wiki:
    enabled: true

  codegraph:
    enabled: true

  backup:
    enabled: true
    schedule: daily
```

上游高级配置保留：

```yaml
providerConfig:
  recallStrategy: hybrid
  pipelineEveryNConversations: 5
  personaTriggerEveryN: 50
```

---

# 8. Secure Store

LLM / embedding / Gateway credentials 不写普通 YAML。

存：

```text
OS Keychain / Credential Manager
```

config 只写：

```text
secretRef
```

---

# 9. turn.before

```ts
async function prepareTurn(ctx: TurnContext) {
  const task = await taskContext.get(ctx.taskId);
  const bundle = await contextRouter.build({
    query: ctx.userMessage,
    task,
    agent: ctx.agent,
    project: ctx.project,
  });

  return promptAssembler.apply(ctx, bundle);
}
```

---

# 10. turn.after

```ts
async function finalizeTurn(ctx: CompletedTurn) {
  await jobQueue.enqueueUnique(
    "memory.capture",
    `${ctx.projectId}:${ctx.sessionId}:${ctx.finalMessageId}`,
    sanitizeCapture(ctx)
  );

  await offload.observe(ctx);

  if (ctx.changedFiles.length) {
    await codeGraph.enqueueIncremental(ctx.changedFiles);
  }
}
```

不要在 UI thread 里跑 extraction。

---

# 11. session.closed

```text
flush capture
finalize task canvas
persist checkpoint
enqueue skill discovery
trigger pipeline if needed
```

---

# 12. Files Changed

来源：

- Agent structured event
- git status diff
- filesystem watcher

去重后发：

```text
knowledge.files.changed
```

---

# 13. Job Queue

本地模式至少需要持久化 job table。

字段：

```text
id
type
dedup_key
status
payload_ref
attempt
max_attempts
created_at
started_at
finished_at
last_error
```

App crash 后可以恢复。

---

# 14. Retry

建议：

```text
capture      3
wiki sync    3
code index   3
skill job    2
backup       2
```

Recall 不进入 job queue，critical path 最多一次快速 fallback。

---

# 15. Caching

缓存：

- L3 Persona
- Agent loadout
- active task canvas
- recent L2
- Wiki top pages metadata
- graph symbol lookup

缓存必须 revision-aware。

---

# 16. API Validation

所有 provider response 用 schema validation：

```text
zod / valibot / equivalent
```

禁止 `as SomeType` 信任远程响应。

---

# 17. Version Compatibility

启动顺序：

```text
read manifest
↓
start provider
↓
health
↓
version
↓
schema check
↓
migration check
↓
ready
```

如果 schema mismatch：

```text
MIGRATION_REQUIRED
```

不自动 destructive migration。

---

# 18. Migration

流程：

```text
backup
dry-run
migrate
validate
start new runtime
smoke test
commit migration marker
```

失败：

```text
restore backup
start old runtime
```

---

# 19. Backup

备份 manifest：

```json
{
  "createdAt": "...",
  "provider": "tencentdb",
  "runtimeVersion": "...",
  "schemaVersion": 3,
  "projectIds": ["..."],
  "checksum": "..."
}
```

支持：

- full backup
- project export
- user export

---

# 20. IPC

Webview 不直接访问 Gateway。

```text
UI
 ↓ Tauri command / app RPC
Application Service
 ↓
Provider
```

这样 credentials 和 scope control 不暴露到 renderer。

---

# 21. Logging

结构化：

```json
{
  "event": "context.bundle.created",
  "turnId": "...",
  "memory": 4,
  "skills": 1,
  "wiki": 2,
  "codeGraph": 3,
  "tokens": 4218,
  "durationMs": 281
}
```

永远不要 INFO 打印全部 memory 内容。

---

# 22. Tracing

Trace：

```text
turn
  context-router
    memory-recall
    skill-search
    wiki-search
    code-impact
  cli-runtime
  capture-enqueue
```

Provider 支持 OTel 时传 trace context。

---

# 23. Recommended Repository Integration Order

1. lifecycle
2. asset domain
3. plugin runtime
4. tencentdb provider
5. sidecar supervisor
6. chat memory
7. context router
8. task offload
9. wiki
10. codegraph
11. skill
12. loadout
13. hub UI
14. operations

---

# 24. 完整工程验收

- [ ] provider can be mocked
- [ ] local sidecar no external manual install
- [ ] credentials secure
- [ ] jobs survive restart
- [ ] migrations reversible
- [ ] context routes traceable
- [ ] duplicate lifecycle events idempotent
- [ ] renderer never directly calls provider
- [ ] provider schemas runtime validated

---

## 上游参考与实现注意

本方案依据 2026-08-18 可见的 TencentCloud/TencentDB-Agent-Memory 项目设计整理，重点参考：

- `README.md` / `README_CN.md`
- Releases 1.x / 2.0.0-beta.1
- `MemoryCore/README.md` (`feat/server_team`)
- Codex / Claude Code / OpenCode adapter work

上游：
https://github.com/TencentCloud/TencentDB-Agent-Memory

实现时必须固定经过测试的 tag/commit，不应直接依赖持续变化的 branch HEAD。
