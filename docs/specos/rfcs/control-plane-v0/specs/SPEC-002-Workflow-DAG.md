# SPEC-002 — Workflow / DAG Core

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §8（D5）；`02` K3；`03` 里程碑 M0
> 依赖：SPEC-001（Artifact 模型与 Issue 状态机）
> 实现语言：不限定；本 SPEC 只定义契约

---

# 1. 目的与范围

定义 SpecOS 的**任务编排内核**：

- DAG 模型与节点结构
- 节点状态机（与 SPEC-001 的 Issue 状态机对齐但不相同）
- 依赖判定与拓扑
- 状态转换的事件触发规则

本 SPEC 不实现 Scheduler 并发调度 / Execution Pool（见 SPEC-104 与 ISSUE-270/271），只定义内核数据结构与状态机。

## 1.1 在链路中的位置

```text
Capability Resolver / Team Builder（SPEC-104）
        │  生成
        ▼
   Task DAG（本 SPEC）
        │  消费
        ├─▶ Agent Resolver（SPEC-104）
        ├─▶ Context Compiler（SPEC-103）
        ├─▶ Runtime Provider（SPEC-101）
        └─▶ Quality Gate（SPEC-003）
```

---

# 2. 数据结构（契约）

## 2.1 DAG

```yaml
dag:
  id: DAG-021            # 通常与 Spec 关联
  spec_id: SPEC-021
  status: planning | running | completed | failed | cancelled
  nodes:
    - TASK-001
    - TASK-002
  edges:                 # 显式边（冗余，便于图查询）
    - from: TASK-001
      to: TASK-002
      type: depends_on
  created_at:
  updated_at:
```

## 2.2 节点（Node）

```yaml
task:
  id: TASK-001
  issue_id: ISSUE-101        # 每个节点通常对应一个 Issue
  title: 后端部分退款 API

  capabilities:              # 由 Capability Resolver 填充
    required: [backend, payment]

  agent_profile: backend-payment-agent
  depends_on:
    - TASK-000

  status: pending
  priority: 50
  risk:
    level: high

  worktree_id: wt-issue-101
  session_id: SES-1001
  run_id: RUN-501

  quality_gate:              # 该节点需要的 Gate（SPEC-003）
    required: [build, unit-test, review]
```

## 2.3 状态集合

```text
pending
ready          # 依赖满足 + 资源可用（由 Scheduler 置位）
running
blocked        # 依赖未满足 / 资源不足
reviewing
verifying
completed
failed
cancelled
```

与 SPEC-001 Issue 状态机的对齐关系：

```text
Issue（SPEC-001）        Task 节点（本 SPEC）
draft        ──────────▶ 无（节点在 issue 进入 ready 后创建）
ready        ──────────▶ ready
running      ──────────▶ running
verifying    ──────────▶ verifying
reviewing    ──────────▶ reviewing
completed    ──────────▶ completed
failed       ──────────▶ failed
blocked      ──────────▶ blocked
cancelled    ──────────▶ cancelled
```

---

# 3. 状态机与转换

## 3.1 状态转换

```text
               ┌──────────────┐
               ▼              │ retry
pending ──▶ ready ──▶ running ─┘
   │         │         │
   │         │         ├──▶ blocked ◀──（依赖/资源）
   │         │         ▼
   │         │      verifying
   │         │         │
   │         │         ▼
   │         │      reviewing ──▶ running（request changes）
   │         │         │
   │         │         ▼
   │         │      completed
   │         ▼
   └────▶ cancelled
```

## 3.2 转换规则

| 事件 | 前状态 | 后状态 | 条件 |
|---|---|---|---|
| `task.ready` | pending | ready | 依赖全部 completed + 资源可用 |
| `task.started` | ready | running | Scheduler 分配 worktree/session |
| `task.blocked` | running | blocked | 依赖/资源不足 |
| `task.unblocked` | blocked | ready | 依赖/资源恢复 |
| `task.agent_failed` | running | failed | |
| `task.retry` | failed | running | |
| `task.verify_start` | running | verifying | Agent 自报完成 |
| `task.verify_fail` | verifying | failed | Gate 未过 |
| `task.review_request` | verifying | reviewing | 验证通过 |
| `task.request_changes` | reviewing | running | Review 打回 |
| `task.completed` | reviewing | completed | 全部 Gate 通过 |
| `task.cancelled` | pending/ready/running/blocked | cancelled | 人工/Policy |

> 原则：**状态转换只由事件触发，不写死**；节点 completed 只允许从 reviewing 且 Gate 全过时进入（与 SPEC-003 一致）。

## 3.3 依赖判定

- 节点可进入 `ready` 当且仅当所有 `depends_on` 节点状态为 `completed`。
- 环检测：DAG 构建时拒绝环（拓扑排序失败即错误）。
- `blocked` 由 Scheduler 或依赖事件置位，`unblocked` 由依赖 completed 事件触发。

---

# 4. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-001 | 节点引用 Issue；节点状态机对齐但独立 |
| SPEC-003 | 节点完成后是否可进入 completed 由 Quality Gate 判定 |
| SPEC-104 | Capability Resolver / Team Builder 生成 DAG；Scheduler 置位 ready |
| SPEC-005 | 节点完成后生成 Handoff 供下游消费 |
| SPEC-006 | 所有转换发布事件（task.* / dag.updated） |

---

# 5. 实现方案

- 内核纯数据结构 + 状态机，不依赖任何 AI / 存储。
- 建议独立 crate/package：`specos-core`（Rust）或 `@specos/core`（TS）。
- Scheduler（并发/资源）作为独立层，本 SPEC 只保证状态可达性。

---

# 6. 数据模型与存储

- 状态与 DAG 结构经 Artifact Store（SPEC-001）持久化。
- 事件由 Event Bus（SPEC-006）发布，不直接写库。

---

# 7. 验收标准

- [ ] DAG 支持依赖与拓扑判定，构建时拒绝环。
- [ ] 节点状态机覆盖 pending/ready/running/blocked/reviewing/verifying/completed/failed/cancelled。
- [ ] 状态转换只由事件触发，不写死。
- [ ] 节点 completed 只能从 reviewing 且 Gate 全过时进入。
- [ ] 无任何 AI / 存储依赖，可用纯内存测试覆盖全部转换。

---

# 8. 边界与不做

- 不做 Scheduler 并发调度 / 资源池（SPEC-104）。
- 不做 Team / Capability 生成（SPEC-104）。
- 不做 Gate 判定（SPEC-003）。
- 不做执行（SPEC-101）。
