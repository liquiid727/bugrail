# ROADMAP — 完整开发拆分与 Definition of Done

> 这是完整产品的实施顺序，不代表砍功能。阶段只是为了降低集成风险。

> **2026-08-23 正式拆分**：下面 Epic A-K 保留为产品检查清单，不再直接作为
> 实施任务。可执行交付以 `.prd/prd-memory-operating-layer-roadmap.md` 和
> `BUGRAIL-SPECOS-028` 至 `036` 为准：A/D foundation -> `028`，B -> `029`，
> C/J Memory governance -> `030`，E -> `031`，F -> `032`，G -> `033`，
> H -> `034`，I -> `035`，剩余 J/K -> `036`。Wiki、CodeGraph、Skill 是独立
> 插件，不能合并进 Memory Issue。

# EPIC A — Foundation

## A001 Asset Domain
定义 Memory/Skill/Wiki/CodeGraph 统一 asset refs。

## A002 Scope Model
User/Workspace/Project/Team/Agent/Task/Session。

## A003 Lifecycle Bus
统一 CLI 生命周期。

## A004 Plugin Runtime
Provider 插件注册、enable/disable、health。

## A005 Job Queue
持久化后台任务。

### Gate A

- domain tests
- lifecycle integration
- plugin mock
- restart-safe jobs

---

# EPIC B — TencentDB Runtime

## B001 Pin upstream
固定 runtime build。

## B002 Sidecar supervisor

## B003 Secure config

## B004 Gateway client

## B005 SDK adapter

## B006 Version/schema check

## B007 Migration coordinator

## B008 Backup/restore

### Gate B

用户无需命令行即可：

```text
install/start/restart/backup/restore/provider
```

---

# EPIC C — Chat Memory

## C001 Capture
## C002 L0 mapping
## C003 L1-L3 pipeline status
## C004 Recall
## C005 Search
## C006 Conflict/validity overlay
## C007 Evidence drill-down
## C008 Delete/correct
## C009 Recall inspector

### Gate C

跨 Session + 跨 CLI 可稳定工作。

---

# EPIC D — Context Router

## D001 Intent classifier
## D002 Source router
## D003 Reranker
## D004 Token budget
## D005 Prompt assembler
## D006 Provenance
## D007 Context inspector

### Gate D

不同 query 能选择正确资产，且注入可解释。

---

# EPIC E — Short-term Task Memory

## E001 Tool result offload
## E002 refs
## E003 JSONL summaries
## E004 Mermaid canvas
## E005 checkpoint
## E006 Resume
## E007 task UI

### Gate E

超长 task 可以关闭后继续，不回放完整 transcript。

---

# EPIC F — Wiki

## F001 Source registry
## F002 Parser integration
## F003 Wiki builder
## F004 Search
## F005 Source citations
## F006 incremental sync
## F007 stale detection
## F008 UI

### Gate F

文档变化后 Wiki 自动更新且可追溯。

---

# EPIC G — CodeGraph

## G001 Repo source
## G002 full index
## G003 symbols
## G004 refs/calls
## G005 impact
## G006 incremental refresh
## G007 changed-file context
## G008 UI

### Gate G

Plan Agent 可以在改代码前自动完成 impact analysis。

---

# EPIC H — Skill

## H001 Skill schema
## H002 CRUD/version
## H003 trace collector
## H004 pattern detector
## H005 candidate generator
## H006 dedup/merge
## H007 validation
## H008 publish
## H009 router
## H010 rollback
## H011 UI candidate inbox

### Gate H

重复成功工作流能形成可验证、可版本化 Skill。

---

# EPIC I — Agent Loadout

## I001 Agent Profile
## I002 Loadout
## I003 Inheritance
## I004 ACL
## I005 Multi-agent task sharing
## I006 A/B isolation
## I007 UI

### Gate I

多个 Agent 可拥有不同模型 + skills + knowledge + memory scope。

---

# EPIC J — Memory Hub / Operations

## J001 Hub shell
## J002 Diagnostics
## J003 Jobs UI
## J004 Backup UI
## J005 Migration UI
## J006 Logs
## J007 Metrics
## J008 Export/import

---

# EPIC K — Hardening

## K001 Security suite
## K002 Failure injection
## K003 Migration matrix
## K004 Performance suite
## K005 Soak
## K006 Cross-platform Windows/macOS/Linux
## K007 Packaging
## K008 Update rollback

---

# 建议开发顺序

```text
A
↓
B
↓
C
↓
D
↓
E
↓
F + G
↓
H
↓
I
↓
J
↓
K
```

F/G 可以并行。

---

# 单 Task Definition of Done

任何 task 必须满足：

- [ ] code
- [ ] types/contracts
- [ ] unit tests
- [ ] error handling
- [ ] structured logs
- [ ] config
- [ ] UI state（若可见）
- [ ] docs
- [ ] no upstream DTO leakage
- [ ] no secret logging
- [ ] CI green

---

# Epic Definition of Done

除了 Task DoD：

- [ ] integration test
- [ ] E2E
- [ ] failure path
- [ ] restart path
- [ ] migration impact
- [ ] telemetry
- [ ] manual QA
- [ ] acceptance demo

---

# 完整产品 Release Checklist

## Runtime

- [ ] provider bundled/installed
- [ ] startup automatic
- [ ] upgrades controlled
- [ ] rollback tested

## Memory

- [ ] cross session
- [ ] cross CLI
- [ ] correction/delete
- [ ] provenance
- [ ] scopes

## Task

- [ ] offload
- [ ] canvas
- [ ] resume

## Skill

- [ ] discover
- [ ] validate
- [ ] version
- [ ] publish
- [ ] rollback

## Wiki

- [ ] sources
- [ ] search
- [ ] incremental
- [ ] citations

## CodeGraph

- [ ] symbols
- [ ] refs
- [ ] impact
- [ ] incremental

## Agent

- [ ] loadout
- [ ] multiple model profiles
- [ ] permissions

## Ops

- [ ] diagnostics
- [ ] backup
- [ ] restore
- [ ] migrations
- [ ] logs
- [ ] metrics

## Security

- [ ] secret redaction
- [ ] local bind
- [ ] remote auth
- [ ] ACL
- [ ] prompt injection guards

---

# 最终完成标准

完整版本不能以“接口已经接通”为完成。

最终标准是：

> 一个普通用户在全新机器安装 SpecOS 后，不需要打开终端、不需要手改数据库、不需要理解 TencentDB 内部结构，就可以开启 Memory，导入项目，使用 Codex/Claude 工作数天，跨 Session 恢复任务，查询/纠正记忆，查看 Wiki/CodeGraph，审核 Skill，切换 Agent，并在出现 provider 故障后通过 UI 恢复。

达到这一点，才叫 Done。

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
