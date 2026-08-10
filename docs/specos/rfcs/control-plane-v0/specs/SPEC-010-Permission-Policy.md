# SPEC-010 — Permission / Policy 模型

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §13.3 / §5.6（D2/D10）；`02` K10；`03` 里程碑 M0
> 依赖：SPEC-001, SPEC-009
> 实现语言：不限定；本 SPEC 只定义模型

---

# 1. 目的与范围

定义 SpecOS 的权限与治理模型：

- Agent 能力边界声明
- Policy 评估接口
- Human Gate 定义

Agent 必须声明能力边界；越权操作被拒绝；高风险动作触发 Human Gate。

---

# 2. 权限声明（契约）

```yaml
permissions:
  filesystem:
    read: true
    write: true

  shell:
    allowed: true

  git:
    commit: true
    push: false
    merge: false

  network:
    allowed: restricted

  secrets:
    allowed: false

  database:
    production: false

  deployment:
    production: approval-required
```

## 2.1 默认 Profile 建议

```yaml
execution-agent:   # write/commit 允许，push 拒绝
  filesystem.write: true
  git.push: false

review-agent:      # 只读
  filesystem.write: false
  git.commit: false
```

---

# 3. Policy 评估接口

```ts
interface PolicyEngine {
  check(actor: AgentProfile, action: Action, resource: Resource): Promise<Decision>
  // Decision: allow | deny | require_human_approval(reason)
  requireApproval(actor, action, resource, reason): Promise<ApprovalRequest>
}
```

```yaml
decision:
  verdict: allow | deny | require_human_approval
  reason: payment module, public API changed
  gate: human-approval       # 触发 SPEC-003 Gate
  approval_request_id:
```

---

# 4. Human Gate

## 4.1 定义

Human Gate 是一种 Workflow Node（SPEC-002 兼容）：

```text
waiting-for-approval → approved | rejected | changes-requested
```

## 4.2 适用场景

- Spec / Architecture 审批
- 高风险 merge
- 生产 migration
- Release
- Secrets / 生产数据访问

## 4.3 审批上下文（UI 展示）

```text
Why approval required
Risk
Affected modules
Diff
Tests
Review
Recommended action
```

---

# 5. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-001 | 权限声明可存于 Profile/Issue metadata |
| SPEC-003 | deny → 阻止执行；require_human_approval → 追加 human-approval Gate |
| SPEC-002 | Human Gate 作为节点状态参与 DAG |
| SPEC-105 | 风险等级决定是否 require_human_approval |
| SPEC-008 | 执行前权限检查 |

---

# 6. 验收标准

- [ ] 权限声明 schema 完整，默认 Profile 建议生效。
- [ ] Policy 评估返回 allow/deny/require_human_approval。
- [ ] 越权操作被拒绝。
- [ ] Human Gate 状态机完整，审批上下文可展示。

---

# 7. 边界与不做

- 不做风险评分（SPEC-105）。
- 不做审批 UI（SpecOS/Codeg UI）。
- 不做具体 Policy 规则集（项目配置，SPEC-009）。
