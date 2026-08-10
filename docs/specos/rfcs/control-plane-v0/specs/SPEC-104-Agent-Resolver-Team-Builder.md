# SPEC-104 — Agent Resolver / Team Builder

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §8（D5）；`02` P4；`03` 里程碑 M0（L1）/ M1（L2）/ M2（L3）
> 依赖：SPEC-010（权限）、SPEC-011（Registry）
> 实现语言：不限定；接口契约为本 SPEC 主体

---

# 1. 目的与范围

定义「任务 → 能力 → Agent Profile → Team」的解析与组队插件。

---

# 2. 接口（契约）

```ts
interface CapabilityResolver {
  resolve(issue: Issue): Promise<Capabilities>
}

interface AgentResolver {
  resolve(capabilities: Capabilities, opts?: ResolveOpts): Promise<AgentProfile[]>
  explain(issueId): Promise<RouteExplanation>
}

interface TeamBuilder {
  build(dag: Dag, opts?: TeamOpts): Promise<Team>
}
```

```ts
interface Capabilities {
  required: string[]
  optional: string[]
  verification: string[]
  review: string[]
}
```

---

# 3. Capability Resolver 输入

```text
Title
Goal
Scope
Acceptance Criteria
Impacted modules（SPEC-102 L2 起）
Risk
```

示例：`「增加支付退款导出 Excel」` → `backend / payment / export / frontend / permission / test`。

---

# 4. Agent Resolver 评分（L2）

```text
domain exact match       +50
required capability      +20
preferred by project     +15
related skill            +10
permission compatible    required
model available          required
```

## L3 — Eval 感知

结合历史成功率 / 成本 / 延迟（SPEC-113），支持优化目标：Fast / Balanced / Quality / Budget。

---

# 5. Team Builder

- 简单任务可单 Agent。
- 复杂任务生成动态 Team：

```text
architecture → [backend, frontend] → integration → security-review → test
```

- 每个成员可优化：quality/speed/cost 权重。

---

# 6. 可解释路由

```yaml
route_explanation:
  issue: ISSUE-101
  capabilities: [backend, payment]
  agent_candidates:
    - backend-payment-agent (score: 95)
    - backend-agent (score: 70)
  selected_agent: backend-payment-agent
  model_candidates: [...]
  selected_model: claude-sonnet
  skill_selection: [...]
  quality_policy: [...]
```

`codeg route explain ISSUE-101` 输出以上内容。

---

# 7. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-010 | 权限兼容是硬条件 |
| SPEC-002 | 生成 DAG 节点 |
| SPEC-011 | 从 Registry 解析 Profile |
| SPEC-113 | L3 消费 Eval |
| SPEC-111 | 模型选择（配合 Router） |

---

# 8. 验收标准

- [ ] Issue 可自动推导 capabilities。
- [ ] Agent Resolver 可基于能力排序并输出解释。
- [ ] Team Builder 能对 L3 任务生成多角色 Team。
- [ ] 权限不兼容的 Profile 不会被选中。
- [ ] `route explain` 可解释 agent/model/skill/quality 选择。

---

# 9. 边界与不做

- 不做执行（SPEC-101）。
- 不做 DAG 内核（SPEC-002）。
- 不做模型路由（SPEC-111）。
