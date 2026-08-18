# SPEC-107 — Review Provider

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §9（D6）；`02` P7；`03` 里程碑 M0（L1）/ M1（L2）
> 依赖：SPEC-005（Handoff）、SPEC-001
> 实现语言：不限定；接口契约为本 SPEC 主体

---

# 1. 目的与范围

定义独立于执行的代码审查插件。Review 与 Execution 必须使用独立 Session（强制）。

---

# 2. 接口（契约）

```ts
interface ReviewProvider {
  requestReview(issueId: string, opts: ReviewOpts): Promise<ReviewId>
  getReview(reviewId): Promise<Review>
}
```

```ts
interface ReviewOpts {
  runId: string
  diff?: DiffRef
  handoff?: HandoffRef          // SPEC-005
  focus?: ReviewType            // L2: security/database/api/architecture/performance/test
  model?: string                // 可用不同模型
}
```

---

# 3. Review 数据结构

```yaml
review:
  id: REVIEW-101
  issue_id: ISSUE-101
  run_id: RUN-501
  reviewer_agent: review-agent
  model: claude-sonnet
  status: pending | in_progress | approved | request_changes | failed

  verdict: approve | request_changes
  findings:
    - severity: high | medium | low
      file:
      line:
      message:
  acceptance_check:
    - ac_id: AC-001
      status: met | unmet | unverified
  risk:
    level:
```

---

# 4. 输入与判定

- 输入：Issue + Acceptance Criteria + Diff + Handoff（SPEC-005）+ 相关项目规则。
- 不读整个执行聊天（依赖 Handoff 与 Diff）。
- `verdict: approve` 时 findings 应无 high severity 未处理项。

## 实现层次

- **L1**：单一 review-agent，独立 Session，可配不同模型。
- **L2**：专业 Reviewer 按风险选择（配合 SPEC-105 Risk Engine），避免调用所有 Review Agent。

---

# 5. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-005 | 读取 Handoff |
| SPEC-001 | Review 是 Artifact |
| SPEC-003 | 产出 review GateResult |
| SPEC-105 | L2 按风险选择 Reviewer |
| SPEC-101 | 独立 Session 执行 |

---

# 6. 验收标准

- [ ] Review 与 Execution 使用独立 Session。
- [ ] Review 基于 Handoff+Diff+Issue，不依赖聊天上下文。
- [ ] 判定 approve/request_changes 带 findings 与 acceptance_check。
- [ ] L2 按风险选择专业 Reviewer。

---

# 7. 边界与不做

- 不做风险评分（SPEC-105）。
- 不做测试执行（SPEC-106）。
- 不做 Done 判定（SPEC-003）。
