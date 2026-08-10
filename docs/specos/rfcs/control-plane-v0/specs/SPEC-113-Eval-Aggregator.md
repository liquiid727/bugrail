# SPEC-113 — Eval Aggregator

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §10（D7）；`02` P13；`03` 里程碑 M1（L1）/ M2（L2）
> 依赖：SPEC-007（Run Trace）、SPEC-011（Registry）
> 实现语言：不限定；接口契约为本 SPEC 主体

---

# 1. 目的与范围

定义 Run 指标聚合插件：按 Agent / Model / Provider / Skill / Context / Workflow / Task / Domain / Risk 维度统计，为 Router（SPEC-111）、Resolver（SPEC-104）、Skill（SPEC-110）提供数据。

- **L1**：只观测，不做决策。
- **L2**：作为 Router / Resolver 的输入。

---

# 2. 接口（契约）

```ts
interface EvalAggregator {
  aggregate(dimensions: EvalDimensions, filter?: EvalFilter): Promise<EvalResult>
  failureTaxonomy(): Promise<FailureTaxonomy>
}
```

```ts
interface EvalDimensions {
  by: Agent | Model | Provider | Skill | ContextPolicy | Workflow | TaskType | Domain | RiskLevel
  metrics: [success_rate, first_pass, review_score, test_pass, rework, latency, cost, token, failure_category, human_intervention]
}
```

---

# 3. 指标与维度

## 指标

```text
success rate
first pass success
review score
test pass
rework
latency
cost
token usage
failure category
human intervention
```

## 输出示例（Agent Eval）

```text
backend-agent / payment
Claude Sonnet    success 94%   first-pass 89%   avg cost $0.42   avg 4m20s
Codex            success 91%   first-pass 86%   avg cost $0.19   avg 3m05s
```

---

# 4. Failure Taxonomy

统一「失败」分类，否则 Eval 无法反哺。

```text
context_missing
context_noise
wrong_agent
model_failure
tool_failure
build_failure
test_failure
review_failure
requirement_misunderstanding
architecture_mismatch
merge_conflict
timeout
provider_failure
```

---

# 5. 数据来源

- Run Trace（SPEC-007）的 metrics / failure / verification。
- Event Bus（SPEC-006）的 run.* 事件。
- 无法统一拿到的字段（token/cost）允许为空。

---

# 6. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-007 | 主数据源 |
| SPEC-111 | L2 作为 Router 输入 |
| SPEC-104 | L2 作为 Resolver 输入 |
| SPEC-110 | Skill 成功率 |
| SPEC-108 | 聚合查询存储 |

---

# 7. 验收标准

- [ ] 可按 Agent/Model/Skill 聚合基础指标。
- [ ] 失败有统一分类。
- [ ] L1 只观测，L2 供路由决策。
- [ ] 指标缺失字段允许为空。

---

# 8. 边界与不做

- 不做路由（SPEC-111）。
- 不做执行（SPEC-101）。
- 不做 Trace 记录（SPEC-007）。
