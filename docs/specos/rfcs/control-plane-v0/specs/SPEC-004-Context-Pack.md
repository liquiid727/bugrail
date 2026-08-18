# SPEC-004 — Context Pack 协议

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §6（D3）；`02` K4；`03` 里程碑 M0
> 依赖：SPEC-001（Artifact 模型）
> 实现语言：不限定；本 SPEC 只定义协议

---

# 1. 目的与范围

定义 SpecOS 的**上下文协议**：

- Context Pack 数据结构
- 每个 item 的来源追踪与相关性元数据
- token budget 语义

本 SPEC 不实现上下文装配逻辑（SPEC-103），只定义协议与 schema。

---

# 2. 数据结构（契约）

## 2.1 Context Pack

```yaml
context_pack:
  id: CP-501
  run_id: RUN-501
  issue_id: ISSUE-101
  agent_profile: backend-payment-agent

  task:                    # 任务契约（来自 Issue）
    goal: 实现部分退款 API
    acceptance_criteria:
      - AC-001
      - AC-002

  spec_summary:            # 相关 Spec 摘要
    spec_id: SPEC-021
    requirements:
      - 支持订单支付后的部分退款

  project_rules:
    - .specos/rules/backend.md
    - .specos/rules/transaction.md

  skills:
    - ddd-backend
    - api-design

  knowledge:
    - docs/domain/payment
    - ADR/payment

  files:                   # 显式指定或解析得到的文件
    - src/payment/refund.ts
    - src/payment/refund.test.ts

  constraints:             # 执行约束
    - 不改动 legacy refund path
    - 必须提供 rollback

  token_budget:
    limit: 60000
    estimated: 42000

  items:                   # 完整来源清单（见 §3）
    - ...
```

## 2.2 Context item

```yaml
item:
  source: SPEC-021          # 来源引用
  type: requirement | rule | skill | knowledge | file | symbol | test | memory | decision | constraint
  content_ref: docs/backend/transaction.md   # 内容位置（不内联大块内容）
  relevance: 0.9           # 0..1
  reason: 直接引用 AC-001 的实现要求
  token_cost: 1200
  required: true           # required 项不可被 budget 裁剪
```

---

# 3. 来源追踪

## 3.1 原则

- 每个 item 必须可溯源到「为什么在这里」。
- `source` 指向 Artifact（SPEC/AC/ISSUE）、文件、规则、Skill、Memory、Decision。
- `reason` 记录装配时的判定理由，支持 Context Inspect 调试。

## 3.2 装配管线（由 SPEC-103 实现，本协议只定义输出）

```text
Issue → Scope Resolver → Artifact Resolver → Symbol Resolver →
Dependency Expansion → Rule/Skill Resolver → Rank → Token Budget → Context Pack
```

## 3.3 优先级（默认排序）

```text
Acceptance Criteria
> directly impacted symbols
> related spec / ADR
> direct dependencies
> related tests
> project rules
> relevant history
> semantic neighbors
```

---

# 4. Token Budget 语义

- `required: true` 的 item 不可裁剪。
- 超限时按 `relevance` 降序裁剪非 required 项。
- 裁剪必须记录在 `items` 的 `excluded` 清单（Context Inspect 展示）。

```yaml
excluded:
  - item: docs/payment/legacy.md
    reason: over_budget
    relevance: 0.3
```

---

# 5. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-001 | 读取 Issue/Spec/AC 字段 |
| SPEC-103 | 唯一装配者，产出本协议 |
| SPEC-102 | L2 阶段提供 symbol/dependency/impact 输入 |
| SPEC-109 | L3 阶段注入相关 Memory |
| SPEC-006 | 发布 context.compiled 事件 |

---

# 6. 验收标准

- [ ] Context Pack schema 完整，含 token_budget 与 items。
- [ ] 每个 item 有 source / type / relevance / reason。
- [ ] required 项不可被 budget 裁剪。
- [ ] 裁剪动作有 excluded 记录。
- [ ] 无任何 AI / 存储依赖，可纯内存构造与校验。

---

# 7. 边界与不做

- 不做装配逻辑（SPEC-103）。
- 不做检索（SPEC-102）。
- 不做 Memory 注入策略（SPEC-109）。
