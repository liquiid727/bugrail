# SPEC-008 — Control API / CLI

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §13（D10）；`02` K8；`03` 里程碑 M0
> 依赖：SPEC-001, 002, 003
> 实现语言：不限定；本 SPEC 只定义命令树与契约

---

# 1. 目的与范围

定义 SpecOS 的统一控制面：CLI 命令树、参数校验、输出格式。Desktop / Web / Mobile / CI / MCP 复用同一 Control API。

关键命令 `codeg issue run ISSUE-101` 是完整黄金路径的入口。

---

# 2. 命令树（契约）

## 2.1 Artifact

```bash
codeg idea new [--goal "..."]

codeg spec create [--title ...] [--from PRD-...]
codeg spec show SPEC-001
codeg spec validate SPEC-001
codeg spec approve SPEC-001
codeg spec issues SPEC-001            # 从 Spec 拆 Issue

codeg issue list [--status ...]
codeg issue show ISSUE-101
codeg issue run ISSUE-101
codeg issue retry ISSUE-101
codeg issue cancel ISSUE-101

codeg run list
codeg run inspect RUN-501
```

## 2.2 编排

```bash
codeg team plan SPEC-021
codeg team run SPEC-021
codeg team status SPEC-021

codeg graph show SPEC-021
codeg context inspect ISSUE-101
codeg risk inspect ISSUE-101
codeg review ISSUE-101
codeg test ISSUE-101
```

## 2.3 智能 / 学习

```bash
codeg code symbol PaymentService
codeg code references PaymentService.refund
codeg code impact PaymentService.refund

codeg eval agent backend-agent
codeg eval model claude-sonnet
codeg eval skill prisma-migration

codeg route explain ISSUE-101
codeg architecture inspect
codeg architecture drift

codeg memory list
codeg memory inspect MEM-21
codeg memory promote MEM-21

codeg skill candidates
codeg skill inspect SKILL-C-12
codeg skill validate SKILL-C-12
codeg skill promote SKILL-C-12
codeg skill deprecate SKILL-12

codeg replay RUN-101
codeg ship SPEC-021
```

## 2.4 关键命令内部语义

`codeg issue run ISSUE-101`：

```text
resolve agent → resolve model → resolve skills → resolve knowledge
→ compile context → allocate worktree → create session → execute
→ verify → review → done → trace
```

---

# 3. 参数校验与输出格式

## 3.1 校验

- Artifact ID 必须符合 SPEC-001 ID 规则。
- 状态参数必须属于该类型状态集合。
- 不存在/非法引用返回结构化错误码（见 §4）。

## 3.2 输出

- 默认人类可读表格/Markdown。
- `--json` 输出稳定 schema（版本化）。
- 错误输出统一 `{ error: { code, message, detail } }`。

---

# 4. 错误码（首期）

```text
ARTIFACT_NOT_FOUND
ARTIFACT_INVALID_STATE
ISSUE_NOT_RUNNABLE      # 依赖未满足 / Spec 未 approve
GATE_BLOCKED
PERMISSION_DENIED
WORKTREE_CONFLICT
SESSION_UNAVAILABLE
RUN_FAILED
```

---

# 5. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-001 | 读取/写入 Artifact |
| SPEC-002 | 触发 DAG 节点 |
| SPEC-003 | 查询 Gate 状态 |
| SPEC-006 | 订阅事件输出实时状态 |
| SPEC-010 | 执行前权限检查 |

---

# 6. 验收标准

- [ ] 命令树覆盖 Artifact / 编排 / 智能 / 学习。
- [ ] `issue run` 走通完整黄金路径。
- [ ] 参数校验与错误码结构化。
- [ ] `--json` 输出 schema 稳定。

---

# 7. 边界与不做

- 不做 UI（在 SpecOS UI / Codeg UI 实现）。
- 不做执行细节（SPEC-101）。
- 不做权限策略（SPEC-010）。
