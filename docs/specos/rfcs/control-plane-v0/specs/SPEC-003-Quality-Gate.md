# SPEC-003 — Quality Gate 模型

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §9（D6）；`02` K6；`03` 里程碑 M0
> 依赖：SPEC-001（Artifact 模型）
> 实现语言：不限定；本 SPEC 只定义契约

---

# 1. 目的与范围

定义 SpecOS 的**完成判定内核**：

- Gate 类型枚举
- Gate 结果记录
- Done 判定逻辑（Agent completed ≠ Issue completed）
- Gate 组合的动态策略接口（具体策略由 Risk Engine 提供，见 SPEC-105）

本 SPEC 不实现命令执行（SPEC-106）、Review（SPEC-107）、Test 运行，只定义模型。

---

# 2. 数据结构（契约）

## 2.1 Gate 类型枚举

```text
implementation      实现已完成（diff 非空）
build               构建通过
lint                Lint 通过
typecheck           类型检查通过
unit-test           单元测试通过
integration-test    集成测试通过
acceptance          验收标准验证通过
review              Review 通过
security-review     安全 Review 通过
qa                  QA 通过
human-approval      人工审批通过
```

## 2.2 Gate 结果

```yaml
gate_result:
  gate: unit-test
  issue_id: ISSUE-101
  run_id: RUN-501
  status: pending | passed | failed | skipped | blocked
  evidence:            # 证据
    test: TEST-101
    review: REVIEW-101
    artifact: RUN-501
  started_at:
  finished_at:
  detail:              # 失败原因等
```

## 2.3 Quality Gate 需求声明

```yaml
quality_gate:
  issue_id: ISSUE-101
  required:
    - build
    - unit-test
    - review
  dynamic: true        # 是否允许 Risk Engine 追加
  decisions:           # 记录每次判定
    - gate: unit-test
      status: passed
      evidence: TEST-101
```

---

# 3. Done 判定

## 3.1 核心规则

```text
Agent completed ≠ Issue completed

Issue completed ⇔
  所有 required gate 均 passed（或显式 skipped 且被批准）
  且无任何 required gate 处于 pending / failed / blocked
```

## 3.2 判定输入

```text
Issue.verification.required   （SPEC-001）
+ Risk Engine 动态追加的 gate （SPEC-105）
+ 每个 gate 的 GateResult
```

## 3.3 判定结果

```yaml
done_decision:
  issue_id: ISSUE-101
  verdict: done | not_done | blocked_on_gate
  unmet:
    - gate: security-review
      status: blocked
  required: [build, unit-test, security-review, review]
  decided_at:
```

## 3.4 特殊规则

- `human-approval` 只能由人类或 Policy 置为 passed，Agent 无权通过。
- 任何 gate `failed` 使 Issue 回到 `running` 或 `failed`（由 Orchestrator 决定 retry）。
- `skipped` 需记录原因与批准人，否则视为 `failed`。

---

# 4. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-001 | 消费 Issue 的 verification.required；写回 Issue 状态 |
| SPEC-002 | 节点 completed 依赖本模型的 Done 判定 |
| SPEC-105 | Risk Engine 提供动态 Gate 追加策略 |
| SPEC-106 | 执行 build/lint/typecheck/test 并产出 GateResult |
| SPEC-107 | Review 产出 review 类 GateResult |
| SPEC-006 | 发布 verification.* / quality.failed 事件 |

---

# 5. 实现方案

- 内核纯判定逻辑，无 AI / 存储依赖，可单测。
- 每个 gate 由对应插件产出结果；判定器只聚合。
- 判定器必须可解释：任何 issue 的 done_decision 可被 CLI/UI 查询。

---

# 6. 数据模型与存储

- GateResult 与 done_decision 作为 Artifact 附属记录，经 SPEC-001 Store 持久化。
- 关键判定写入 Issue 的 metadata，保证可追溯。

---

# 7. 验收标准

- [ ] Gate 类型枚举完整且可扩展。
- [ ] Done 判定严格满足「Agent completed ≠ Issue completed」。
- [ ] 任何 required gate 未过 → 不进入 done。
- [ ] human-approval 只能由人类/Policy 通过。
- [ ] skipped 必须有原因与批准人。
- [ ] done_decision 可解释，可在 CLI/UI 查询。

---

# 8. 边界与不做

- 不做命令执行（SPEC-106）。
- 不做 Review 执行（SPEC-107）。
- 不做风险驱动 Gate 追加的具体策略（SPEC-105）。
- 不做状态机（SPEC-002）。
