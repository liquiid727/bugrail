# AGENT — Loadout、Scope、权限与多模型协同

# 1. Entity Model

```text
User
 └─ Workspace
     └─ Project
         ├─ Agents
         ├─ Tasks
         ├─ Sessions
         └─ Assets

Team
 ├─ Users
 ├─ Agents
 └─ Shared Assets
```

---

# 2. Agent Profile

```yaml
id: implementation-agent
name: Implementation Agent

runtime:
  adapter: codex
  provider: openai
  model: ...
  reasoning: high

prompt:
  roleFile: ...
  instructions: ...

loadout:
  id: impl-default

permissions:
  memoryRead: true
  memoryWrite: true
  skillExecute: true
  wikiRead: true
  codeGraphRead: true
```

模型、CLI、Skill、Knowledge 是独立配置维度。

---

# 3. 同供应商不同 Agent

支持：

```text
Agent A:
  provider=openai
  model=X
  reasoning=high
  skills=architecture

Agent B:
  provider=openai
  model=X
  reasoning=medium
  skills=review
```

Agent identity 不能等同 model identity。

---

# 4. Loadout

```yaml
id: architecture-loadout

memory:
  scopes:
    - user
    - project
  layers:
    - L3
    - L2
    - L1
  maxTokens: 2500

skills:
  collections:
    - architecture

wiki:
  sources:
    - project-docs
    - adr

codegraph:
  enabled: true
  defaultQueries:
    - module-neighborhood

taskMemory:
  enabled: true
```

---

# 5. Loadout inheritance

```text
System Default
   ↓
Workspace Default
   ↓
Project Default
   ↓
Agent Profile
   ↓
Task Override
   ↓
Session Override
```

后层只覆盖显式字段。

---

# 6. Scope

```text
private   — only owner
project   — project members/agents
team      — team
restricted— ACL
agent     — explicitly bound agents
task      — task only
```

---

# 7. ACL

```ts
ACL {
  users[]
  roles[]
  agents[]
  permissions:
    read
    write
    manage
    execute
}
```

Skill 的 `execute` 与 `read` 分开。

---

# 8. Personal vs Project Memory

例：

```text
"I prefer concise plans"
→ user private

"This repo uses pnpm"
→ project

"review-agent should be strict"
→ agent config, not memory
```

分类器不确定时默认更窄 scope。

---

# 9. Agent Memory

每个 Agent 可以拥有专属 Scenario：

```text
review-agent has repeatedly found migration rollback bugs
```

但不要把 agent-produced hallucination 自动升级为 project fact。

---

# 10. Multi-Agent Task

Task 有共享 context：

```text
Task Canvas
Task decisions
Task artifacts
```

不同 Agent 可以读。

写入 Chat Memory 时必须注明来源：

```text
originAgentId
```

---

# 11. Planner → Implementer → Reviewer

典型：

```text
Planner
  ↓ plan artifact
Implementer
  ↓ code + trace
Reviewer
  ↓ findings
Test Agent
  ↓ validation
```

Task Memory 聚合全部角色状态。

长期 Memory 只沉淀最终确认信息。

---

# 12. Trust Level

不同来源可信度：

```text
user explicit statement       1.0
accepted project rule         1.0
merged code evidence          0.95
accepted ADR                  0.95
agent inferred memory         0.65
unverified task hypothesis    0.4
```

Recall rerank 可用。

---

# 13. Model Routing 与 Memory

Memory 不决定模型。

Agent Orchestrator 决定：

```text
role → provider/model profile
```

Memory Context Router 只根据当前 role/loadout 准备 context。

---

# 14. A/B Agent

用户可：

```text
Run with Agent A
Run with Agent B
```

两者读取同 project evidence，但可使用不同 Persona/Skill/Knowledge budget。

结果比较不应分别永久写入冲突 memory，直到用户选择/任务完成。

---

# 15. Review Memory Policy

Reviewer 产生：

```text
finding
```

只有以下才长期沉淀：

- 被确认的问题模式
- recurring defect
- explicit team rule

不是每条 review comment 都进 L1。

---

# 16. 用户 UI

Agents 页面：

```text
Agents
 ├─ Architecture Agent
 ├─ Implementation Agent
 ├─ Reviewer
 └─ Test Agent
```

Agent detail：

```text
Runtime
Model
Reasoning
Prompt
Loadout
Memory access
Skills
Knowledge
Permissions
Metrics
```

---

# 17. Acceptance

- [ ] Agent identity 独立于 model
- [ ] 同模型可创建多个 Agent profile
- [ ] Agent loadout 独立
- [ ] Project assets 默认不跨项目
- [ ] private memory 不被 team agent 读取
- [ ] task context 可跨 agent
- [ ] unconfirmed agent hypothesis 不升级为 project fact
- [ ] A/B 结果不污染长期 memory

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
