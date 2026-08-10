# SpecOS 总体蓝图

> 状态：Architecture / Product Baseline  
> 目标：作为 SpecOS 后续所有阶段 PRD、Spec、Issue 拆分的统一上位文档。  
> 基础运行时：Codeg  
> 核心定位：面向软件工程生命周期的 Agent Control Plane / Software Engineering Agent OS

---

## 1. 背景与决策

SpecOS 不再独立开发完整 Coding IDE UI，而是在 Codeg 已有能力之上增强工程控制层。

Codeg 主要承担：

- Workspace / Project
- CLI Agent Runtime
- Codex / Claude 等 Agent 运行
- Session
- Worktree
- Git
- ACP / CLI 接入
- 多会话与基础 Agent 调度
- 基础 Skills / Automations
- Desktop / Remote UI

SpecOS 不重复建设以上成熟能力，而重点建设：

- Engineering Artifact
- Agent Profile / Agent Team
- Workflow / Task DAG
- Context Compiler
- Code Intelligence
- Quality Gate
- Run Trace / Eval
- Project Memory
- Skill Evolution
- Model Routing
- Engineering Control API

最终关系：

```text
┌──────────────────────────────────────────┐
│                  SpecOS                  │
│                                          │
│ Artifact / Workflow / Agent / Context    │
│ CodeIntel / Quality / Memory / Eval      │
└────────────────────┬─────────────────────┘
                     │ Control API
┌────────────────────▼─────────────────────┐
│                  Codeg                   │
│ Workspace / Session / Git / Worktree     │
│ ACP / CLI Runtime / Desktop UI           │
└────────────────────┬─────────────────────┘
                     │
       ┌─────────────┼─────────────┐
       ▼             ▼             ▼
     Codex         Claude       Other CLI
```

---

## 2. 产品核心定义

SpecOS 的核心不是“更多 Agent”，而是：

> 将用户需求转换为可追踪的工程 Artifact 与 Issue DAG，根据任务能力、风险和上下文动态选择 Agent、模型、Skill 与 Knowledge，在隔离的 Worktree / Session 中执行，并通过 Review、Test、QA 和 Acceptance Criteria 完成工程验证，最后把有效经验沉淀成项目 Memory、Rule、Pattern 或 Skill。

核心生命周期：

```text
Idea
  ↓
PRD
  ↓
Spec
  ↓
Issues / Task DAG
  ↓
Execution
  ↓
Review
  ↓
Test
  ↓
QA
  ↓
Ship
  ↓
Memory / Skill Evolution
```

并非所有任务都强制走完整链路。

---

## 3. 设计原则

### 3.1 Codeg 是 Runtime，SpecOS 是 Control Plane

SpecOS 尽量通过稳定接口控制 Codeg，不把核心工程知识强耦合到 Codeg UI 或具体 CLI。

### 3.2 Artifact First

PRD、Spec、Issue、Review、Test、Run、Release 都是一等对象，而非散落的聊天文本。

### 3.3 Agent 是 Profile，不是固定实例

`backend-agent`、`review-agent`、`idea-agent` 本质是可配置 Agent Profile。

Profile 由以下内容组成：

- Role
- Model Policy
- Skills
- Knowledge
- Tools
- Context Policy
- Permissions
- Quality Policy
- Cost / latency preference

运行时根据任务动态实例化。

### 3.4 Workflow 动态选择，不强制固定流水线

引入 Complexity / Workflow Router：

| Level | 场景 | 默认流程 |
|---|---|---|
| L0 | 极小修改 | Execute → Verify |
| L1 | 普通单点任务 | Plan → Execute → Review |
| L2 | 独立 Feature | Spec → Issues → Execute → Test |
| L3 | 跨模块 Feature | Architecture → Spec → Parallel Issues → Integration → QA |
| L4 | 架构变化 | Idea → Architecture → PRD → Spec → Milestones |
| L5 | 大型 Initiative | Program / Multi-stage Workflow |

### 3.5 Context 按任务编译，不向 Agent 灌入全部知识

每一次 Run 生成独立 Context Pack。

### 3.6 Agent 不能自证完成

Done 必须由 Acceptance Criteria、Build、Test、Review 等 Quality Gates 决定。

### 3.7 自动记忆不等于自动生成 Skill

经验先分类，再通过证据和验证决定是否提升为 Skill。

### 3.8 可追踪、可回放、可评估

所有重要 Agent Run 必须有 Trace，支持后续分析成功率、成本、模型效果和 Skill 效果。

---

# 4. 核心领域模型

## 4.1 Engineering Artifact

建议首批 Artifact：

```text
Idea
PRD
Spec
Issue
Run
Review
Test
QA
Release
ADR
```

关系：

```text
Idea
 └── PRD
      └── Spec
           ├── Issue
           │    ├── Run
           │    ├── Commit
           │    ├── Review
           │    └── Test
           ├── Acceptance Criteria
           └── ADR
                 ↓
              Release
```

所有对象需要：

```yaml
id:
project_id:
type:
title:
status:
version:
created_at:
updated_at:
created_by:
relations:
metadata:
```

---

## 4.2 Issue = Task Contract

Issue 不仅是 Markdown 文本，还要有机器可读结构。

```yaml
id: ISSUE-142
title: 支持部分退款

goal:
  支持订单支付后的部分退款。

depends_on:
  - ISSUE-139

scope:
  modules:
    - payment
    - order

acceptance_criteria:
  - AC-001
  - AC-002
  - AC-003

verification:
  required:
    - build
    - unit-test
    - integration-test
    - review

risk:
  level: high

agent:
  capabilities:
    - backend
    - payment

worktree:
  isolated: true
```

Issue 是 Scheduler、Context Compiler、Agent Resolver、Quality Engine 的共同输入。

---

## 4.3 Agent Profile

示例：

```yaml
id: backend-payment-agent
name: Payment Backend Expert

role:
  domain: payment
  purpose:
    - implementation
    - debugging

capabilities:
  - backend
  - payment
  - api
  - database

model_policy:
  quality: high
  cost: medium
  primary:
    provider: anthropic
    model: sonnet
  fallback:
    - provider: openai
      model: codex

skills:
  - ddd-backend
  - api-design
  - transaction-pattern

knowledge:
  - docs/domain/payment
  - docs/backend
  - ADR/payment

tools:
  - filesystem
  - shell
  - git
  - codebase
  - lsp

permissions:
  filesystem: write
  shell: allowed
  git_commit: true
  git_push: false
  production: false

quality:
  require:
    - build
    - unit-test
```

建议内置 Profile：

- idea-agent
- architecture-agent
- plan-agent
- spec-agent
- execution-agent
- frontend-agent
- backend-agent
- database-agent
- security-agent
- review-agent
- test-agent
- qa-agent
- integration-agent
- release-agent

但它们都通过 Registry 管理，而非写死在 Orchestrator。

---

# 5. Agent Team 与编排

## 5.1 Capability Resolver

任务先解析所需能力：

```text
“增加支付退款导出 Excel”
        ↓
backend
payment
export
frontend
permission
test
```

再从 Agent Registry 中选择 Profile。

---

## 5.2 Team Builder

生成动态 Team：

```text
architecture-agent
        ↓
 ┌──────┴──────┐
 ▼             ▼
backend      frontend
 ▼             ▼
 └──────┬──────┘
        ▼
 integration
        ▼
 review
        ▼
 test
```

简单任务可以只有一个 Agent。

---

## 5.3 Task DAG

每个节点至少包含：

```yaml
task_id:
issue_id:
agent_profile:
capabilities:
depends_on:
status:
session_id:
worktree_id:
run_id:
quality_gate:
```

状态：

```text
pending
ready
running
blocked
reviewing
verifying
completed
failed
cancelled
```

---

## 5.4 Handoff Protocol

Agent 间不依赖长聊天上下文传递结果。

```yaml
handoff:
  task: ISSUE-102
  status: completed

  changed:
    - PaymentService
    - RefundRepository

  decisions:
    - refund_id used as idempotency key

  risks:
    - legacy refund path remains

  verification:
    unit_test: passed

  artifacts:
    commit: 72ac839
```

---

# 6. Context Compiler

Context Compiler 是 SpecOS 核心服务之一。

输入：

- Issue
- Agent Profile
- Current Artifact
- Code Intelligence
- Project Memory
- Skills
- Knowledge
- Git history
- Related Runs

输出：

```text
Context Pack
├── Goal
├── Scope
├── Acceptance Criteria
├── Related Spec
├── Related ADR
├── Relevant Symbols
├── Relevant Files
├── Call Graph / Dependency
├── Relevant Tests
├── Project Rules
├── Skills
├── Memory
├── Previous Decisions
└── Execution Constraints
```

原则：

- 只包含当前 Agent 需要的内容
- 记录每个 Context item 的来源
- 支持 token budget
- 支持 relevance score
- 支持缓存
- 支持 Run 后分析“哪些 Context 真正有用”

---

# 7. Code Intelligence Service

Codebase 能力不能只做向量检索。

目标组成：

```text
Filesystem Index
+
Git Index
+
AST Index
+
Symbol Index
+
LSP
+
Reference Graph
+
Dependency Graph
+
Semantic Retrieval
+
Change Impact Analysis
```

核心查询：

```text
definition(symbol)
references(symbol)
implementations(symbol)
callers(symbol)
callees(symbol)
tests(symbol)
history(symbol)
related_artifacts(symbol)
impact(symbol_or_file)
```

预期返回：

```text
PaymentService.refund
├── Definition
├── References
├── Callers
├── Callees
├── Implementations
├── Tests
├── Git History
├── Related Issues
├── Related Specs
└── Related ADR
```

---

# 8. Project Graph

长期将以下对象连接成项目知识图谱：

```text
Code
Docs
Artifacts
Symbols
Tests
Git
ADR
Runs
Agents
Skills
Memory
```

示例：

```text
PaymentService
 ├── defined_in → payment.ts
 ├── covered_by → refund.test.ts
 ├── belongs_to → payment module
 ├── implements → SPEC-31
 ├── changed_by → ISSUE-87
 ├── explained_by → ADR-12
 └── expert → payment-agent
```

Project Graph 是后续 Architecture Intelligence、Impact Analysis、Context Compiler 和 Memory 的共同基础。

---

# 9. Execution Runtime Integration

SpecOS 不负责重新实现 CLI Runtime。

标准执行链：

```text
Issue
 ↓
Agent Resolver
 ↓
Context Compiler
 ↓
Worktree Allocator
 ↓
Codeg Session
 ↓
CLI Agent
 ↓
Run Trace
 ↓
Verification
```

Worktree Execution Pool：

```text
ISSUE-102 → WT/backend  → Codex
ISSUE-103 → WT/frontend → Claude
ISSUE-104 waits for 102/103
```

最终 Integration Agent：

```text
merge
→ conflict resolution
→ integration test
→ quality gate
```

---

# 10. Quality Engine

## 10.1 Acceptance Criteria

Spec 中所有关键需求转化为可追踪 AC。

```text
SPEC-12
├── AC-001
├── AC-002
└── AC-003
```

Issue 与 Test 建立关系：

```text
AC-001 ← ISSUE-23 ← code
  ↑
TEST-31
```

---

## 10.2 Quality Gate

Issue Done 可由以下 Gate 构成：

```text
Implementation
Build
Lint
Type Check
Unit Test
Integration Test
Acceptance Verification
Review
Security Review
QA
Human Approval
```

按 Risk 和 Workflow Level 动态组合。

---

## 10.3 Risk Engine

风险信号：

- auth
- payment
- permission
- migration
- security
- secret
- infra
- production
- large diff
- public API breaking change
- cross-module dependency

风险等级：

```text
low
medium
high
critical
```

风险可以动态增加：

- 专家 Review
- Test 类型
- QA
- Human Approval
- 禁止自动 merge / deploy

---

# 11. Run Trace / Eval

每次运行记录：

```yaml
run:
  id:
  project_id:
  issue_id:
  agent_profile:
  model:
  provider:
  context_pack:
  skills:
  knowledge:
  tools:
  session_id:
  worktree_id:
  started_at:
  finished_at:
  token_usage:
  estimated_cost:
  changed_files:
  commits:
  tests:
  review_result:
  outcome:
```

未来指标：

- task success rate
- first-pass success rate
- test pass rate
- review rejection rate
- token cost
- latency
- model fallback rate
- skill contribution
- rework rate

为后续 Model Router 与 Skill Evolution 提供真实数据。

---

# 12. Memory 与 Skill Evolution

## 12.1 Memory 分类

经验首先分类：

| 类型 | 内容 |
|---|---|
| Fact | 项目事实 |
| Rule | 工程规范 |
| Decision | 技术决策 |
| Pattern | 重复出现的解决模式 |
| Skill Candidate | 可执行 workflow 候选 |

---

## 12.2 Skill Evolution

禁止“一次 Agent 执行 = 自动 Skill”。

正确流程：

```text
Run Observation
    ↓
Pattern Detection
    ↓
Skill Candidate
    ↓
Evidence Accumulation
    ↓
Validation
    ↓
Promotion
    ↓
Active Skill
```

Skill 生命周期：

```text
candidate
active
validated
deprecated
archived
```

统计：

```yaml
usage_count:
success_rate:
failure_rate:
last_used:
source_runs:
supported_models:
version:
```

---

# 13. UI 增强

SpecOS 不重新建设完整 IDE，而增强 Codeg UI。

## 13.1 Engineering Artifact Inspector

聊天和 Agent 输出中的 Artifact ID 可点击。

右侧统一面板支持：

- Idea
- PRD
- Plan
- Spec
- Issue
- Acceptance Criteria
- Diff
- Review
- Test
- QA
- ADR
- Release
- Run
- Context

---

## 13.2 Task Graph

可视化 DAG：

```text
             Architecture
                  │
       ┌──────────┴──────────┐
       ▼                     ▼
   Backend                 Frontend
   running                 completed
   Codex                   Claude
   WT/backend              WT/frontend
       │                     │
       └──────────┬──────────┘
                  ▼
              Integration
                  ▼
                Review
```

节点显示：

- Status
- Agent
- Model
- Session
- Worktree
- Cost
- Duration
- Tests
- Review

点击直接进入对应 Codeg Session / Artifact。

---

## 13.3 Run Inspector

展示：

- Prompt / Task Contract
- Context Pack
- Skills
- Tool calls
- Timeline
- Subagents
- Changed files
- Tests
- Cost
- Final output
- Failure reason

---

# 14. Control API / CLI

CLI 最终应覆盖：

```bash
codeg idea new

codeg spec create
codeg spec validate SPEC-12

codeg issue list
codeg issue show ISSUE-31
codeg issue run ISSUE-31

codeg team run SPEC-12

codeg run list
codeg run inspect RUN-31

codeg worktree list

codeg review ISSUE-31
codeg test ISSUE-31

codeg context inspect ISSUE-31

codeg skill list
codeg skill evolve

codeg ship SPEC-12
```

关键命令：

```bash
codeg issue run ISSUE-31
```

内部：

```text
resolve agent
→ resolve model
→ resolve skills
→ resolve knowledge
→ compile context
→ allocate worktree
→ create session
→ execute
→ verify
→ trace
```

同时为 Desktop / Web / Mobile / CI / MCP 暴露统一 Control API。

---

# 15. 权限与治理

Agent Profile 必须声明能力边界：

```yaml
permissions:
  filesystem:
    read: true
    write: true

  shell:
    allowed: true

  git:
    commit: true
    push: false
    merge: false

  network:
    allowed: restricted

  secrets:
    allowed: false

  database:
    production: false

  deployment:
    production: approval-required
```

Human Gate：

```yaml
approval:
  spec: required
  architecture: required
  merge: high-risk-only
  release: required
```

---

# 16. 完整形态与交付顺序

SpecOS 采用**契约一次定稿、按交付顺序实现**的策略，而非先做最小版再升级 API。

- 完整产品定义见 `01-SpecOS-PRD.md`（统一 PRD，取代原三份阶段 PRD）。
- 模块架构与契约见 `02-SpecOS-Module-Decomposition.md`。
- SPEC 索引与交付顺序见 `03-SpecOS-Specs.md`；落地任务见 `04-SpecOS-Issues.md`。

交付顺序只决定「先实现哪个」，不改变任何契约，避免 v1→v2→v3 的演进成本。

## Milestone 概览

| 里程碑 | 目标 | 重点 |
|---|---|---|
| M0 | 最小垂直切片 | Spec → Issue → Agent → Context → Worktree → Session → Review/Test → Done 稳定跑通 |
| M1 | 智能理解与编排 | Code Intelligence、Team Builder、DAG、Context 智能装配、Quality/Risk、Run Trace、Task UI |
| M2 | 学习与受控自治 | Auto Memory、Skill Evolution、Eval、Model Router、Architecture Intelligence、Controlled Autonomy |

内核 K1–K12 在 M0 一次建完；M1/M2 只增强插件实现与存储索引能力，不改内核契约。

---

# 17. 总体验收标准

SpecOS 完整形态应能够完成：

```text
用户：
“给支付模块增加部分退款功能”
```

系统：

1. 分析 Complexity / Risk。
2. 生成或关联 Spec。
3. 建立 Acceptance Criteria。
4. 拆分 Issue DAG。
5. 分析需要的 capabilities。
6. 选择 Agent Profiles。
7. 选择模型与 Skills。
8. 从 Code Intelligence 构建 Context Pack。
9. 自动创建 Worktree + Codeg Session。
10. Backend / Frontend 等任务可并行执行。
11. Agent 使用 Handoff Protocol 交接。
12. Integration Agent 合并。
13. Review/Test/QA 验证。
14. Acceptance Criteria 全部可追踪。
15. Human Gate 根据风险触发。
16. 完成后生成 Run Trace。
17. 从运行结果提炼 Fact / Rule / Decision / Pattern。
18. 重复成功 Pattern 可成为 Skill Candidate。
19. Eval 数据反哺模型和 Agent 路由。

最终用户可以在一个 Task Graph 中查看整个过程并随时进入对应 Session 人工介入。

---

# 18. 明确不做

为了避免项目膨胀，以下不作为 SpecOS 前期目标：

- 重做完整代码编辑器
- 重做 Terminal
- 重做 Git Client
- 自研大模型 Runtime
- 替代 Codex / Claude CLI
- 第一阶段构建复杂知识图数据库
- 第一阶段追求全自动自治
- 每个任务都强制多 Agent
- 每次 Run 自动生成 Skill
- 无限制跨项目共享 Memory

SpecOS 优先构建“工程控制能力”，而不是复制现有 IDE 能力。

---

# 19. 北极星指标

产品最终需要关注：

### 工程效率

- 从 Spec 到完成的 Cycle Time
- 并行任务比例
- 人工介入次数
- Agent 探索代码所占 Token / 时间

### 工程质量

- First-pass Success Rate
- Review 一次通过率
- Regression Rate
- Acceptance Criteria 覆盖率

### Agent 效率

- 每类 Agent 成功率
- 平均成本
- 平均耗时
- Context 命中率
- Model Fallback 率

### 学习效果

- Memory 使用率
- Skill Candidate → Validated 比例
- Skill 使用后的成功率提升
- 失效 Skill 淘汰率

---

# 20. 最终产品形态

```text
                        User
                          │
                    Idea / Request
                          │
                  Workflow Router
                          │
                  Engineering Artifact
                          │
                      Issue DAG
                          │
                 Capability Resolver
                          │
                     Team Builder
                          │
             ┌────────────┴────────────┐
             ▼                         ▼
        Agent Profile             Agent Profile
             │                         │
       Context Compiler           Context Compiler
             │                         │
      Worktree / Session         Worktree / Session
             │                         │
        Codex / Claude             Other Agent
             │                         │
             └────────────┬────────────┘
                          ▼
                     Integration
                          │
                    Quality Engine
                          │
                Review / Test / QA
                          │
                     Human Gate
                          │
                        Ship
                          │
                ┌─────────┴─────────┐
                ▼                   ▼
            Run Trace           Memory
                │                   │
                └─────────┬─────────┘
                          ▼
                  Eval / Evolution
```

SpecOS 的长期竞争力来自：

> Artifact Graph + Code Intelligence + Context Compiler + Agent Orchestration + Quality Trace + Project Learning

而不是单个 Agent 的 Prompt。
