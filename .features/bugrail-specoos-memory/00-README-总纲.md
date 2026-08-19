# SpecOS 全量 Memory / Knowledge / Skill 系统总纲

> 文档状态：Full Product Vision（非实施授权，Not Implementation Ready）  
> 目标版本：Full Product / Daily-use Ready  
> 日期：2026-08-18  
> 上游基础：TencentCloud/TencentDB-Agent-Memory  
> 目标宿主：当前 SpecOS / Codeg-derived 多 CLI Harness + Tauri Desktop + Web Remote Control

> **2026-08-18 边界声明**：本目录是完整产品愿景（Full Product Vision），不是可实施规格。
> 唯一获得实施授权的是 `BUGRAIL-SPECOS-017` Memory Plugin MVP01
> （`.features/BUGRAIL-SPECOS-017-memory-plugin-mvp01/spec.md`，v0.2 approved）。
> Epic A–K、Memory Hub、短期 Context Offload、sidecar 生命周期管理、Wiki、CodeGraph、
> Skill Evolution、Recall Router、跨项目共享与动态插件安装均不属于 MVP01；
> 它们需要各自独立的 Feature、接口与验证，不能借 MVP01 的名义提前实施。

---

## 1. 目标

本方案不是 MVP，而是直接定义一套可长期日常使用、可继续演进的完整 Agent Memory Operating Layer。

系统完成后，应让 Codex、Claude Code、OpenCode 以及未来任意 CLI/Agent 共享以下能力：

1. **Chat Memory**：跨 Session 记住事实、偏好、决定、上下文。
2. **Short-term Context Offload**：长任务自动卸载大日志，通过摘要和 Mermaid Canvas 保持可恢复的短期任务状态。
3. **Skill Memory**：从成功任务中沉淀可复用 SOP/Skill，支持版本、审核、启停、作用域和资源。
4. **Wiki**：把项目文档、ADR、规范、外部资料构建为可浏览、可检索、可追溯的 Wiki。
5. **CodeGraph**：构建 repository 的文件、符号、引用、调用、依赖与 impact analysis。
6. **Memory Hub**：统一查看、管理、审计 Chat Memory / Skill / Wiki / CodeGraph。
7. **Agent Loadout**：为不同 Agent/角色装配不同 Memory、Skill、Wiki、CodeGraph 资产。
8. **User / Team / Agent / Task Scope**：支持个人、项目、团队、角色和任务级隔离。
9. **Recall Router**：自动决定当前问题应该优先召回 Persona、Scenario、Atom、Wiki、CodeGraph 还是 Skill。
10. **White-box Traceability**：任何高层摘要都可以下钻到底层原始证据。
11. **Local-first**：默认本机可完整运行；未来可切换远程/团队服务。
12. **Provider 插件化**：TencentDB 是默认实现，不是硬编码唯一实现。

---

## 2. 上游能力基线

TencentDB-Agent-Memory 当前的完整方向已经包含四类资产：

- Chat Memory
- Skill
- Wiki
- CodeGraph

并通过 Memory Hub、Memory Proxy、Team/Agent/Task/ACL、Loadout 形成统一资产管理和 Agent 注入体系。

长期记忆采用：

```text
L0 Conversation
  ↓
L1 Atom
  ↓
L2 Scenario
  ↓
L3 Persona
```

短期任务记忆采用：

```text
Raw Tool Results / refs
  ↓
Step Summaries / JSONL
  ↓
Mermaid Task Canvas
```

统一原则：

> 低层保留证据，高层保留结构；平时使用高层，必要时沿引用下钻到底层。

---

## 3. SpecOS 最终产品边界

我们不把 TencentDB UI 或 Proxy 原样嵌进去，而是：

```text
┌─────────────────────────────────────────────┐
│ SpecOS Desktop / Web UI                     │
│                                             │
│ Chat | Tasks | Memory Hub | Wiki | Skills  │
│ CodeGraph | Agents | Settings              │
└─────────────────────┬───────────────────────┘
                      │
              SpecOS Orchestrator
                      │
        ┌─────────────┼──────────────┐
        │             │              │
   CLI Runtime   Context Router   Asset Layer
        │             │              │
 Codex/Claude     Recall/Loadout   Provider API
                                  │
                       TencentDB Provider
                                  │
             TencentDB MemoryCore / Knowledge
```

### SpecOS 自己负责

- Desktop/Web UX
- CLI 生命周期统一
- Session/Task/Project identity
- Provider plugin boundary
- Agent configuration
- Loadout configuration
- Context routing
- token budgeting
- prompt assembly
- security policy
- permission UI
- observability UX
- upgrade/migration
- backup/restore
- user-facing QA diagnostics

### TencentDB 默认负责

- L0/L1/L2/L3 pipeline
- recall/search
- dedup/conflict processing
- context offload engine
- Skill storage/extraction backend
- Wiki / CodeGraph knowledge services
- asset metadata
- Gateway
- SDK/API
- storage primitives

---

## 4. 为什么不是直接把 Memory Proxy 放在所有 CLI 前面

完整产品中 Memory Proxy 可以保留为一种 Adapter，但不应成为 SpecOS 唯一接入方式。

SpecOS 已经拥有 CLI Runtime 和 Session UI，因此最佳长期结构是：

```text
CLI Adapter
   ↓
Unified Harness Lifecycle
   ↓
Context Router / Memory Plugin
   ↓
Provider SDK/Gateway
```

而不是：

```text
UI → CLI → vendor-specific proxy → LLM
```

原因：

- Session identity 由 SpecOS 统一掌握；
- 不同 CLI 的 hook 能力不同；
- SpecOS 还要同时注入 Skill/Wiki/CodeGraph/Agent role；
- 后续会有多 Agent 编排；
- 可以避免 provider-specific proxy 成为架构中心。

Memory Proxy 仍然可作为：
- 无法修改生命周期的外部 CLI 接入方案；
- Headless/Remote 模式；
- 第三方 Agent 快速适配方案。

---

## 5. 文档导航

### 产品

- `01-PRD-完整产品需求.md`
  - 用户场景
  - 功能模块
  - 产品行为
  - 非功能要求
  - 完整验收

### 架构

- `02-ARCH-系统架构与插件边界.md`
  - 组件
  - 进程
  - Provider
  - Lifecycle
  - IPC/API
  - 插件模型

### Memory

- `03-MEMORY-记忆模型召回与上下文.md`
  - L0-L3
  - Short-term Offload
  - Recall Router
  - token budget
  - 冲突/过期/去重
  - evidence drill-down

### Knowledge & Skill

- `04-KNOWLEDGE-SKILL-WIKI-CODEGRAPH.md`
  - Wiki
  - CodeGraph
  - Skill
  - 自动提炼
  - Knowledge registry
  - 版本化

### Agent / Scope

- `05-AGENT-LOADOUT-SCOPE-权限.md`
  - User/Team/Project/Agent/Task
  - Loadout
  - ACL
  - Agent 专家配置
  - 多模型/多 CLI

### UI

- `06-UIUX-MemoryHub完整交互.md`
  - Desktop UX
  - Memory Hub
  - Wiki
  - CodeGraph
  - Skill Manager
  - Recall inspector
  - Diagnostics

### 实现

- `07-IMPLEMENTATION-工程实现与接口.md`
  - TS/Rust 结构
  - contracts
  - config
  - lifecycle hooks
  - API mapping
  - database/storage
  - sidecar process

### QA / Ops

- `08-QA-SECURITY-OPERATIONS.md`
  - Unit/Contract/Integration/E2E
  - security
  - backup
  - recovery
  - observability
  - release gates

### 交付计划

- `09-ROADMAP-开发拆分与Definition-of-Done.md`
  - Epic
  - Task IDs
  - 依赖关系
  - 实施阶段
  - 每阶段验收
  - 完成定义

---

## 6. 最终日常使用体验

用户打开项目后：

```text
Project Open
  ↓
加载 Project Identity
  ↓
恢复 Agent Loadout
  ↓
检查 Memory/Knowledge provider
  ↓
启动/连接 Gateway
  ↓
读取 L3 Persona + active task canvas
  ↓
UI Ready
```

用户输入：

```text
“把权限模块重构一下，注意不要影响移动端。”
```

Context Router 自动处理：

```text
1. 当前 Session / Task context
2. project policy / AGENTS.md
3. L3 Persona
4. L2 architecture scenario
5. L1 “mobile still uses legacy auth” fact
6. CodeGraph impact analysis
7. Wiki ADR for auth module
8. matching refactor Skill
9. 当前用户消息
```

最后只注入 token budget 内的最相关内容。

执行过程中：

```text
大 tool output
  ↓
offload to refs
  ↓
step summary
  ↓
update task canvas
```

任务完成后：

```text
Conversation → L1/L2/L3
Successful trace → Skill candidate
Code changes → CodeGraph incremental refresh
Docs/ADR changes → Wiki incremental refresh
```

这才是最终闭环。

---

## 7. 版本策略

完整产品采用：

```text
SpecOS Provider Contract
        ↓
TencentDB Provider Adapter
        ↓
Pinned TencentDB build
```

禁止生产环境直接追 `main` / branch HEAD。

建议：

1. 选定经过内部测试的 TencentDB 2.x commit/tag；
2. 放入 lock manifest；
3. adapter contract test 固定 API；
4. 所有升级走 migration + compatibility suite；
5. provider 版本与数据 schema version 分开记录。

示例：

```yaml
providers:
  memory:
    id: tencentdb
    runtimeVersion: "2.0.0-beta.1+specos.1"
    schemaVersion: 3
    autoUpgrade: false
```

---

## 8. 完整交付标准

只有以下都具备，才叫“可以自己长期使用”：

- memory 不丢
- project 不串
- agent 不串
- task 可恢复
- CLI 切换仍能记住
- memory 能查看和修正
- skill 可审核/版本化
- wiki 可增量更新
- codegraph 可 impact analysis
- recall 可解释
- token 可控
- gateway 挂了 agent 不挂
- backup 能恢复
- provider 升级可回滚
- log/trace 能定位
- secrets 不被随意沉淀
- 全链路 QA 自动化

---

## 9. 核心设计原则

### P1 — Memory ≠ History
记忆是结构化、分层、可检索资产，不是无限聊天记录。

### P2 — Knowledge ≠ Memory
项目事实性知识进入 Wiki / CodeGraph；用户偏好、经验和决策进入 Chat Memory。

### P3 — Skill ≠ Memory Atom
Skill 是可执行能力资产，必须有触发条件、步骤、资源、验证和版本。

### P4 — Evidence First
任何高层产物必须可追溯到低层证据。

### P5 — Fail Open
Memory/Knowledge 故障不能阻止编码 Agent 正常执行。

### P6 — Explicit Scope
所有资产必须带 scope，不允许“靠 session key 猜租户”。

### P7 — Bounded Context
所有自动注入必须有 token/character budget。

### P8 — Human Controllable
用户必须能查看、禁用、删除、纠正和审计自动沉淀内容。

### P9 — Plugin First
TencentDB 是默认 provider，所有上层逻辑使用 SpecOS domain contract。

### P10 — Local First, Remote Ready
个人使用默认本地；团队使用可以平滑转 remote deployment。

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
