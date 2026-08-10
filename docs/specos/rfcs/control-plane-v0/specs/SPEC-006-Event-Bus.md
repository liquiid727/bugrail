# SPEC-006 — Event Bus

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §13（D10）；`02` K7；`03` 里程碑 M0
> 依赖：SPEC-001（Artifact 模型）
> 实现语言：不限定；本 SPEC 只定义契约

---

# 1. 目的与范围

定义 SpecOS 的**事件驱动内核**：

- 事件类型清单
- payload 结构
- 发布 / 订阅语义

所有状态变化通过事件发布，上层（Scheduler / UI / Integration）通过订阅响应，不轮询。

---

# 2. 事件类型（契约）

## 2.1 内核事件

| 事件 | 说明 | 关键 payload |
|---|---|---|
| `artifact.created` | Artifact 创建 | id, type |
| `artifact.updated` | Artifact 更新 | id, type, status |
| `issue.ready` | Issue 可执行 | issue_id |
| `issue.completed` | Issue 完成 | issue_id, run_id |
| `issue.failed` | Issue 失败 | issue_id, reason |
| `run.created` | Run 创建 | run_id, issue_id |
| `run.started` | Run 开始 | run_id, session_id |
| `run.completed` | Run 完成 | run_id, outcome |
| `run.failed` | Run 失败 | run_id, error |
| `verification.started` | 验证开始 | run_id, gate |
| `verification.failed` | 验证失败 | run_id, gate, detail |
| `verification.passed` | 验证通过 | run_id, gate |
| `review.requested` | 请求 Review | run_id, review_id |
| `review.completed` | Review 完成 | review_id, verdict |
| `quality.failed` | Quality Gate 未过 | issue_id, gate |
| `worktree.allocated` | Worktree 分配 | worktree_id, issue_id |
| `worktree.merged` | Worktree 合并 | worktree_id |
| `context.compiled` | Context 编译完成 | run_id, context_pack_id |
| `handoff.generated` | Handoff 生成 | run_id, handoff |

## 2.2 编排事件

| 事件 | 说明 |
|---|---|
| `task.created` / `task.ready` / `task.started` / `task.blocked` / `task.completed` | DAG 节点状态 |
| `team.created` | Team 生成 |
| `dag.updated` | DAG 结构/状态更新 |
| `risk.updated` | 风险重估 |
| `integration.started` / `integration.completed` | 集成节点 |
| `memory.extracted` | Memory 候选提取 |
| `skill.candidate` / `skill.promoted` / `skill.deprecated` | Skill 演化 |

## 2.3 payload 通用字段

```yaml
event:
  id: EVT-0001
  type: run.started
  at: 2026-08-09T10:00:00Z
  actor: orchestrator | agent | human | system
  correlation_id: CORR-501     # 关联一次 issue run
  data: { ... }                # 上表的关键 payload
```

---

# 3. 发布 / 订阅语义

## 3.1 发布

- 写操作成功后由 Store / 状态机发布（见 SPEC-001 §7.2）。
- 事件不可变，发布后不得修改。

## 3.2 订阅

```ts
interface EventBus {
  publish(evt: Event): Promise<void>
  subscribe(type: EventType | '*', handler): Promise<SubscriptionId>
  unsubscribe(id): Promise<void>
  replay(types: EventType[], since): Promise<Event[]>   // 事件溯源/恢复
}
```

## 3.3 可靠性

- 至少一次投递（at-least-once）；消费者需幂等。
- 事件持久化到 Event Log（配合 SPEC-007 与 Storage），支持 replay 恢复状态。

---

# 4. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-001 | Store 写操作后发布 artifact.* 事件 |
| SPEC-002 | 状态转换发布 task.* / dag.updated |
| SPEC-003 | Gate 判定发布 verification.* / quality.failed |
| SPEC-007 | Run 事件进入 Trace 供 Eval 使用 |
| SPEC-101 | Session/任务事件映射到 run.* |

---

# 5. 验收标准

- [ ] 事件类型清单完整（内核 + 编排）。
- [ ] payload 有通用字段（id/type/at/actor/correlation_id/data）。
- [ ] 发布后不可变；写操作成功后自动发布。
- [ ] 支持订阅与 replay。
- [ ] 至少一次投递，消费者幂等。

---

# 6. 边界与不做

- 不做持久化实现细节（配合 SPEC-108）。
- 不做 Trace 聚合（SPEC-007 / SPEC-113）。
- 不做状态机（SPEC-002）。
