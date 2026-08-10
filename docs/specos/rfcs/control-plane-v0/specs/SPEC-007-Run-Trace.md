# SPEC-007 — Run Trace 数据结构

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §10（D7）；`02` K11；`03` 里程碑 M0
> 依赖：SPEC-001（Artifact 模型）
> 实现语言：不限定；本 SPEC 只定义数据结构

---

# 1. 目的与范围

定义每次 Agent Run 的**可回放记录** schema，为调试、成本核算、Eval（SPEC-113）与学习（SPEC-109/110）提供统一数据模型。

---

# 2. 数据结构（契约）

```yaml
run:
  id: RUN-501
  issue_id: ISSUE-101
  dag_node: TASK-001
  attempt: 2                # 第几次尝试（retry 递增）

  agent_profile: backend-payment-agent
  model: claude-sonnet
  provider: anthropic
  model_selected_by: rule | router      # M2 后由 SPEC-111 填充

  context_pack: CP-501
  context_sources:          # 来源统计（供 Context Optimizer）
    - source: SPEC-021
    - source: .specos/rules/backend.md
  skills:
    - ddd-backend
  knowledge:
    - docs/domain/payment
  tools:
    - filesystem
    - lsp

  session_id: SES-1001
  worktree_id: wt-issue-101

  status: pending | running | verifying | completed | failed | cancelled

  timestamps:
    created_at:
    started_at:
    context_compiled_at:
    session_started_at:
    agent_running_at:
    first_tool_call_at:
    commit_created_at:
    verification_started_at:
    review_started_at:
    finished_at:

  token_usage:              # 拿不到则留空
    input_tokens:
    output_tokens:
    cache_read_tokens:
    estimated_cost:

  changed_files:
    - src/payment/refund.ts
  commits:
    - abc1234

  verification:
    - gate: unit-test
      status: passed
      evidence: TEST-101
  review:
    review_id: REVIEW-101
    verdict: approve

  handoff: HANDOFF-501
  failure:                  # 失败时
    category: build_failure   # Failure Taxonomy（SPEC-113）
    error: ...
```

## 2.1 指标字段（Eval 输入）

```yaml
metrics:
  duration_s:
  tool_call_count:
  subagent_count:
  retry_count:
  test_pass_rate:
  first_pass: true | false
```

---

# 3. 时间线

```text
created → context_compiled → worktree_allocated → session_started →
agent_running → tool_call → subagent_started → commit_created →
verification_started → review_started → completed
```

关键时间点由 Event Bus（SPEC-006）事件回填，实现时间线还原。

---

# 4. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-001 | Run 是 Artifact 之一；引用 Issue/Review/Test |
| SPEC-006 | 由 run.* 事件回填字段 |
| SPEC-113 | 聚合 metrics 生成 Eval |
| SPEC-109 | 从 Run 提取 Memory 候选 |
| SPEC-110 | 从 Run 序列检测 Pattern |

---

# 5. 验收标准

- [ ] Run schema 完整，含 timestamps / token_usage / verification / failure。
- [ ] 字段允许缺失（不同 CLI 拿不到 token 时留空）。
- [ ] 时间线可由事件回放还原。
- [ ] failure.category 使用统一 Failure Taxonomy。

---

# 6. 边界与不做

- 不做聚合统计（SPEC-113）。
- 不做事件系统（SPEC-006）。
- 不做执行（SPEC-101）。
