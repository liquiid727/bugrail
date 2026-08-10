# SPEC-005 — Handoff 协议

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §8.3（D5）；`02` K5；`03` 里程碑 M0
> 依赖：SPEC-001（Artifact 模型）
> 实现语言：不限定；本 SPEC 只定义协议

---

# 1. 目的与范围

定义 Agent 之间**结构化交接**的数据契约，避免依赖长聊天上下文传递结果。

本 SPEC 只定义数据结构与生成/消费规则，不实现执行。

---

# 2. 数据结构（契约）

```yaml
handoff:
  issue_id: ISSUE-101
  run_id: RUN-501
  dag_node: TASK-001
  producer_agent: backend-payment-agent

  result:
    status: completed | failed | partial
    summary: 实现部分退款 API，含幂等键与金额校验。

  changed:
    - src/payment/refund.ts
    - src/payment/refund.test.ts

  decisions:
    - use refund_id for idempotency
    - reject refund if amount > paid

  risks:
    - legacy endpoint untouched
    - migration requires rollback

  verification:
    build: passed
    unit_test: passed

  artifacts:
    commit: abc1234
    test: TEST-101

  followups:               # 需要下游处理的事项
    - frontend 需消费新的 refund API
```

## 2.1 字段语义

| 字段 | 必填 | 说明 |
|---|---|---|
| issue_id | ✓ | 交接归属 |
| run_id | ✓ | 产生它的 Run |
| result.status | ✓ | completed/failed/partial |
| result.summary | ✓ | 一段话 |
| changed | ✓ | 改动文件（空则告警） |
| decisions | ✓ | 关键决策，供 Review/下游参考 |
| risks | ✓ | 未处理风险 |
| verification | ✗ | 已跑 Gate 结果 |
| artifacts | ✗ | commit / test 引用 |
| followups | ✗ | 下游待办 |

---

# 3. 生成与消费规则

## 3.1 生成

- 每个节点完成后由 Orchestrator 收集 Run 结果生成。
- 禁止从完整聊天转储生成；只收结构化数据。

## 3.2 消费方

```text
Review Provider（SPEC-107）  读 Handoff + Diff + Issue，而非整个聊天
Integration Agent            收集多个子 Handoff 合并
Context Compiler（SPEC-103） 将相关 Handoff 的 decisions/risks 注入下游 Context
```

## 3.3 校验

- `changed` 为空且 status=completed → 校验失败（Agent 自报完成但无改动）。
- `risks` 中存在高风险项时，下游必须显式处理或升级（配合 SPEC-105）。

---

# 4. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-001 | 引用 Issue/Run Artifact |
| SPEC-002 | 节点间交接依赖本协议 |
| SPEC-107 | Review 读取本协议 |
| SPEC-105 | 消费 risks 参与风险判定 |

---

# 5. 验收标准

- [ ] Handoff 数据结构完整，必填校验生效。
- [ ] status=completed 但 changed 为空 → 校验失败。
- [ ] 下游（Review/Integration）可直接基于 Handoff 工作，不依赖聊天上下文。

---

# 6. 边界与不做

- 不做执行（SPEC-101）。
- 不做 Review（SPEC-107）。
- 不做 DAG（SPEC-002）。
