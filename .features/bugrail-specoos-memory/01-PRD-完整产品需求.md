# PRD — SpecOS Full Memory Operating Layer

## 1. 产品定位

SpecOS Memory Layer 是面向 coding agent / multi-agent harness 的“长期经验与知识操作层”。

它不是一个单独的聊天记忆插件，而是把：

```text
Chat Memory
Short-term Task Memory
Skill
Wiki
CodeGraph
Agent Loadout
Asset Scope
```

统一成可管理、可召回、可装配、可审计的资产系统。

---

# 2. 核心产品目标

## G1 跨 Session 连续性

同一项目的新 Session 可以直接理解：

- 项目长期约束
- 历史架构决定
- 用户偏好
- 已解决问题
- 未完成任务
- 当前工作方式

## G2 长任务不因 context window 退化

工具日志、搜索结果、测试输出等自动卸载，当前上下文仅保留结构化任务画布和必要摘要。

## G3 从“记住”提升到“会做”

成功完成的重复任务可以沉淀为可管理 Skill。

## G4 Agent 在改代码前拥有项目知识

Wiki 提供语义/文档知识，CodeGraph 提供源码结构与影响关系。

## G5 不同 Agent 获得不同知识装配

例如：

```text
plan-agent:
  Persona + architecture Wiki + ADR + planning Skills

implementation-agent:
  CodeGraph + coding rules + implementation Skills

review-agent:
  relevant diff + review Skills + historical defects

testing-agent:
  test Wiki + CodeGraph + testing Skills
```

---

# 3. 用户类型

## 3.1 Solo Developer

单人开发多个项目，希望 Agent 长期记住自己和项目。

## 3.2 Power User

同时使用 Codex、Claude Code、OpenCode、多个模型，希望跨 CLI 共享 Memory。

## 3.3 Multi-Agent User

配置 plan / implementation / review / test 等不同专家 Agent。

## 3.4 Team

后续使用团队资产共享、权限和 Agent Loadout。

---

# 4. 产品模块

## F01 Chat Memory

### 功能

- 自动捕获对话
- L0 原始记录
- L1 原子事实
- L2 场景/主题
- L3 Persona
- 搜索
- 过滤
- 查看 evidence
- 删除
- 禁用
- 纠正
- 过期
- 冲突处理
- project / user / agent scope

### 用户必须能看到

每条 Memory：

```text
content
layer
scope
confidence
status
created_at
updated_at
source_session
source_messages
supersedes
conflicts
last_recalled_at
recall_count
```

上游没有的字段由 SpecOS metadata overlay 保存。

---

## F02 Short-term Task Memory / Context Offload

### 目标

解决长 session 中：

- shell logs
- compiler logs
- search results
- test output
- tool output
- large file snippets

占用大量 context。

### 产物

```text
Raw Artifact
  refs/<id>.md

Step Summary
  task-events.jsonl

Task Canvas
  active-task.mmd
```

### 用户体验

Task 面板显示：

- 当前节点
- 已完成节点
- 阻塞节点
- evidence links
- context usage
- offloaded token estimate

### 恢复

新 Session 可以：

```text
Resume task
```

载入：

1. Canvas
2. active summaries
3. 必要 refs
4. project memory

而不是重新塞完整历史。

---

## F03 Memory Recall

支持三类 recall：

### Auto Recall
系统自动在 turn.before 召回。

### Agent Search
模型通过 MCP/tool 主动搜索：

```text
memory.search
conversation.search
wiki.search
code.search
skill.search
```

### User Search
Memory Hub 手工搜索。

---

## F04 Recall Inspector

每一轮 Agent 回答旁边允许打开：

```text
Context Used
```

显示：

```text
System Rules
Project Rules
Persona
Scenarios
Facts
Skills
Wiki
CodeGraph
Task Canvas
```

每项显示：

- 来源
- token 数
- score
- 为什么命中
- 是否自动/主动
- evidence

这对记忆系统日常可用非常关键。

---

## F05 Wiki

### 数据来源

- README
- docs/**
- ADR
- specs
- PRD
- API docs
- selected external docs
- user-created notes

### 功能

- source registration
- incremental sync
- document parsing
- page generation
- page linking
- semantic search
- full-text search
- citation/source mapping
- freshness status
- rebuild
- exclude patterns

### 典型页面

```text
Architecture Overview
Auth Module
Database Schema
Release Process
Coding Conventions
API Contracts
```

---

## F06 CodeGraph

### 索引内容

- repository
- modules
- files
- symbols
- classes
- functions
- interfaces
- imports
- calls
- references
- inheritance
- API boundaries
- tests
- changed files

### 用户能力

- Search Symbol
- Find References
- Dependency Path
- Impact Analysis
- “What uses this?”
- “What breaks if this changes?”
- changed-file neighborhood

### 自动行为

代码提交/工作树变化后：

```text
incremental refresh
```

不要求每次全量重建。

---

## F07 Skill Memory

Skill 是一等资产。

### Skill Schema

```yaml
id:
name:
description:
scope:
status:
version:
trigger:
when_not_to_use:
inputs:
steps:
resources:
validation:
failure_recovery:
source_traces:
created_by:
updated_by:
```

### 生命周期

```text
trace
  ↓
candidate
  ↓
dedup
  ↓
draft
  ↓
validation
  ↓
approved
  ↓
enabled
  ↓
version update / deprecated
```

### 自动提炼

默认：

```text
autoDiscover = true
autoPublish = false
```

系统可以自动发现 candidate，但正式启用需要通过 QA/用户规则。

---

## F08 Agent Loadout

每个 Agent 有自己的资产装配：

```yaml
agent: implementation-agent

memory:
  persona: true
  projectScenarios: true
  userPreferences: true

skills:
  - code-edit
  - db-migration

knowledge:
  wiki:
    - architecture
    - coding-guides
  codegraph:
    mode: full

recall:
  budget: 5000
  priorities:
    - rules
    - task
    - codegraph
    - skill
    - memory
    - wiki
```

用户可以建立 preset。

---

## F09 Asset Hub

统一页：

```text
Memory Hub
 ├── Chat Memory
 ├── Tasks
 ├── Skills
 ├── Wiki
 ├── CodeGraph
 ├── Sources
 ├── Agents
 └── Diagnostics
```

---

## F10 Scope & Ownership

完整支持：

```text
User
Workspace
Project
Team
Agent
Task
Session
```

资产 scope：

```text
private
project
team
restricted
agent
```

MVP 可以弱化，但完整产品必须有显式 scope model。

---

# 5. 使用流程

## 5.1 首次打开项目

用户选择：

```text
Enable Agent Memory
```

Wizard：

1. Project identity
2. Memory provider
3. LLM for extraction
4. Embedding provider optional
5. Wiki sources
6. CodeGraph enable
7. Auto Skill discovery
8. privacy rules
9. backup location

完成后后台初始化。

---

## 5.2 日常编码

用户：

> 把订单退款改成异步处理。

系统自动：

1. recall project decisions
2. recall related previous incidents
3. query Wiki architecture
4. CodeGraph impact analysis
5. match async-job Skill
6. assemble bounded context
7. execute selected CLI

---

## 5.3 Session 结束

系统：

1. capture final conversation
2. trigger extraction when threshold met
3. finalize task canvas
4. generate Skill candidate if qualified
5. enqueue Wiki refresh for modified docs
6. enqueue CodeGraph incremental refresh
7. persist task resume checkpoint

---

# 6. 功能级验收

## Chat Memory

- [ ] 新 Session 能召回旧 Session 决定
- [ ] 错误记忆可删除/纠正
- [ ] Project A 不污染 Project B
- [ ] Persona 可检查来源
- [ ] 召回 token 有上限

## Short-term Offload

- [ ] 10MB tool output 不进入主上下文
- [ ] Agent 可通过 ref 下钻
- [ ] Task Canvas 能恢复当前状态
- [ ] 新 Session 可 Resume

## Skill

- [ ] 自动发现候选
- [ ] Skill 有 version
- [ ] Skill 可 disable
- [ ] Skill 触发可解释
- [ ] Skill 有 source trace
- [ ] 失败 Skill 可回滚到旧版

## Wiki

- [ ] source import
- [ ] incremental update
- [ ] search
- [ ] source citation
- [ ] stale 状态
- [ ] rebuild

## CodeGraph

- [ ] repo build
- [ ] symbol search
- [ ] references
- [ ] impact analysis
- [ ] incremental refresh
- [ ] changed-file context

## Loadout

- [ ] Agent 可独立配置
- [ ] loadout 可 preset
- [ ] runtime 明确记录注入了哪些资产

---

# 7. 非功能指标

## Reliability

- Memory service failure 不得导致 Agent 无法运行。
- 所有写操作幂等。
- crash recovery 不导致重复 capture。

## Performance

- local recall p95 < 1.5 s
- UI search p95 < 1.5 s
- project opening 不因 Wiki/CodeGraph rebuild 阻塞
- background extraction 不阻塞 chat turn

## Context Efficiency

任何 turn 的自动注入必须可预算。

建议：

```text
default global context asset budget = model context × 12%
```

再按模块分配。

## Privacy

- 默认 local-first
- cloud embedding/LLM 明确提示
- secret redaction
- asset delete
- project purge
- export
- backup

---

# 8. 完整产品验收场景

## A. 跨 CLI

Session A 使用 Claude Code：

> UI 必须使用 shadcn，不要引入 MUI。

Session B 使用 Codex：

> 做一个设置页。

必须召回这个决定。

## B. Resume Long Task

一个重构任务执行 3 小时后关闭应用。

第二天点击 Resume：

- task canvas 恢复
- 已完成步骤恢复
- 剩余步骤恢复
- 必要 refs 可读
- 不要求加载整个旧 transcript

## C. Knowledge + Code

用户问：

> 改 PaymentService.retry() 会影响哪些地方？

系统应优先使用 CodeGraph；若有关架构约束，再补 Wiki/Memory。

## D. Skill

同类数据库 migration 成功执行三次。

系统生成 candidate：

```text
Postgres migration workflow
```

用户审核后启用。

下次 migration，matching skill 自动进入 context。

## E. 错误记忆治理

旧决定被新决定替代：

```text
REST → gRPC
```

旧 Atom 标记 superseded，不作为默认事实注入。

---

# 9. Out of Product

以下不属于 Memory Layer 本身：

- LLM provider billing
- coding CLI implementation
- IDE editor
- source control UI
- generic RAG platform
- CI server

它们通过事件/adapter 与本系统协同。

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
