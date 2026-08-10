# SPEC-105 — Risk Engine

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §9（D6）；`02` P5；`03` 里程碑 M0（L1）/ M1（L2）/ M2（L3）
> 依赖：SPEC-001、SPEC-003
> 实现语言：不限定；接口契约为本 SPEC 主体

---

# 1. 目的与范围

定义风险评估插件：评估任务风险，影响 Quality Gate（SPEC-003）与 Human Gate（SPEC-010）。

---

# 2. 接口（契约）

```ts
interface RiskEngine {
  evaluate(issue: Issue, ctx: RiskContext): Promise<RiskAssessment>
}
```

```yaml
risk_assessment:
  issue_id: ISSUE-101
  level: low | medium | high | critical
  reasons:
    - payment module
    - public API changed
  dimensions:
    domain_risk: high
    change_size: medium
    dependency_fanout: low
    public_api_change: true
    database_change: false
    auth_permission: false
    security: medium
    test_gap: low
  required_gates:          # 追加到 SPEC-003 的 dynamic gate
    - security-review
    - integration-test
    - human-approval
```

---

# 3. 风险维度

```text
domain risk
change size
dependency fanout
public API change
database change
auth / permission
payment
security
infra
production config
test gap
```

## 实现层次

- **L1**：Issue 显式 `risk` 字段 + 简单规则（高风险模块 / public API / migration）。
- **L2**：项目配置高风险模块（SPEC-009 `high_risk_modules`）+ Code Intelligence impact（SPEC-102）。
- **L3**：历史数据（失败率 / 返工率，SPEC-113）参与评分。

---

# 4. 输出语义

- `level` 决定强制 Gate 与 Human Gate（SPEC-010 require_human_approval）。
- 高风险任务不因历史成功而绕过强制 Gate。

---

# 5. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-003 | 输出 required_gates 追加 Gate |
| SPEC-010 | high/critical 触发 human-approval |
| SPEC-102 | L2 消费 impact |
| SPEC-113 | L3 消费历史指标 |
| SPEC-009 | 读取 high_risk_modules |

---

# 6. 验收标准

- [ ] L1：识别显式风险字段与简单规则。
- [ ] L2：识别项目配置中的高风险模块 + change impact。
- [ ] L3：历史失败率参与评分。
- [ ] 风险影响 Gate 组合与 Human Gate。
- [ ] 输出可解释（reasons 明确）。

---

# 7. 边界与不做

- 不做 Gate 判定（SPEC-003）。
- 不做审批（SPEC-010）。
- 不做指标聚合（SPEC-113）。
