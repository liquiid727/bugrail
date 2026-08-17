# Codeg Memory & Context System Spec

> Status: Draft / Architecture Proposal  
> Target: Codeg  
> Scope: Memory / Knowledge / Code Intelligence / Skills / Context Orchestration  
> Date: 2026-08-12

---

## 1. 背景

Codeg 当前的演进方向已经不再只是一个对 Claude CLI、Codex CLI 等 Coding CLI 的统一 GUI，而是在逐步形成一个面向 Coding Agent 的完整 Harness / Runtime，包括：

- 多模型供应商与模型配置；
- Agent Team / 专家 Agent；
- Idea / Plan / Spec / Implementation / Review / Test 等不同角色；
- Agent 专属 Prompt / Skill / Knowledge / Tool；
- 多 Agent 编排；
- 项目级长期上下文；
- 自动 Skill 演化；
- Codebase 理解；
- Wiki / Knowledge Graph；
- Agent 执行过程的可观测性。

在该架构下，Memory 不应该作为一个孤立的“聊天记忆功能”存在。

Codeg 最终需要的是一套统一的：

**Context System（上下文系统）**

Memory 只是 Context System 中的一种资产。

---

# 2. 核心结论

Codeg 不建议从零开发完整的：

- Memory；
- Wiki；
- CodeGraph；
- Skill；
- Retrieval；
- Memory Panel。

第一阶段推荐直接采用：

**TencentDB Agent Memory**

作为 Codeg Memory / Context Runtime 的 bootstrap implementation。

但 Codeg 不应该与 TencentDB Agent Memory 强绑定。

推荐定位：

> TencentDB Agent Memory = Codeg 第一版 Memory Engine Provider / Reference Implementation

而不是：

> TencentDB Agent Memory = Codeg Memory Architecture 本身

Codeg 自己必须掌握：

1. Context Domain Model；
2. Context Orchestrator；
3. Agent Loadout；
4. Context Package；
5. Provider Contract；
6. Context UI / Observability。

底层 Memory / Wiki / CodeGraph / Retrieval / Skill Engine 应允许独立替换。

---

# 3. 参考项目定位

当前推荐参考项目如下：

| 项目 | Codeg 中的定位 |
|---|---|
| TencentDB Agent Memory | 第一版 Memory / Context Runtime 主底座 |
| OpenViking | ContextFS、分层检索、Retrieval Trajectory |
| OpenDeepWiki | Wiki 生成能力参考 / 备选 |
| RepoDoc | Wiki 增量更新与 Code → Documentation Impact |
| Codebase-Memory-MCP | CodeGraph / Code Intelligence 2.0 |
| Graphify | Project Knowledge Graph 与图谱可视化 |
| Understand Anything | GUI / Knowledge Graph 可视化参考 |
| SkillWiki | Skill Evolution / Skill Knowledge 参考 |
| Codeg | Context Orchestrator、Agent Loadout、Provider Contract、UI |

整体策略：

```text
TencentDB Agent Memory
        ↓
      跑起来
        ↓
     接入 Codeg
        ↓
找 Coding Agent 场景中的实际缺陷
        ↓
按模块替换 / 强化
```

而不是：

```text
Codeg
 ↓
自己重新开发
Memory
+ Wiki
+ CodeGraph
+ Skill
+ Retrieval
+ Panel
```

---

# 4. 架构原则

## 4.1 Memory 不等于 Context

Codeg 内部不建议把以下所有内容统一称为 Memory：

```text
Chat Memory
Skill
Wiki
CodeGraph
Rules
Docs
```

建议建立一级概念：

```text
Context Asset
```

Context Asset 表示所有可能参与 Agent 推理和执行的长期 / 半长期上下文资产。

---

## 4.2 建议领域模型

```text
Codeg Context System
│
├── Memory
│   ├── User Memory
│   ├── Project Memory
│   ├── Agent Memory
│   ├── Session Memory
│   └── Experience
│
├── Knowledge
│   ├── Wiki
│   ├── Documentation
│   ├── Architecture
│   └── Domain Knowledge
│
├── Code Intelligence
│   ├── Code Index
│   ├── Symbols
│   ├── Dependencies
│   ├── Call Graph
│   ├── API Graph
│   └── Impact Analysis
│
├── Skills
│   ├── System Skills
│   ├── Project Skills
│   ├── Agent Skills
│   └── Learned Skills
│
├── Rules
│   ├── System Rules
│   ├── Project Rules
│   └── Scoped Rules
│
└── Context Runtime
    ├── Retrieval
    ├── Ranking
    ├── Routing
    ├── Loadout
    ├── Budget
    ├── Provenance
    ├── Compression
    └── Injection
```

---

# 5. Codeg 总体架构

```text
┌──────────────────────────────────────────────┐
│                  Codeg UI                    │
│                                              │
│ Agent / Context / Wiki / Graph / Skill       │
├──────────────────────────────────────────────┤
│              Agent Orchestrator              │
│                                              │
│ Idea / Plan / Spec / Coder / Review / Test   │
├──────────────────────────────────────────────┤
│             Context Orchestrator             │
│                                              │
│ Route / Rank / Budget / Loadout / Provenance │
├──────────────────────────────────────────────┤
│               Context Services               │
│                                              │
│ Memory │ Knowledge │ Code │ Skills │ Rules   │
├──────────────────────────────────────────────┤
│                  Providers                   │
│                                              │
│ Tencent │ Viking │ RepoDoc │ CBM-MCP │ ...   │
├──────────────────────────────────────────────┤
│                   Storage                    │
│                                              │
│ SQL │ Vector │ Graph │ Files │ Git           │
└──────────────────────────────────────────────┘
```

其中 Codeg 应牢牢掌握：

```text
Agent Orchestrator
Context Orchestrator
Context Contract
Provider Contract
Agent Loadout
Context Package
Observability UI
```

---

# 6. TencentDB Agent Memory 的定位

第一阶段：

```text
Codeg
  │
  ▼
Codeg Memory Gateway
  │
  ▼
TencentDB Adapter
  │
  ▼
TencentDB Agent Memory
```

Codeg Agent 不应直接依赖 TencentDB API。

错误：

```text
PlannerAgent
   ↓
TencentDB Memory API
```

推荐：

```text
PlannerAgent
    ↓
Context Orchestrator
    ↓
MemoryService
    ↓
MemoryProvider
    ↓
TencentDBMemoryProvider
```

如此未来可以：

```text
TencentDBMemoryProvider
          ↓ replace
OpenVikingMemoryProvider
LocalMemoryProvider
CloudMemoryProvider
CustomMemoryProvider
```

而不影响 Agent Runtime。

---

# 7. Provider Architecture

推荐定义统一 Provider Contract。

例如：

```ts
interface MemoryProvider {
  search(input: MemorySearchInput): Promise<MemoryItem[]>;
  remember(input: MemoryWriteInput): Promise<MemoryItem>;
  forget(id: string): Promise<void>;
  get(id: string): Promise<MemoryItem | null>;
}

interface KnowledgeProvider {
  search(input: KnowledgeSearchInput): Promise<KnowledgeItem[]>;
  get(id: string): Promise<KnowledgeItem | null>;
}

interface CodeIntelligenceProvider {
  searchSymbols(query: string): Promise<CodeSymbol[]>;
  getDependencies(symbol: string): Promise<DependencyGraph>;
  getImpact(input: ImpactQuery): Promise<ImpactResult>;
}

interface SkillProvider {
  resolve(input: SkillResolveInput): Promise<Skill[]>;
  get(id: string): Promise<Skill | null>;
}
```

Provider 内部可对应：

```text
TencentDB
OpenViking
RepoDoc
Codebase-Memory-MCP
Local Markdown
SQLite
Postgres
Vector DB
Remote API
```

---

# 8. Context Orchestrator

Context Orchestrator 是整个系统中最重要、最应该由 Codeg 自己实现的组件。

它负责回答：

> 当前这个 Agent，在当前 Task、当前 Stage、当前 Project 下，究竟应该知道什么？

---

## 8.1 输入

```text
User Request

Current Agent

Task Type

Project

Current Mode

Current Stage

Current Files

Conversation

Agent Loadout

Token Budget
```

例如：

```text
request:
"帮我重构支付模块"

agent:
architecture-agent

stage:
plan

project:
my-saas

mode:
spec
```

---

## 8.2 Context Orchestrator 执行

```text
Task Analyzer
     ↓
Context Requirements
     ↓
Loadout Resolver
     ↓
Provider Routing
     ↓
Retrieval
     ↓
Ranking
     ↓
Deduplication
     ↓
Budgeting
     ↓
Compression
     ↓
Context Package
```

---

## 8.3 示例

```text
User:
帮我重构支付模块
        │
        ▼
Task Analyzer
        │
        ├── task = refactor
        ├── scope = payment
        ├── stage = plan
        └── agent = architect
        │
        ▼
Context Orchestrator
        │
        ├── Memory
        │    └── 历史支付架构决策
        │
        ├── Knowledge
        │    └── Payment Architecture Wiki
        │
        ├── Code Intelligence
        │    └── PaymentService Dependency Graph
        │
        ├── Skills
        │    ├── architecture-refactor
        │    └── migration-safety
        │
        └── Rules
             └── project conventions
```

---

# 9. Context Package

Context Package 建议成为 Codeg Agent Runtime 的一级对象。

```ts
interface ContextPackage {
  task: TaskContext;

  memory: MemoryItem[];
  knowledge: KnowledgeItem[];
  code: CodeContext[];
  skills: Skill[];
  rules: Rule[];

  provenance: ContextSource[];

  budget: {
    maxTokens: number;
    usedTokens: number;
  };
}
```

完整调用链：

```text
Agent
  ↓
Context Request
  ↓
Context Orchestrator
  ↓
Memory Gateway
Knowledge Gateway
Code Intelligence Gateway
Skill Registry
Rules Engine
  ↓
Context Package
  ↓
Prompt Compiler
  ↓
ACP / CLI Adapter
  ↓
Claude / Codex / Gemini / DeepSeek / ...
```

---

# 10. Agent Loadout

Agent Profile 不应该只配置模型。

建议最终统一为：

```text
Agent =
Model Profile
+ Reasoning Profile
+ Prompt
+ Skills
+ Knowledge
+ Memory
+ Tools
+ Rules
+ Context Policy
```

例如：

```yaml
agents:
  architect:
    model:
      provider: openai
      name: gpt-5.6-sol
      reasoning: high

    skills:
      - architecture-design
      - ddd
      - refactoring

    knowledge:
      - project-wiki
      - architecture-docs

    memory:
      - project-decisions
      - architecture-history

    tools:
      - codegraph
      - search
      - git

    context:
      max_tokens: 48000
      retrieval_profile: architecture


  coder:
    model:
      provider: deepseek
      name: deepseek-coder

    skills:
      - implementation
      - project-conventions

    knowledge:
      - project-wiki

    memory:
      - coding-history

    tools:
      - codegraph
      - filesystem
      - terminal

    context:
      max_tokens: 32000
      retrieval_profile: implementation
```

因此可以形成一个非常清晰的概念：

```text
Agent Profile 决定：它是谁
Model Profile 决定：它怎么思考
Skill 决定：它会什么
Context Loadout 决定：它知道什么
Tools 决定：它能做什么
```

---

# 11. ContextFS

OpenViking 最值得 Codeg 借鉴的不是单纯 Memory，而是 ContextFS。

建议未来对 Agent 暴露统一逻辑 namespace：

```text
codeg://project/my-app/

├── memory/
│   ├── decisions/
│   ├── conventions/
│   ├── incidents/
│   └── experiences/
│
├── knowledge/
│   ├── wiki/
│   ├── docs/
│   └── architecture/
│
├── code/
│   ├── modules/
│   ├── symbols/
│   ├── dependencies/
│   └── graph/
│
├── skills/
│   ├── debugging/
│   ├── testing/
│   └── release/
│
├── rules/
│
└── sessions/
```

底层并不要求真的是文件系统。

实际存储可能来自：

```text
TencentDB
Postgres
SQLite
Vector DB
Markdown
Git
Remote Service
```

但 Agent 看到的应尽量是统一逻辑空间。

---

# 12. 分层 Context

建议参考 OpenViking 的分层 Context 思路。

例如一个 Wiki / Module 可以包含：

```text
L0 Abstract
L1 Overview
L2 Full Detail
```

默认 retrieval 首先获取：

```text
L0
```

只有当相关度足够高或者 Agent 主动深入时再读取：

```text
L1
L2
```

避免：

```text
搜索一次
↓
塞入几十个 chunk
↓
大量 token 浪费
```

变成：

```text
Discover
↓
Overview
↓
Drill Down
↓
Detail
```

这非常适合大型 Codebase。

---

# 13. Code Intelligence / CodeGraph

TencentDB 自带 CodeGraph 可以作为 Phase 1 起点。

长期建议把 Code Intelligence 独立为 Provider。

重点能力：

```text
Repository Tree
Symbols
Classes
Functions
Imports
Dependencies
Call Graph
Routes
DB Models
API
Cross-service Links
Impact Analysis
Change Impact
```

候选实现：

```text
Codebase-Memory-MCP
```

未来架构：

```text
Context Orchestrator
        ↓
CodeIntelligenceService
        ↓
CodeIntelligenceProvider
        ↓
TencentDBCodeGraph
or
CodebaseMemoryMCP
or
Custom Graph Engine
```

---

# 14. Wiki / Knowledge

Wiki 不应该只做：

```text
Repository
 ↓
LLM
 ↓
Generate Wiki Once
```

真正重要的是长期同步。

建议：

```text
Repository
    ↓
Knowledge Graph
    ↓
Module Mapping
    ↓
Wiki
```

Git Change 后：

```text
git diff
   ↓
changed symbols
   ↓
dependency impact
   ↓
affected knowledge nodes
   ↓
affected wiki pages
   ↓
selective regeneration
```

该方向重点参考：

```text
RepoDoc
```

因此未来可形成：

**Knowledge Sync Engine**

---

# 15. Skill 与 Memory 的关系

Codeg 不应该把一次 Agent Execution 直接沉淀为 Skill。

推荐生命周期：

```text
Execution
   ↓
Trace
   ↓
Experience
   ↓
Experience Candidate
   ↓
Pattern Mining
   ↓
Skill Candidate
   ↓
Validation
   ↓
Skill
```

也就是说：

```text
Session / Execution
```

首先沉淀成：

```text
Experience Memory
```

而不是：

```text
Skill
```

---

## 15.1 Skill 晋升条件

只有经验满足一定条件后才可以形成 Skill：

```text
Reusable

Repeated

Task-independent

Stable trigger

Stable workflow

Observable success criteria

Validation method
```

例如：

```text
过去 20 次前端任务

发现：
修改 API schema 后
通常都需要：
1. update generated client
2. update types
3. run typecheck
4. update tests

该模式重复成功出现
```

才可能形成：

```text
Skill:
api-schema-change
```

而不是把任意一次执行链路保存为 Skill。

---

# 16. Experience Memory

建议专门增加：

```text
Experience
```

类型。

示例：

```yaml
experience:
  task: migrate-user-table
  trigger:
    - prisma schema changed

  actions:
    - generate migration
    - run migration test
    - regenerate client

  result:
    status: success

  environment:
    project: xxx
    framework: prisma

  evidence:
    tests_passed: true
```

Experience 是：

```text
Memory → Skill
```

之间的中间层。

---

# 17. Retrieval Provenance

Codeg UI 应允许用户知道：

> Agent 为什么拿到了这些 Context？

每个 ContextItem 建议记录：

```ts
interface ContextSource {
  provider: string;
  sourceId: string;
  sourceType: string;

  query?: string;

  score?: number;

  reason?: string;

  retrievalPath?: string[];

  timestamp?: number;
}
```

---

# 18. Context Activity UI

不建议只做：

```text
Memory Panel
```

推荐一级入口：

```text
Context
```

结构：

```text
Context

├── Overview
├── Memory
├── Knowledge
│   ├── Wiki
│   └── Docs
├── Codebase
│   ├── Index
│   └── Graph
├── Skills
├── Agents
│   └── Loadout
└── Activity
```

---

## 18.1 Activity 示例

```text
Task
Refactor PaymentService

Agent
architecture-agent

Context Used
──────────────────────

Memory
✓ ADR-018 Keep Stripe adapter isolated

Knowledge
✓ Payment Architecture
✓ Billing Domain

Code
✓ PaymentService
✓ StripeGateway
✓ SubscriptionService

Skills
✓ architecture-refactor
✓ migration-safety

Rules
✓ backend conventions

Tokens
12.4K / 32K

Reason
PaymentService
 → StripeGateway
 → Billing Domain

Providers
TencentDB Memory
CodeGraph
Project Skills
```

这会成为非常重要的 Context Observability 能力。

---

# 19. Context Budget

Context Orchestrator 必须处理 token budget。

例如：

```yaml
context_budget:
  total: 48000

  allocation:
    system: 4000
    rules: 4000
    skills: 6000
    memory: 6000
    knowledge: 10000
    code: 14000
    conversation: 4000
```

但实际运行时应该允许动态变化。

例如：

Architecture Agent：

```text
Knowledge ↑
Architecture Memory ↑
Code Dependencies ↑
Implementation Detail ↓
```

Coder：

```text
Code ↑
Rules ↑
Skills ↑
Wiki ↓
Long-term Memory ↓
```

Reviewer：

```text
Diff ↑
Rules ↑
Historical Bugs ↑
CodeGraph ↑
```

---

# 20. Scope

## Phase 1 包含

```text
TencentDB integration
Memory Gateway
Provider abstraction
basic Agent Loadout
Memory read/write
Wiki read
CodeGraph read
Skill binding
basic Context Panel
Context provenance
```

---

## Phase 1 不包含

```text
复杂自动 Memory Routing
自主 Skill 生成
全量 ContextFS
完整 Knowledge Graph
复杂 Wiki 增量更新
完全替换 TencentDB CodeGraph
多 Provider 自动竞价 / ensemble retrieval
```

避免第一阶段过度设计。

---

# 21. Phase 1 — Bootstrap

目标：

> 用最小成本证明 Memory / Context 对 Codeg Coding Agent 的真实价值。

架构：

```text
TencentDB Agent Memory
        ↓
TencentDB Adapter
        ↓
Codeg Memory Gateway
        ↓
Context Orchestrator Lite
        ↓
Agent Loadout
        ↓
Agent
```

实现：

### 21.1 TencentDB Agent Memory 部署

首先原样运行：

```text
MemoryCore
MemoryKnowledge
MemoryProxy
MemoryPanel
```

理解内部链路：

```text
MemoryCore
   ↓
MemoryKnowledge
   ↓
Proxy
   ↓
Agent Loadout
   ↓
Tool API
```

目的不是马上 fork。

首先确定：

```text
哪些模块可以原封不动使用？
哪些模块需要 Adapter？
哪些模块不适合 Coding Agent？
哪些模块未来需要替换？
```

---

## 21.2 Codeg Adapter

实现：

```text
packages/context
packages/memory
packages/providers/tencent-memory
```

示例：

```text
context/
  core/
  orchestrator/
  types/
  loadout/

providers/
  tencent-memory/
    memory.ts
    knowledge.ts
    skill.ts
    codegraph.ts
```

---

## 21.3 MVP Workflow

必须先跑通：

```text
Session
 ↓
Agent Execution
 ↓
Memory Write

New Session
 ↓
Agent
 ↓
Memory Retrieval
 ↓
Context Injection
 ↓
Execution
```

再跑：

```text
Agent
 ↓
Loadout
 ↓
Skill
Wiki
CodeGraph
Memory
 ↓
Context Package
```

---

# 22. Phase 2 — Codeg Context Runtime

Phase 2 才正式建立 Codeg 自己的竞争力。

实现：

```text
Context Orchestrator

Context Package

Dynamic Retrieval

Context Ranking

Budget

Compression

Provenance

Agent scoped context

Task scoped context

Context Activity UI
```

核心目标：

> TencentDB 变成 Provider，而不再控制 Codeg 的 Context Architecture。

---

# 23. Phase 3 — Provider Evolution

逐步替换单点能力。

---

## 23.1 Retrieval

参考：

```text
OpenViking
```

引入：

```text
ContextFS
Hierarchical Retrieval
L0 / L1 / L2
Retrieval Trajectory
```

---

## 23.2 Wiki

参考：

```text
RepoDoc
OpenDeepWiki
```

增加：

```text
incremental wiki update
change impact
selective regeneration
```

---

## 23.3 CodeGraph

替换 / 强化：

```text
Codebase-Memory-MCP
```

增加：

```text
AST
symbols
call graph
impact analysis
cross-service graph
```

---

## 23.4 Skill Evolution

Codeg 自己实现：

```text
Execution Trace
 ↓
Experience
 ↓
Pattern
 ↓
Skill Candidate
 ↓
Evaluation
 ↓
Project Skill
```

参考：

```text
SkillWiki
Hermes-style skill lifecycle
```

---

# 24. 建议目录

未来可考虑：

```text
packages/

  context-core/
    types/
    package/
    provenance/
    budget/

  context-orchestrator/
    routing/
    ranking/
    retrieval/
    compression/

  agent-runtime/
    profiles/
    loadout/
    execution/

  memory/
    memory-service/
    experience/

  knowledge/
    wiki/
    docs/

  code-intelligence/
    graph/
    symbols/
    impact/

  skills/
    registry/
    evolution/

  providers/
    tencent-memory/
    openviking/
    repodoc/
    codebase-memory/

apps/

  desktop/

    context-panel/
    memory-panel/
    wiki-panel/
    graph-panel/
    skills-panel/
```

---

# 25. 数据标识

每个 Context Asset 应尽量有统一 ID。

例如：

```text
memory://project/foo/decision/18

knowledge://project/foo/wiki/payment

code://project/foo/symbol/PaymentService

skill://project/foo/api-migration

rule://project/foo/backend-conventions
```

后续如果引入 ContextFS：

```text
codeg://project/foo/...
```

可以成为统一入口。

---

# 26. 多 Agent 隔离

Memory 需要考虑不同 scope：

```text
Global

User

Workspace

Project

Team

Agent

Session

Task
```

建议优先级：

```text
Task
  ↓
Session
  ↓
Agent
  ↓
Project
  ↓
Workspace
  ↓
User
  ↓
Global
```

例如：

Architecture Agent 可以访问：

```text
Project Architecture Memory
Project ADR
Architecture Skills
Project Wiki
```

Coder 默认无需读取：

```text
大量 brainstorming history
```

---

# 27. Memory 写入原则

不能把所有 conversation 都长期保存为 Memory。

建议：

```text
Raw Conversation
      ↓
Memory Extractor
      ↓
Candidate
      ↓
Classification
      ↓
Dedup
      ↓
Importance
      ↓
Memory
```

分类：

```text
Decision
Preference
Constraint
Convention
Fact
Experience
Incident
TODO
Architecture
```

---

# 28. 生命周期

建议 Memory 状态：

```text
candidate
active
deprecated
superseded
archived
```

例如：

```text
ADR-018
status: superseded
superseded_by: ADR-025
```

这样 Agent 不会持续注入已经过时的决策。

---

# 29. Git Integration

Coding Agent 的 Memory 必须和 Git 建立联系。

ContextItem 可记录：

```text
repo
branch
commit
file
symbol
range
```

例如：

```yaml
memory:
  type: architecture-decision

  source:
    commit: a13df1
    files:
      - src/payment/service.ts

  valid_since:
    commit: a13df1
```

后续可以判断：

```text
代码已经大改
↓
Memory 可能 stale
```

这是 Coding Memory 和普通 Chat Memory 最大区别之一。

---

# 30. Freshness / Staleness

Context Asset 应增加：

```text
freshness
```

例如：

```ts
interface Freshness {
  createdAt: number;
  updatedAt: number;

  sourceCommit?: string;

  stale?: boolean;

  staleReason?: string;
}
```

Git Change 可以触发：

```text
CodeGraph refresh
Wiki refresh
Memory validation
Skill validation
```

---

# 31. Tool API

Agent 不一定需要知道底层 Provider。

建议统一暴露：

```text
context.search

context.read

context.tree

memory.search

memory.remember

knowledge.search

code.symbol

code.dependencies

code.impact

skill.search

skill.read
```

然后 Tool Runtime 再 route：

```text
Tool
 ↓
Context Service
 ↓
Provider
```

---

# 32. 与 CLI / ACP 的关系

Codeg 的 Context System 应位于 CLI Adapter 之上。

```text
Codeg Agent Runtime

Context Runtime

Prompt / Tool Compiler

CLI Adapter / ACP Adapter

Claude CLI
Codex CLI
Gemini CLI
Other CLI
```

不应该让：

```text
Claude CLI Adapter
```

自己决定 Memory / Wiki / Skill。

如此同一 Agent Profile 才可以：

```text
GPT
Claude
DeepSeek
Gemini
```

之间切换。

---

# 33. 与 Plan / Spec Mode 的关系

Context Loadout 也应该根据 Mode 改变。

例如：

```text
Plan Mode
```

优先：

```text
architecture
wiki
ADR
codegraph
spec skills
historical decisions
```

Implementation Mode：

```text
source
tests
implementation skills
project rules
recent errors
```

Review Mode：

```text
diff
rules
tests
known incidents
dependencies
security skills
```

因此：

```text
Agent Profile
+
Mode Profile
+
Task Profile
```

共同决定 Context。

---

# 34. 与 Agent Team 的关系

未来多 Agent 编排：

```text
Planner
   ↓
Coder
   ↓
Reviewer
```

不应该把 Planner 全部 prompt 原样传给 Coder。

推荐传：

```text
Task Artifact
+
Shared Context
+
Scoped Memory
+
Relevant Decisions
```

也就是：

```text
Agent A Context
      ↓
Artifact / Memory
      ↓
Agent B Context
```

而不是：

```text
Agent A Entire Conversation
      ↓
Agent B
```

---

# 35. UI 产品方向

Context UI 是 Codeg 和单纯 Memory Server 的主要差异之一。

建议：

```text
Context
  Overview
  Memory
  Knowledge
  Codebase
  Skills
  Agents
  Activity
```

尤其需要支持：

```text
为什么加载？

来源是什么？

哪个 Agent 加载？

消耗多少 Token？

相关度？

是否 stale？

是否自动产生？

是否用户固定？

是否属于 Project？
```

---

# 36. 用户控制

ContextItem 应支持：

```text
Pin

Unpin

Disable

Delete

Edit

Promote

Demote

Mark stale

Always load

Never load

Agent scope

Project scope
```

避免形成一个不可解释的“黑盒记忆系统”。

---

# 37. Observability

每一次 Agent Execution 建议记录：

```text
Agent

Model

Task

Context Package

Retrieval Query

Provider

Retrieved Items

Dropped Items

Token Cost

Tool Calls

Result

User Feedback
```

这批数据未来同时用于：

```text
Context optimization
Skill evolution
Memory quality evaluation
Agent evaluation
```

---

# 38. Evaluation

不能只测试：

```text
Memory 能不能搜出来
```

真正需要评价：

```text
Task Success

Context Precision

Context Recall

Token Cost

Tool Calls

Repository Exploration Cost

Hallucination

Stale Context Rate

User Correction Rate
```

最终指标类似：

```text
同一个 coding task

Without Context System

vs

With Context System
```

比较：

```text
time
tokens
tool calls
correctness
rework
```

---

# 39. 第一阶段最值得验证的场景

推荐优先测试：

### 场景 A

```text
Project Convention Recall
```

Agent 是否能自动知道：

```text
项目不允许直接访问数据库
必须经过 Repository
```

---

### 场景 B

```text
Architecture Decision Recall
```

Agent 是否记住：

```text
Stripe adapter 必须与 domain 隔离
```

---

### 场景 C

```text
Cross-session Coding
```

Session A：

```text
修改 payment
```

Session B：

```text
继续 payment
```

是否能够减少重新探索。

---

### 场景 D

```text
Agent-specific Context
```

Planner 和 Coder 是否加载不同 Context。

---

### 场景 E

```text
Review
```

Reviewer 是否能够自动加载：

```text
Project Rules
Historical Bugs
Architecture Decisions
Diff Dependencies
```

---

# 40. 风险

## 风险 1：TencentDB 深度耦合

解决：

```text
Provider Adapter
```

---

## 风险 2：Context 越来越大

解决：

```text
Context Budget
Hierarchical Retrieval
Ranking
Compression
```

---

## 风险 3：Memory 污染

解决：

```text
Candidate
Validation
Confidence
Scope
Freshness
Lifecycle
```

---

## 风险 4：过期 Knowledge

解决：

```text
Git-aware freshness
RepoDoc-style incremental update
```

---

## 风险 5：Skill 污染

解决：

```text
Experience != Skill
Skill Candidate
Evaluation
Promotion
```

---

## 风险 6：Agent 不知道为什么得到某 Context

解决：

```text
Provenance
Retrieval Trajectory
Activity UI
```

---

# 41. Non-goals

当前阶段不追求：

```text
完全自主的 AGI Memory

无限长期聊天记录

所有 Context 自动生成

完全无人工控制

一开始就实现完整 Knowledge Graph

一次性集成所有开源项目
```

核心原则：

> 先验证真实收益，再升级基础设施。

---

# 42. Implementation Roadmap

## Phase 1 — Bootstrap

目标：

```text
TencentDB Agent Memory 可用
+
Codeg 接入
+
Agent Loadout 可用
+
Context 基础 UI 可见
```

工作：

```text
[ ] TencentDB Agent Memory 本地部署

[ ] 源码模块分析

[ ] MemoryCore 边界确认

[ ] MemoryKnowledge 边界确认

[ ] Proxy / API 分析

[ ] Agent Loadout 分析

[ ] Tool API 分析

[ ] Codeg Context Domain Types

[ ] Memory Provider Contract

[ ] TencentDB Provider Adapter

[ ] Context Orchestrator Lite

[ ] Agent Loadout

[ ] Session Memory

[ ] Project Memory

[ ] Wiki Retrieval

[ ] Skill Binding

[ ] CodeGraph Retrieval

[ ] Context Activity MVP
```

---

## Phase 2 — Context Runtime

```text
[ ] Context Package

[ ] Context Routing

[ ] Dynamic Retrieval

[ ] Ranking

[ ] Dedup

[ ] Token Budget

[ ] Compression

[ ] Provenance

[ ] Retrieval Trace

[ ] Agent-scoped Context

[ ] Mode-scoped Context

[ ] Task-scoped Context

[ ] Context Activity UI

[ ] Context Evaluation
```

---

## Phase 3 — Advanced Providers

```text
[ ] OpenViking ContextFS ideas

[ ] Hierarchical Retrieval

[ ] Codebase-Memory-MCP Provider

[ ] Code Impact Analysis

[ ] RepoDoc Incremental Wiki

[ ] Knowledge Sync Engine

[ ] Skill Evolution

[ ] Experience Mining

[ ] Graph Visualization
```

---

# 43. Recommended First Source Review

下一步对 TencentDB Agent Memory 的源码分析建议按以下顺序：

```text
1. MemoryCore

2. MemoryKnowledge

3. Proxy

4. Agent Loadout

5. Tool API

6. Storage Schema

7. Search / Retrieval

8. Skill

9. Wiki

10. CodeGraph

11. MemoryPanel
```

每个模块重点回答：

```text
职责是什么？

输入输出是什么？

是否独立？

是否存在内部强耦合？

Codeg 是否需要 Adapter？

是否应该直接复用？

是否应该裁掉？

未来可能被谁替代？
```

最终形成：

```text
KEEP
WRAP
MODIFY
REPLACE
REMOVE
```

五类结论。

---

# 44. 最终战略

Codeg Memory 方向推荐正式定义为：

> TencentDB Agent Memory 做 bootstrap，Codeg 自己掌握 Context Orchestration；成熟后逐步将 Wiki、CodeGraph、Retrieval、Skill Evolution 替换为更适合 Coding Agent 的专业模块。

最终产品不是：

```text
Coding IDE
+
Memory
```

而应该是：

```text
Coding Agent Runtime

其中拥有：

Agent Team
Model Runtime
Context Runtime
Skill Runtime
Tool Runtime
Execution Runtime
```

Memory 是 Context Runtime 的一部分。

---

# 45. 最终核心模型

整个 Codeg Agent Runtime 可以压缩成：

```text
Agent
 =
Identity
+
Model
+
Reasoning
+
Prompt
+
Context
+
Skills
+
Tools
+
Rules
```

其中：

```text
Agent Profile
决定它是谁

Model Profile
决定它怎么思考

Context Loadout
决定它知道什么

Skill
决定它会什么

Tools
决定它能做什么

Rules
决定它必须遵守什么
```

Context Runtime 则负责：

> 在正确的时间，把正确的上下文，用正确的成本，交给正确的 Agent。

这应该成为 Codeg Memory / Knowledge / Code Intelligence / Skill 系统共同的长期架构目标。
