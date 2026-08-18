# Spec: Codeg Agent Team & Orchestration Architecture

> Status: Draft / Proposed
> Scope: Codeg enhancement
> Target: Multi-model, multi-agent expert team orchestration
> Date: 2026-08-12

---

# 1. Background

Codeg 当前主要解决的是多个 Coding Agent / CLI 的统一接入，例如：

- Claude Code
- Codex CLI
- Gemini / ACP Agent
- Custom Agent Runtime

现有模型更偏向：

```text
User
  ↓
Codeg
  ↓
Claude CLI / Codex CLI / ACP
```

这种架构适合“选择一个 Agent / CLI 执行任务”，但随着系统能力增强，会逐渐出现以下需求：

- 策划 Agent 使用 GPT 系列模型；
- 实现 Agent 使用 DeepSeek / Claude / 其他 Coding Model；
- Review Agent 使用另外一个高 reasoning 模型；
- 同一个供应商、同一个模型，需要配置多个不同 Agent；
- Agent 之间拥有不同的：
  - System Prompt
  - Role Prompt
  - Skills
  - Knowledge
  - Rules
  - Tools
  - Permissions
  - Reasoning Strength
  - Context Policy
- 多个 Agent 组成一个专业团队；
- Agent 可以互相委派任务；
- Planner 可以生成任务 DAG；
- 多 Agent 可以并行执行；
- 用户可以在 UI 中查看、暂停、重试、审批、替换 Agent；
- Team、Workflow、Model、Runtime 不应互相强耦合。

因此，Codeg 需要在现有 CLI / ACP Runtime 之上增加一层正式的：

```text
Agent Runtime + Agent Team Orchestrator
```

---

# 2. Goals

本设计的目标是让 Codeg 从：

```text
Multi-CLI Coding GUI
```

升级为：

```text
Multi-Agent Expert Team Runtime
```

核心目标包括：

1. Agent 与 Model 解耦；
2. Agent 与 CLI Runtime 解耦；
3. 支持同一模型创建多个不同 Agent；
4. 支持 Agent 独立绑定 Skill / Knowledge / Rule / Tool；
5. 支持全局级和项目级 Agent；
6. 支持 Team；
7. 支持 Workflow；
8. 支持 Task DAG；
9. 支持 Agent delegation；
10. 支持 Agent-as-Tool；
11. 支持 handoff；
12. 支持 parallel execution；
13. 支持 runtime model override；
14. 支持 fallback / retry / budget；
15. 最大程度复用 Codeg 当前：
    - Session
    - ACP
    - Delegation
    - Skill
    - Worktree
    - Streaming UI
    - CLI Adapter

---

# 3. Non-Goals

第一阶段不追求：

- 完全自主的无限循环 Agent；
- 高度复杂的 AI 自主组织结构；
- 动态生成无限数量 Agent；
- 完全自动的企业级 BPM Workflow；
- 引入 LangGraph / CrewAI 作为核心 Runtime；
- 将所有 CLI Provider 重写为统一 API；
- 重新开发 Codeg 已存在的 Session / Worktree / Streaming 系统。

核心原则：

> 尽量基于 Codeg 现有能力增量实现，而不是重做 Agent Runtime。

---

# 4. Core Design Principle

## 4.1 Agent != Model

Agent 是逻辑身份。

例如：

```text
Planner Agent
Reviewer Agent
Frontend Agent
Architecture Agent
```

Model 只是 Agent 的一个配置依赖。

例如：

```text
Planner Agent
  ↓
GPT-X / High Reasoning
```

```text
Reviewer Agent
  ↓
GPT-X / XHigh Reasoning
```

即使使用同一个供应商和同一个 Model，它们仍然是不同 Agent。

---

## 4.2 Agent != CLI

CLI 只是 Runtime Adapter。

例如：

```text
Planner Agent
  ↓
Codex CLI
  ↓
OpenAI Model
```

也可以：

```text
Planner Agent
  ↓
Direct API
  ↓
OpenAI Model
```

或者：

```text
Frontend Agent
  ↓
ACP Runtime
  ↓
DeepSeek
```

Agent 不应绑定到具体 CLI。

---

## 4.3 Team != Workflow

Team 表示：

```text
有哪些专家
```

Workflow 表示：

```text
这些专家如何协作
```

同一个 Team 可以运行多个 Workflow。

---

# 5. Overall Architecture

```text
┌──────────────────────────────────────────────────────┐
│                     Application UI                   │
│                                                      │
│ Chat / Agent Mode / Team Mode / Orchestration Mode  │
└─────────────────────────┬────────────────────────────┘
                          │
┌─────────────────────────▼────────────────────────────┐
│                     Team Layer                       │
│                                                      │
│ Engineering Team / Product Team / Review Team       │
└─────────────────────────┬────────────────────────────┘
                          │
┌─────────────────────────▼────────────────────────────┐
│                   Workflow Layer                     │
│                                                      │
│ DAG / Supervisor / Handoff / Parallel / Review      │
└─────────────────────────┬────────────────────────────┘
                          │
┌─────────────────────────▼────────────────────────────┐
│                   Agent Profile                      │
│                                                      │
│ Role                                                 │
│ Prompt                                               │
│ Model Profile                                        │
│ Skills                                               │
│ Knowledge                                            │
│ Rules                                                │
│ Tools                                                │
│ Permissions                                          │
└─────────────────────────┬────────────────────────────┘
                          │
┌─────────────────────────▼────────────────────────────┐
│                    Agent Runtime                     │
│                                                      │
│ Agent Instance                                       │
│ Session                                              │
│ Context                                              │
│ Task                                                 │
│ Worktree                                             │
│ Budget                                               │
│ State                                                │
└─────────────────────────┬────────────────────────────┘
                          │
┌─────────────────────────▼────────────────────────────┐
│                   Runtime Adapter                    │
│                                                      │
│ Codex CLI                                            │
│ Claude Code                                          │
│ ACP                                                  │
│ Custom CLI                                           │
│ Direct API                                           │
└─────────────────────────┬────────────────────────────┘
                          │
┌─────────────────────────▼────────────────────────────┐
│                     Model Layer                      │
│                                                      │
│ OpenAI / Anthropic / DeepSeek / Kimi / GLM / etc.   │
└──────────────────────────────────────────────────────┘
```

---

# 6. Domain Model

建议核心 Domain Entity：

```text
Provider
ModelProfile
AgentProfile
AgentInstance
Team
Workflow
Task
Run
```

附属能力：

```text
Skill
Knowledge
Rule
Tool
MCP
Prompt
PermissionPolicy
ContextPolicy
RuntimePolicy
```

整体关系：

```text
Provider
   ↓
ModelProfile
   ↓
AgentProfile
   ↓
AgentInstance
   ↓
Task / Run
```

同时：

```text
Team
  ↓
AgentProfile[]
```

以及：

```text
Workflow
  ↓
Task Graph
  ↓
AgentProfile
```

---

# 7. Model Profile

Model Profile 负责描述模型运行参数。

不应在每个 Agent 中重复配置：

- API Base
- API Key
- Provider
- Model
- Reasoning
- Temperature
- Timeout
- Retry
- Fallback

推荐结构：

```yaml
models:

  planner-premium:
    provider: openai
    model: gpt-x
    reasoning_effort: high
    timeout: 600

  planner-fast:
    provider: openai
    model: gpt-x
    reasoning_effort: low
    timeout: 180

  coding-deepseek:
    provider: deepseek
    model: deepseek-coder
    temperature: 0.2

  reviewer-premium:
    provider: anthropic
    model: claude-x
    thinking:
      enabled: true
      budget: 32000
```

Agent 使用：

```yaml
model_ref: planner-premium
```

---

# 8. Agent Profile

Agent Profile 是整个系统最核心的配置实体。

建议：

```text
AgentProfile
├── Identity
├── Runtime
├── Model
├── Prompt
├── Skills
├── Knowledge
├── Rules
├── Tools
├── Permissions
├── Context Policy
├── Delegation Policy
└── Execution Policy
```

---

# 9. Agent Profile Schema

示例：

```yaml
id: planner
name: Planning Expert
description: Product planning and technical decomposition agent

role:
  type: planner
  description: >
    Responsible for requirement analysis,
    task decomposition and implementation planning.

runtime:
  type: codex-cli

model:
  ref: planner-premium

prompt:
  system: ./prompts/planner.md

skills:
  - brainstorming
  - spec-writing
  - architecture
  - task-decomposition

knowledge:
  - project-wiki
  - architecture-docs
  - product-requirements

rules:
  - project-conventions
  - planning-policy

tools:
  allow:
    - repo-read
    - search
    - git-read

permissions:
  filesystem:
    read: true
    write: false
  terminal:
    enabled: false

delegation:
  enabled: true
  allowed_agents:
    - architecture
    - researcher
    - reviewer
  max_depth: 2

execution:
  timeout: 900
  max_turns: 40
  retry: 1
```

---

# 10. Implementation Agent Example

```yaml
id: frontend-implementer
name: Frontend Engineer

role:
  type: implementer

runtime:
  type: acp
  adapter: deepseek-agent

model:
  ref: coding-deepseek

skills:
  - frontend-design
  - react
  - typescript
  - test-driven-development

knowledge:
  - frontend-architecture
  - component-library

rules:
  - frontend-conventions

tools:
  allow:
    - filesystem
    - terminal
    - git
    - tests

permissions:
  filesystem:
    read: true
    write: true

execution:
  worktree: true
  timeout: 1800
  retry: 2
```

---

# 11. Same Model, Different Agents

以下必须被视为不同 Agent：

```yaml
id: architecture-reviewer

model:
  ref: gpt-high

skills:
  - architecture-review
  - ddd-review

reasoning:
  effort: high
```

```yaml
id: security-reviewer

model:
  ref: gpt-high

skills:
  - security-review
  - dependency-audit

reasoning:
  effort: high
```

虽然底层 Model 完全相同，但是：

```text
Prompt
Skill
Knowledge
Rule
Tool
Permission
Context
```

全部不同。

因此：

> Agent Identity 不应由 Model 唯一决定。

---

# 12. Skill Architecture

Skill 属于 Agent，而不是属于 Model。

错误模型：

```text
GPT
 ├─ Skill A
 └─ Skill B
```

正确模型：

```text
Agent
 ├─ Skill A
 ├─ Skill B
 └─ Skill C
```

同一个 Model 可以被不同 Agent 以完全不同的 Skill 使用。

---

# 13. Knowledge Architecture

建议 Knowledge 单独抽象。

Agent Intelligence 最终由：

```text
Prompt
+
Skill
+
Knowledge
+
Rules
+
Context
```

共同组成。

推荐：

```text
Agent Profile
│
├── Prompt
├── Skills
├── Knowledge
├── Rules
└── Context Policy
```

Knowledge 可来自：

```text
Project Wiki
Memory
Codebase Index
Architecture Docs
Product Docs
ADR
External Docs
Vector Index
Graph Knowledge
```

---

# 14. Rule Architecture

Rule 与 Skill 分开。

Skill 更偏：

```text
How to do
```

Rule 更偏：

```text
Must / Must Not
```

例如：

```yaml
rules:
  - no-code-before-spec
  - always-run-tests
  - project-style-guide
  - no-schema-change-without-migration
```

---

# 15. Tool Policy

每个 Agent 可以拥有独立工具权限。

例如 Planner：

```yaml
tools:
  allow:
    - repo-read
    - search
    - docs
```

Frontend：

```yaml
tools:
  allow:
    - repo-read
    - repo-write
    - terminal
    - tests
    - browser
```

Security Reviewer：

```yaml
tools:
  allow:
    - repo-read
    - dependency-scanner
    - git-diff
```

---

# 16. Permission Policy

建议 Tool 与 Permission 分离。

Tool：

```text
Agent 是否拥有某项能力
```

Permission：

```text
Agent 能否执行危险操作
```

例如：

```yaml
permissions:

  filesystem:
    read: true
    write: true

  terminal:
    enabled: true
    dangerous_commands: false

  git:
    commit: false
    push: false

  network:
    outbound: true
```

---

# 17. Runtime Adapter

推荐定义统一 Runtime Adapter Interface。

```ts
interface AgentRuntimeAdapter {
  start(config: ResolvedAgentRuntime): Promise<AgentSession>;
  send(sessionId: string, message: AgentInput): Promise<void>;
  cancel(sessionId: string): Promise<void>;
  resume(sessionId: string): Promise<void>;
  dispose(sessionId: string): Promise<void>;
}
```

支持：

```text
CodexRuntimeAdapter
ClaudeRuntimeAdapter
ACPRuntimeAdapter
DirectApiRuntimeAdapter
CustomCliRuntimeAdapter
```

---

# 18. Resolved Agent Runtime

AgentProfile 不直接执行。

启动时：

```text
AgentProfile
   ↓
AgentRuntimeResolver
   ↓
ResolvedAgentRuntime
```

最终配置可能为：

```json
{
  "agent_id": "planner",
  "runtime": "codex-cli",
  "provider": "openai",
  "model": "gpt-x",
  "reasoning_effort": "high",
  "skills": [
    "planning",
    "architecture"
  ],
  "knowledge": [
    "project-wiki"
  ],
  "permissions": {
    "filesystem_write": false
  }
}
```

---

# 19. Agent Profile vs Agent Instance

必须区分：

```text
AgentProfile
```

和：

```text
AgentInstance
```

AgentProfile 是模板。

AgentInstance 是某次任务实际运行出来的实例。

例如：

```text
frontend-agent
```

是 Profile。

用户同时要求：

```text
实现用户模块
实现订单模块
实现支付模块
```

可以生成：

```text
frontend-agent#001
frontend-agent#002
frontend-agent#003
```

它们拥有：

```text
相同 Profile
不同 Session
不同 Task
不同 Worktree
不同 Context
不同 State
```

---

# 20. Agent Instance

建议：

```text
AgentInstance
├── instance_id
├── profile_id
├── session_id
├── task_id
├── runtime_id
├── worktree
├── context_snapshot
├── status
├── token_usage
├── cost
├── started_at
└── finished_at
```

状态：

```text
created
queued
running
waiting
blocked
paused
completed
failed
cancelled
```

---

# 21. Team

Team 表示一组可协作 Agent。

示例：

```yaml
id: product-engineering-team
name: Product Engineering Team

coordinator:
  agent: planner

members:

  - agent: planner
    role: lead

  - agent: architecture
    role: specialist

  - agent: frontend
    role: worker

  - agent: backend
    role: worker

  - agent: tester
    role: verifier

  - agent: reviewer
    role: reviewer
```

---

# 22. Team Is An Expert Pool

Team 不应该强制固定执行顺序。

Team 本质是：

```text
Expert Pool
```

例如：

```text
Product Engineering Team

Planner
Architect
Frontend
Backend
Database
Testing
Reviewer
Security
```

针对不同 Task 可以调用不同成员。

---

# 23. Workflow

Workflow 描述 Team 本次如何工作。

例如 Feature Workflow：

```text
Planner
   ↓
Architect
   ↓
┌───────────────┐
Frontend     Backend
└───────┬───────┘
        ↓
      Tester
        ↓
     Reviewer
```

Bug Workflow：

```text
Debugger
   ↓
Implementer
   ↓
Tester
   ↓
Reviewer
```

Spec Workflow：

```text
Idea Agent
   ↓
Planner
   ↓
Architect
   ↓
Spec Reviewer
```

---

# 24. Workflow Schema

```yaml
id: feature-development

team: product-engineering-team

steps:

  - id: plan
    agent: planner

  - id: architecture
    agent: architecture
    depends_on:
      - plan

  - id: frontend
    agent: frontend
    depends_on:
      - architecture

  - id: backend
    agent: backend
    depends_on:
      - architecture

  - id: test
    agent: tester
    depends_on:
      - frontend
      - backend

  - id: review
    agent: reviewer
    depends_on:
      - test
```

---

# 25. Task DAG

第一阶段建议 Team Orchestrator 以 DAG 为核心。

Planner 可以生成：

```json
{
  "tasks": [
    {
      "id": "T1",
      "agent": "architecture",
      "description": "Design module architecture"
    },
    {
      "id": "T2",
      "agent": "frontend",
      "description": "Implement frontend",
      "depends_on": ["T1"]
    },
    {
      "id": "T3",
      "agent": "backend",
      "description": "Implement backend",
      "depends_on": ["T1"]
    },
    {
      "id": "T4",
      "agent": "reviewer",
      "description": "Review implementation",
      "depends_on": ["T2", "T3"]
    }
  ]
}
```

Orchestrator 负责：

```text
dependency
queue
parallel
retry
state
cancel
resume
result
```

---

# 26. Orchestration Modes

建议至少支持：

## Sequential

```text
A → B → C
```

## Parallel

```text
      ┌→ B
A ────┤
      └→ C
```

## Supervisor

```text
          Supervisor
         /    |     \
        A     B      C
         \    |     /
          Supervisor
```

## Handoff

```text
Agent A
   ↓
transfer control
   ↓
Agent B
```

## Agent-as-Tool

```text
Main Agent
   ↓
call Specialist
   ↓
result returned
   ↓
Main Agent continues
```

## Review Loop

```text
Implementer
    ↓
Reviewer
    ↓
Rejected?
    ↓ yes
Implementer
```

---

# 27. Delegation

现有 Codeg delegation 能力应升级，而不是推翻。

建议演进：

```text
DelegationBroker
      ↓
AgentRuntimeManager
      +
TeamOrchestrator
```

Delegation Policy：

```yaml
delegation:
  enabled: true
  max_depth: 2

  allowed_agents:
    - frontend
    - backend
    - reviewer

  max_parallel: 4

  require_approval:
    - security
    - production-deploy
```

---

# 28. Agent-as-Tool

Agent 应允许被其他 Agent 当作 Tool 调用。

例如 Planner：

```text
Planner
  ↓
call architecture-agent
  ↓
architecture result
  ↓
Planner
```

与 Handoff 不同：

Agent-as-Tool 中：

```text
主 Agent 保持控制权
```

---

# 29. Handoff

Handoff 表示：

```text
控制权从 Agent A 转移给 Agent B
```

例如：

```text
Idea Agent
   ↓
需求已经明确
   ↓
handoff → Planner
```

或者：

```text
Planner
   ↓
进入实施阶段
   ↓
handoff → Implementation Coordinator
```

---

# 30. Context Policy

每个 Agent 应可以配置 Context Policy。

例如：

```yaml
context:

  include:
    - task
    - relevant-files
    - project-rules

  memory:
    enabled: true

  wiki:
    enabled: true

  history:
    mode: summarized

  max_tokens: 100000
```

不同 Agent 不需要收到完全相同的 Context。

---

# 31. Context Isolation

推荐默认：

```text
Agent Instance = Independent Context
```

Agent 之间通过：

```text
Task Input
Artifact
Summary
Handoff Message
Shared Memory
```

交流。

不要默认将整个 Session History 全量共享给所有 Agent。

原因：

- Token 成本过高；
- Agent 容易被无关信息污染；
- 不方便调试；
- 不方便复现；
- Agent specialization 会下降。

---

# 32. Runtime Overrides

运行时允许用户覆盖 Profile。

例如：

```text
Planner
Default: GPT-X High
```

用户本次任务临时选择：

```text
GPT-X XHigh
```

覆盖规则：

```text
Global Default
   ↓
Project Override
   ↓
Agent Profile
   ↓
Workflow Override
   ↓
Run Override
```

后者优先级更高。

---

# 33. Config Resolution

推荐最终优先级：

```text
System Default
  <
User Global
  <
Project Config
  <
Agent Profile
  <
Team Config
  <
Workflow Config
  <
Task Config
  <
Run Override
```

---

# 34. Fallback

ModelProfile 可以配置 fallback：

```yaml
model:
  primary: planner-premium

  fallback:
    - planner-fast
    - claude-planner
```

触发条件：

```text
timeout
rate_limit
provider_error
quality_reject
budget_limit
manual_switch
```

---

# 35. Retry

Retry 分两层：

```text
Model Retry
Agent Retry
```

Model Retry：

```text
API / CLI 临时失败
```

Agent Retry：

```text
整个 Task 结果不合格
```

建议避免混为一谈。

---

# 36. Review Mechanism

Task 可以配置 Review Policy：

```yaml
review:
  required: true
  reviewers:
    - code-reviewer

  retry_on_reject: true
  max_rounds: 2
```

---

# 37. Multi-Model Review

支持：

```text
Implementer
  ↓
DeepSeek
```

完成后：

```text
Reviewer
  ↓
GPT / Claude
```

即：

```text
生成模型
!=
Review 模型
```

这是专业团队能力的重要价值。

---

# 38. Agent Registry

建议新增：

```text
AgentRegistry
```

负责：

```text
load
resolve
validate
list
enable
disable
override
```

配置目录推荐：

```text
.speco/
  agents/
    manifest.yaml

    roles/
      planner.yaml
      architecture.yaml
      frontend.yaml
      reviewer.yaml

    prompts/
      planner.md
      architecture.md
      reviewer.md
```

也可以兼容：

```text
.agents/
```

最终目录名可根据 Codeg 当前结构决定。

---

# 39. Global vs Project Agent

需要两级：

```text
Global Agent
Project Agent
```

Global：

```text
~/.codeg/agents/
```

Project：

```text
<project>/.agents/
```

Project Agent 可以：

- 新增；
- 覆盖；
- 禁用 Global Agent；
- 修改 Model；
- 添加项目 Skill；
- 绑定项目 Knowledge。

---

# 40. Agent Package

未来可以抽象 Agent Package：

```text
Expert Package
 =
 Role Prompt
 + Skills
 + Rules
 + Tool Policy
 + Knowledge references
 + Model Recommendation
```

例如：

```text
Senior React Architect
Security Reviewer
DDD Architect
Database Migration Expert
Test Engineer
```

Model Recommendation 只作为默认推荐，不作为强绑定。

---

# 41. Team Package

同样可以支持：

```text
Team Package
```

例如：

```text
Fullstack Engineering Team
Product Planning Team
Architecture Review Board
Release Team
QA Team
```

其中包含：

```text
Agents
Workflow Templates
Rules
Default Model Suggestions
```

---

# 42. Codeg Integration

本架构应优先复用 Codeg 当前组件：

```text
Existing Codeg
├── CLI Adapter
├── ACP
├── Session
├── Delegation
├── Skill
├── Worktree
├── Streaming
├── Chat UI
└── Agent Execution
```

新增：

```text
New Layer
├── ModelProfile
├── AgentProfile
├── AgentRegistry
├── AgentRuntimeResolver
├── AgentInstance
├── Team
├── Workflow
├── TaskGraph
└── TeamOrchestrator
```

---

# 43. Recommended Runtime Flow

```text
User Request
    ↓
Select Team / Agent / Mode
    ↓
Load AgentProfile
    ↓
Resolve ModelProfile
    ↓
Resolve Skills / Knowledge / Rules
    ↓
Resolve Tool / Permission Policy
    ↓
Create AgentInstance
    ↓
Create / Reuse Codeg Session
    ↓
Select Runtime Adapter
    ↓
Start CLI / ACP / API
    ↓
Stream Result
    ↓
Persist Task / Run State
```

---

# 44. Team Runtime Flow

```text
User Request
    ↓
TeamOrchestrator
    ↓
Planner
    ↓
TaskGraph
    ↓
Scheduler
    ↓
AgentInstance A
AgentInstance B
AgentInstance C
    ↓
Artifacts / Results
    ↓
Reviewer
    ↓
Final Result
```

---

# 45. Orchestration UI

后续可以提供：

```text
Agent Team Mode
```

或者：

```text
Orchestration Mode
```

UI 类似无限画布 / DAG Canvas。

示例：

```text
                         Requirement
                              │
                              ▼
                           Planner
                     GPT-X · High
                              │
                         Architecture
                              │
                    ┌─────────┴─────────┐
                    ▼                   ▼
                 Frontend             Backend
                 DeepSeek             Claude
                    │                   │
                    └─────────┬─────────┘
                              ▼
                           Tester
                              │
                              ▼
                          Reviewer
                        GPT-X · XHigh
```

---

# 46. UI Node Information

每个 Agent Node 可以显示：

```text
Agent Name
Role
Model
Provider
Reasoning
Status
Task
Duration
Tokens
Cost
Worktree
Session
```

状态：

```text
Waiting
Running
Completed
Failed
Paused
Blocked
Needs Approval
```

---

# 47. UI Operations

用户可以：

```text
Run
Pause
Resume
Cancel
Retry
Approve
Reject
Replace Agent
Change Model
Change Reasoning
Inspect Context
Inspect Prompt
Inspect Skill
Inspect Tool Calls
Open Session
Open Worktree
View Diff
Branch Task
```

---

# 48. Human-in-the-loop

Task 可以设置：

```yaml
approval:
  before_run: false
  after_run: true
```

或者：

```yaml
approval:
  required_for:
    - production-deploy
    - database-migration
    - destructive-operation
```

---

# 49. Artifacts

Agent 输出不应只是一段 Chat Message。

建议支持标准 Artifact：

```text
Plan
Spec
Patch
Code Diff
Test Report
Review Report
Architecture Document
Task Result
Decision
```

Task 之间应优先传递 Artifact，而不是完整 Chat History。

---

# 50. Shared Memory

Team 可以共享 Memory，但要区分：

```text
Agent Private Memory
Project Memory
Team Shared Memory
Task Memory
```

推荐：

```text
Agent Private
      ↓
Agent specialization

Project Shared
      ↓
Project conventions

Team Shared
      ↓
Current collaboration

Task Shared
      ↓
Current execution
```

---

# 51. MVP Scope

第一阶段建议只实现：

## Configuration

- ModelProfile
- AgentProfile
- AgentRegistry

## Runtime

- AgentRuntimeResolver
- AgentInstance

## Team

- Static Team
- Static Workflow
- Task DAG

## Execution

- Sequential
- Parallel
- Dependency
- Retry
- Cancel

## UI

- Agent list
- Agent configuration
- Team configuration
- Task state
- Basic DAG

---

# 52. Phase 2

第二阶段：

- Planner-generated DAG
- Agent-as-Tool
- Handoff
- Review Loop
- Model fallback
- Runtime override
- Cost / Token tracking
- Context Inspector
- Agent replacement
- Human approval
- Team Canvas

---

# 53. Phase 3

第三阶段：

- Dynamic Agent selection
- Dynamic Team construction
- Agent Marketplace
- Team Marketplace
- Auto Skill injection
- Memory-assisted Agent
- Historical quality score
- Model routing
- Cost / quality optimizer
- Auto retry using alternate Agent
- Evolution / Skill extraction

---

# 54. Recommended Internal Modules

建议代码模块：

```text
src/
  agent/
    profile/
    registry/
    resolver/
    instance/
    context/
    permission/

  model/
    provider/
    profile/
    router/

  team/
    team/
    workflow/
    orchestrator/
    scheduler/

  task/
    graph/
    state/
    artifact/

  runtime/
    adapter/
    codex/
    claude/
    acp/
    api/
```

---

# 55. Suggested Interfaces

## AgentProfile

```ts
interface AgentProfile {
  id: string;
  name: string;
  description?: string;

  runtime: RuntimeConfig;
  model: ModelRef;

  prompt?: PromptConfig;

  skills: string[];
  knowledge: string[];
  rules: string[];

  tools: ToolPolicy;
  permissions: PermissionPolicy;

  context?: ContextPolicy;
  delegation?: DelegationPolicy;
  execution?: ExecutionPolicy;
}
```

---

# 56. ModelProfile

```ts
interface ModelProfile {
  id: string;

  provider: string;
  model: string;

  reasoning?: {
    effort?: string;
    budget?: number;
  };

  temperature?: number;

  timeout?: number;

  fallback?: string[];
}
```

---

# 57. AgentInstance

```ts
interface AgentInstance {
  id: string;
  profileId: string;

  taskId?: string;
  sessionId?: string;

  status: AgentStatus;

  runtime: ResolvedAgentRuntime;

  worktree?: string;

  startedAt?: number;
  finishedAt?: number;
}
```

---

# 58. Team

```ts
interface AgentTeam {
  id: string;
  name: string;

  coordinator?: string;

  members: TeamMember[];

  defaults?: TeamRuntimePolicy;
}
```

---

# 59. Workflow

```ts
interface AgentWorkflow {
  id: string;
  name: string;

  teamId: string;

  steps: WorkflowStep[];
}
```

---

# 60. Task

```ts
interface AgentTask {
  id: string;

  title: string;
  description: string;

  agentId?: string;

  dependsOn: string[];

  status: TaskStatus;

  inputArtifacts: string[];
  outputArtifacts: string[];

  retry?: number;
}
```

---

# 61. Orchestrator Responsibility

TeamOrchestrator 负责：

```text
Task decomposition
Agent selection
Dependency management
Parallel scheduling
Run state
Retry
Fallback
Handoff
Review
Approval
Artifact routing
Result aggregation
```

但不负责：

```text
实际与 Claude/Codex CLI 通讯
```

该职责继续由 Runtime Adapter / Codeg Runtime 完成。

---

# 62. Important Boundary

必须保持：

```text
TeamOrchestrator
       ↓
AgentRuntimeManager
       ↓
RuntimeAdapter
       ↓
Codex / Claude / ACP / API
```

不要：

```text
TeamOrchestrator
       ↓
直接调用 OpenAI API
```

否则 CLI / ACP abstraction 会被破坏。

---

# 63. Why Not Embed CrewAI / LangGraph Directly

不建议将 CrewAI / LangGraph 作为 Codeg 的核心 Runtime。

原因：

1. Codeg 已经拥有 Agent Session；
2. Codeg 已经拥有 CLI Runtime；
3. Codeg 已经拥有 ACP；
4. Codeg 已经拥有 Delegation；
5. Codeg 已经拥有 Skill；
6. Codeg 已经拥有 Worktree；
7. Codeg 已经拥有 Streaming UI。

如果直接嵌入其它 Agent Framework：

```text
Codeg Runtime
+
External Agent Runtime
```

会出现双重：

```text
Session
State
Task
Retry
Tool
Streaming
Context
```

导致复杂度显著增加。

推荐：

```text
借鉴抽象
不引入核心 Runtime
```

---

# 64. External Architecture References

可以重点参考以下思想：

```text
CrewAI
→ Role / Crew / Specialist

LangGraph
→ State Graph / DAG / Routing

OpenAI Agents
→ Agent-as-Tool / Handoff

Microsoft Agent Framework
→ Agent + Workflow

AutoGen
→ Multi-Agent Conversation

Claude / Codex
→ Agent Runtime / Tool execution
```

最终实现：

```text
Codeg-native Agent Orchestrator
```

---

# 65. Final Architecture

最终推荐结构：

```text
                     Speco / Codeg

                         USER
                           │
                           ▼
                  ┌────────────────┐
                  │     MODE       │
                  │                │
                  │ Chat           │
                  │ Agent          │
                  │ Team           │
                  │ Orchestration  │
                  └───────┬────────┘
                          │
                          ▼
                 ┌─────────────────┐
                 │ TeamOrchestrator│
                 └───────┬─────────┘
                         │
                 ┌───────▼─────────┐
                 │    Workflow     │
                 │   / Task DAG    │
                 └───────┬─────────┘
                         │
           ┌─────────────┼─────────────┐
           ▼             ▼             ▼
       Planner       Implementer    Reviewer
           │             │             │
           ▼             ▼             ▼
      AgentProfile  AgentProfile   AgentProfile
           │             │             │
           ▼             ▼             ▼
       ModelProfile  ModelProfile   ModelProfile
           │             │             │
           ▼             ▼             ▼
     GPT / Claude    DeepSeek       GPT / Claude
           │             │             │
           ▼             ▼             ▼
      Codex CLI        ACP         Claude CLI
```

---

# 66. Final Core Principle

整个系统最关键的关系：

```text
Team
contains
AgentProfile[]
```

```text
AgentProfile
references
ModelProfile
+
Skills
+
Knowledge
+
Rules
+
Tools
+
Permissions
```

```text
AgentProfile
on execution
spawns
AgentInstance
```

```text
AgentInstance
uses
RuntimeAdapter
```

```text
RuntimeAdapter
connects
CLI / ACP / API
```

因此：

> Agent 是专家身份，Model 是脑子，Skill 是能力，Knowledge 是知识，Tool 是手，Permission 是权限，Runtime Adapter 是执行通道，Team 是专家集合，Workflow 是协作方式，Orchestrator 是调度中枢。

---

# 67. Recommended Product Positioning

从产品层来看，该能力可以定义为：

```text
Agent Team
```

底层模块命名建议：

```text
Agent Runtime
Agent Registry
Team Orchestrator
Workflow Engine
```

UI Mode 可命名：

```text
Team Mode
```

或：

```text
Orchestration Mode
```

如果未来采用无限画布交互，可以进一步定义为：

```text
Agent Canvas
```

---

# 68. Recommended First Implementation Order

建议实现顺序：

```text
1. ModelProfile
2. AgentProfile
3. AgentRegistry
4. AgentRuntimeResolver
5. AgentInstance
6. Static Team
7. Workflow
8. Task DAG
9. Scheduler
10. Parallel Execution
11. Review
12. Handoff / Agent-as-Tool
13. Orchestration UI
14. Dynamic Orchestration
```

---

# 69. Architecture Decision

建议正式确定以下架构决策：

### ADR-1

Agent 与 Model 解耦。

### ADR-2

Agent 与 CLI Runtime 解耦。

### ADR-3

Skill / Knowledge / Rule / Tool 独立建模。

### ADR-4

Team 与 Workflow 解耦。

### ADR-5

AgentProfile 与 AgentInstance 解耦。

### ADR-6

TeamOrchestrator 不直接调用模型 Provider。

### ADR-7

优先复用 Codeg Session / ACP / Delegation / Worktree。

### ADR-8

第一阶段以显式 Workflow / Task DAG 为主，不追求完全自主 Agent Team。

---

# 70. Conclusion

Codeg 当前已经具备较好的 Agent Runtime 底座。

下一阶段不需要重新开发一套 Multi-Agent Framework。

最合理的演进路径是：

```text
Codeg
+
AgentProfile
+
ModelProfile
+
AgentRuntimeResolver
+
Team
+
Workflow
+
Task DAG
+
TeamOrchestrator
```

形成：

```text
Codeg-native Multi-Agent Expert Team Platform
```

它既可以继续保持：

```text
Codex App / Claude Code GUI
```

这种简单使用体验，

同时又可以在高级模式下提供：

```text
Planner
+
Architecture
+
Implementation
+
Testing
+
Review
+
Human Approval
```

组成真正的专业 Agent Team。

这套结构也能够与后续的：

- Project Memory
- Auto Skill Evolution
- Wiki
- CodeGraph
- Spec Mode
- Plan Mode
- Idea Agent
- Agent Canvas
- Multi-Model Router

自然整合，而不需要重新调整核心架构。
