# SPEC-111 — Model Router

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §12（D9）；`02` P11；`03` 里程碑 M2
> 依赖：SPEC-010（权限）、SPEC-113（Eval）
> 实现语言：不限定；接口契约为本 SPEC 主体

---

# 1. 目的与范围

定义模型选择与 Provider Fallback 插件。Profile 不再固定具体模型，由策略 + Eval 决定，且路由可解释。

---

# 2. 接口（契约）

```ts
interface ModelRouter {
  route(task: RouteInput): Promise<RouteDecision>
  explain(issueId): Promise<RouteExplanation>
}
```

```ts
interface RouteInput {
  taskType: string
  domain: string
  risk: RiskLevel
  contextSize: number
  costBudget?: number
  modelAvailability: ModelAvailability[]
  historicalEval?: EvalMetrics      // SPEC-113
  providerHealth?: ProviderHealth[]
}
```

```ts
interface RouteDecision {
  primary: ModelRef
  fallbackChain: ModelRef[]
  reviewModel?: ModelRef
}
```

---

# 3. Model Policy

```yaml
model_policy:
  quality: high
  max_cost: 1.0
  latency: normal
  fallback: true
  diversity_review: true
```

取代 Profile 里的固定模型。

---

# 4. 路由策略

```text
low-risk simple task        → cheap fast model
complex implementation      → coding-specialized model
high-risk architecture      → high-quality model
review                      → model from different family
```

## Fallback 处理

```text
provider timeout
rate limit
model unavailable
context overflow
quality retry
budget exceeded
```

---

# 5. 可解释路由

`codeg route explain ISSUE-101` 回答：

```text
为什么选这个 Agent
为什么选这个 Model
为什么装载这些 Skills
为什么需要 Security Review
为什么需要 Human Gate
```

输出结构化 `route_explanation`（见 SPEC-104 §6）。

---

# 6. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-113 | 输入 historicalEval |
| SPEC-104 | 与 Agent Resolver 协同 |
| SPEC-105 | 输入 risk |
| SPEC-010 | 约束模型可用性 |
| SPEC-007 | 记录实际使用模型 |

---

# 7. 验收标准

- [ ] 模型按策略选择而非固定。
- [ ] 支持 fallback 链（timeout/rate limit/unavailable）。
- [ ] 路由可解释。
- [ ] 高风险任务不因历史成功而绕过强制 Gate。
- [ ] review 可用不同家族模型（diversity_review）。

---

# 8. 边界与不做

- 不做 Agent 选择（SPEC-104）。
- 不做风险评分（SPEC-105）。
- 不做指标聚合（SPEC-113）。
