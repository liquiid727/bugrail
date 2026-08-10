# SPEC-001 — Artifact System

> 状态：Baseline  
> 对应上位文档：`01-SpecOS-PRD.md` §4（D1）；`02-SpecOS-Module-Decomposition.md` K1/K2；交付 M0  
> 依赖：无（本 SPEC 是整个 SpecOS 的第一个数据契约）  
> 实现语言：不限定（Rust / TypeScript 均可），本 SPEC 只定义契约

---

# 1. 目的与范围

## 1.1 目的

定义 SpecOS 所有一等工程 Artifact 的：

- 数据结构（schema）
- ID 规则
- 状态机
- 关系（relations）
- 持久化接口（Store API）
- 文件存储布局

## 1.2 在链路中的位置

```text
本 SPEC 定义的数据契约
        │
  被下面这些模块消费（不反向依赖）
        │
  K3 状态机 ── K6 Quality Gate ── K7 Event ── K11 RunTrace
  K8 CLI ── P3 Context Resolver ── P1 Runtime ── P6/P7 验证审查
```

## 1.3 不在本 SPEC 范围

- 状态机引擎 / DAG（K3，见 SPEC-002）
- Quality Gate 判定逻辑（K6，见 SPEC-003）
- Context Pack 装配（P3，见 SPEC-103）
- 任何 AI / Agent 执行（P1，见 SPEC-101）
- 检索 / 索引（P2，见 SPEC-102）

---

# 2. Artifact 基类

## 2.1 通用字段

所有 Artifact 共有：

```yaml
id:            # 全局唯一，见 §3 ID 规则
project_id:    # 项目标识
type:          # 枚举，见 §4
title:         # 人类可读标题
status:        # 状态，按 type 不同，见 §5
version:       # 整数，从 1 递增
created_at:    # ISO 8601 UTC
updated_at:    # ISO 8601 UTC
created_by:    # agent_profile_id | human | system
relations:     # 关系列表，见 §6
metadata:      # 自由键值对，扩展用
```

## 2.2 必填 / 可选

| 字段 | 必填 | 说明 |
|---|---|---|
| id | ✓ | 创建时由 ID Generator 分配 |
| project_id | ✓ | 来自 project.yaml |
| type | ✓ | 创建时固定，不可变 |
| title | ✓ | |
| status | ✓ | 默认按 type 的初始状态 |
| version | ✓ | 默认 1 |
| created_at | ✓ | 系统写入 |
| updated_at | ✓ | 系统写入 |
| created_by | ✓ | |
| relations | ✗ | 可空数组 |
| metadata | ✗ | 可空对象 |

> **不可变字段**：`id`、`type`、`project_id`、`created_at`、`created_by`。  
> 其余字段可更新，更新时 `updated_at` 由系统刷新，`version` 在重大变更（如状态回退）时 +1。

---

# 3. ID 规则

## 3.1 格式

```text
<TYPE-PREFIX>-<序列号>
```

| type | 前缀 | 示例 |
|---|---|---|
| Spec | `SPEC` | SPEC-001 |
| Acceptance Criteria | `AC` | AC-001 |
| Issue | `ISSUE` | ISSUE-101 |
| Run | `RUN` | RUN-501 |
| Review | `REVIEW` | REVIEW-101 |
| Test | `TEST` | TEST-101 |

## 3.2 规则

- 序列号**全局递增**，不按 type 重置（沿用 01-SpecOS-PRD 中 SPEC-001 / ISSUE-101 / RUN-501 的风格，暗示同一全局序列）。
- 序列号由 `IdGenerator` 分配，存储层保证唯一。
- ID 一旦分配**永不复用**（即使 Artifact 被删除）。
- ID 是 Artifact 在文件系统中的目录名（见 §8）。

## 3.3 引用格式

其他 Artifact 或外部文档引用时，使用裸 ID 字符串：

```yaml
depends_on:
  - ISSUE-139
acceptance_criteria:
  - AC-001
```

---

# 4. type 枚举

首期实现以下 6 种：

```text
spec       规格
ac         验收标准（Acceptance Criteria）
issue      任务契约
run        一次执行
review     审查
test       测试执行
```

后续扩展：`idea`、`prd`、`qa`、`release`、`adr`。

---

# 5. 各 Artifact Schema 与状态机

## 5.1 Spec

### Schema

```yaml
type: spec
id: SPEC-001
title: 订单支持部分退款
status: draft

summary:             # 一段话
requirements:        # 需求列表（Markdown 数组）
  - 支持订单支付后的部分退款
  - 退款金额不能超过已支付金额

acceptance_criteria: # 引用 AC 的 ID
  - AC-001
  - AC-002

issues:              # 由 Spec 拆出的 Issue
  - ISSUE-101
  - ISSUE-102

version: 1
```

### 状态机

```text
draft ──approve──▶ approved ──archive──▶ archived
  │
  └──deprecate──▶ archived
```

- `draft`：正在编写，可修改。
- `approved`：已评审通过，Issue 只能从 approved 的 Spec 拆出。
- `archived`：不再使用。

### 约束

- 只有 `approved` 的 Spec 才能产生 Issue。
- `approved` 后修改 `requirements` 需 `version +1`。

---

## 5.2 Acceptance Criteria（AC）

### Schema

```yaml
type: ac
id: AC-001
title: 部分退款 API 返回 200
status: defined

spec_id: SPEC-001
description: |
  调用 POST /refund 且参数合法时，返回 200 并创建退款记录。

verification:        # 建议验证方式（自由文本）
  - integration-test
```

### 状态机

```text
defined ──implemented──▶ verified ──▶ closed
   │         （由 Issue/代码 实现）
   │
   └── rejected
```

- `defined`：已定义，待实现。
- `implemented`：有关联 Issue/Commit 实现（由 Acceptance Traceability 判定，首期可只做记录）。
- `verified`：有测试/Review 证明。
- `closed` / `rejected`：终态。

### 约束

- `spec_id` 必填。
- 一个 AC 必须能被追踪到至少一个 Issue（首期允许手动关联）。

---

## 5.3 Issue

### Schema

```yaml
type: issue
id: ISSUE-101
title: 后端部分退款接口
status: draft

spec_id: SPEC-001
goal:                 # 一句话目标
  实现部分退款的后端 API。

depends_on:           # 前置 Issue
  - ISSUE-099

scope:                # 涉及模块
  modules:
    - payment
    - order

acceptance_criteria:  # 引用 AC
  - AC-001
  - AC-002

agent_profile: backend-agent   # 手动/规则指定（M0）
verification:                  # 需要的验证
  required:
    - build
    - unit-test
    - review

risk:
  level: high

commands:             # 可覆盖项目默认命令
  test: pnpm test payment
```

### 状态机（核心）

```text
               ┌──────────────┐
               ▼              │ retry
draft ──▶ ready ──▶ running ──┘
          │         │    │
          │         │    ├──▶ failed ◀──┐
          │         │    │              │ retry
          │         │    ▼              │
          │         │  verifying ──▶ failed
          │         │    │
          │         │    ▼
          │         │  reviewing ──▶ running (request changes)
          │         │    │
          │         │    ▼
          │         │  completed
          │         ▼
          └────▶ blocked ◀───（依赖未满足）
```

| 事件 | 前状态 | 后状态 | 条件 |
|---|---|---|---|
| `approve` | draft | ready | Spec 已 approved |
| `start` | ready | running | 依赖全部 completed |
| `agent_failed` | running | failed | |
| `retry` | failed | running | |
| `verify_start` | running | verifying | Agent 自报完成 |
| `verify_fail` | verifying | failed | 验证未通过 |
| `review_request` | verifying | reviewing | 验证通过 |
| `request_changes` | reviewing | running | Review 打回 |
| `approve` | reviewing | completed | 全部 Gate 通过 |
| `block` / `unblock` | running | blocked | 依赖/资源 |
| `cancel` | running/ready/blocked | cancelled | 人工 |

> 关键原则：`Agent completed ≠ Issue completed`。Issue 进入 `completed` 只允许从 `reviewing` 且所有 Gate 通过时发生。

### 约束

- `spec_id` 必填。
- `depends_on` 引用必须存在。
- `acceptance_criteria` 引用的 AC 必须存在且属于同一 Spec。
- `verification.required` 至少一项。

---

## 5.4 Run

### Schema

```yaml
type: run
id: RUN-501
title: ISSUE-101 第一次执行
status: pending

issue_id: ISSUE-101
agent_profile: backend-agent
model: codex                # 实际使用
provider: openai

worktree_id: wt-issue-101
session_id: SES-1001        # ACP session 标识

started_at:
finished_at:
result:                     # 完成后填充
  summary:
  outcome: succeeded | failed
  changed_files:
    - src/payment/refund.ts
  commits:
    - abc1234
  error:                    # 失败时
```

### 状态机

```text
pending ──▶ running ──▶ verifying ──▶ completed
              │            │
              └──▶ failed ◀┘
              │
              └──▶ cancelled
```

| 事件 | 前状态 | 后状态 |
|---|---|---|
| `start` | pending | running |
| `agent_failed` | running | failed |
| `cancel` | pending/running | cancelled |
| `verify_start` | running | verifying |
| `verify_fail` | verifying | failed |
| `complete` | verifying | completed |

### 约束

- `issue_id` 必填。
- 一个 Issue 可以有多个 Run（retry 产生新 Run，`retry_count` 通过 Run 数量推导）。

---

## 5.5 Review

### Schema

```yaml
type: review
id: REVIEW-101
title: ISSUE-101 代码审查
status: pending

issue_id: ISSUE-101
run_id: RUN-501
reviewer_agent: review-agent
model: claude-sonnet

verdict: approve | request_changes   # 完成后
findings:                            # 数组
  - severity: high | medium | low
    file:
    line:
    message:
acceptance_check:                    # 每个 AC 的核对
  - ac_id: AC-001
    status: met | unmet | unverified
risk:
  level:
```

### 状态机

```text
pending ──▶ in_progress ──▶ approved
               │            └──▶ request_changes
               └──▶ failed（异常）
```

### 约束

- Review 与 Execution 必须使用独立 Session（强制）。
- `verdict` 为 `approve` 时，`findings` 应无 high severity 未处理项。

---

## 5.6 Test

### Schema

```yaml
type: test
id: TEST-101
title: ISSUE-101 单元测试执行
status: pending

issue_id: ISSUE-101
run_id: RUN-501
command: pnpm test payment

result:                     # 完成后
  passed: true | false
  summary:
  failed_cases:
    - test name
  coverage_of_ac:           # 覆盖到的 AC
    - AC-001
```

### 状态机

```text
pending ──▶ running ──▶ passed
               └──▶ failed
```

### 约束

- `command` 来自项目配置或 Issue override。
- 结果必须落盘，作为 Quality Gate 证据。

---

# 6. Relations（关系）

## 6.1 关系模型

每个关系一条记录：

```yaml
relation:
  from: ISSUE-101      # 源 Artifact
  to: AC-001           # 目标 Artifact
  type: satisfies      # 关系类型
  ref: SPEC-001        # 可选：关系描述/引用
  created_by: human
```

## 6.2 关系类型（首期）

| type | 方向 | 含义 |
|---|---|---|
| `belongs_to` | Spec → AC / Spec → Issue | 归属 |
| `depends_on` | Issue → Issue | 前置依赖 |
| `satisfies` | Issue → AC | 实现某个验收标准 |
| `produced_by` | Run → Issue | 执行产生 |
| `covered_by` | AC → Test | 被测试覆盖 |
| `reviewed_by` | Issue → Review | 被审查 |

## 6.3 存储位置

关系可以：

- 内嵌在各 Artifact 的 `relations` 字段（默认推荐，简单可查）；
- 或独立 `relations/` 目录（后续需要图查询时再引入）。

> 决策：**内嵌在 `relations` 字段**，`queryRelations(id)` 通过扫描各 Artifact 的 relations 汇总实现。引入 SQLite 索引后（M1）再独立存储。

---

# 7. Store API（K2）

## 7.1 接口定义

```ts
interface ArtifactStore {
  // 写
  save(artifact: Artifact): Promise<void>           // 新建或覆盖（同 version 覆盖）
  updateStatus(id: string, status: Status): Promise<Artifact>
  appendMetadata(id: string, kv: Record<string, unknown>): Promise<Artifact>

  // 读
  load(id: string): Promise<Artifact>               // 不存在抛 ArtifactNotFound
  loadByType(type: Type, id: string): Promise<Artifact>
  list(type?: Type, filter?: ListFilter): Promise<Artifact[]>
  queryRelations(id: string): Promise<Relation[]>

  // 版本
  bumpVersion(id: string): Promise<Artifact>

  // 生命周期
  remove(id: string): Promise<void>                 // 软删除（保留历史）
  exists(id: string): Promise<boolean>
}
```

## 7.2 语义

- `save` 幂等：同 id + 同 version 覆盖；新 version 追加。
- `list` 支持按 `status`、`spec_id`、`issue_id` 过滤。
- `remove` 是软删除：文件移到 `.specos/.trash/`，ID 不复用。
- 所有写操作成功后触发对应 Event（K7，见 SPEC-006）。

## 7.3 实现不可知

K2 只定义接口。默认实现是文件系统（§8），后续可加 SQLite 索引层（M1）与图/嵌入（M2），均不影响调用方。

---

# 8. 文件存储布局（默认实现）

## 8.1 目录结构

```text
.specos/
├── project.yaml
├── ids.json                     # ID 序列号持久化（或用文件系统推导）
├── specs/
│   └── SPEC-001/
│       ├── spec.md              # 人类阅读
│       └── spec.yaml            # 机器读取（含完整 relations/metadata）
├── acs/
│   └── AC-001/
│       ├── ac.md
│       └── ac.yaml
├── issues/
│   └── ISSUE-101/
│       ├── issue.md
│       └── issue.yaml
├── runs/
│   └── RUN-501/
│       ├── run.md
│       └── run.yaml
├── reviews/
│   └── REVIEW-101/
│       ├── review.md
│       └── review.yaml
├── tests/
│   └── TEST-101/
│       ├── test.md
│       └── test.yaml
└── .trash/                      # 软删除
```

## 8.2 双文件原则

- **`.yaml` 是源**：机器读写，schema 严格校验，包含全部字段（含 relations、metadata、version）。
- **`.md` 是渲染**：由 yaml 生成或人工维护，给人看。首期允许 md 滞后，以 yaml 为准。

## 8.3 文件命名

- 目录名 = Artifact ID（`SPEC-001`）。
- 文件名固定：`<type>.yaml` / `<type>.md`（如 `issue.yaml`、`issue.md`），不把 ID 放文件名，避免冗余。

## 8.4 原子性

- 写操作用「临时文件 + rename」保证原子性。
- 一次 save 写两个文件：先 yaml，后 md（md 失败不影响数据完整性）。

## 8.5 Git 追踪

- `.specos/` 应纳入 git（Artifact 是可追踪的历史）。
- `.specos/.trash/` 可纳入 git 或 gitignore，由项目决定。

---

# 9. 校验规则

Store 在 `save` 时执行以下校验，不通过抛 `ValidationError`：

```text
1. 必填字段齐全（§2.2）
2. type 是已知枚举
3. status 是该 type 状态集合的合法值
4. 引用完整性：
   - issue.spec_id 指向已存在 Spec
   - issue.acceptance_criteria 指向已存在 AC 且 spec_id 匹配
   - run.issue_id / review.issue_id / test.issue_id 指向已存在 Issue
5. ID 唯一（重复 id 且非覆盖 = 错误）
6. version 为正整数且 ≥ 当前版本（同 version 覆盖仅限 status/metadata 更新）
```

# 10. 验收标准

- [ ] 6 种 Artifact（spec/ac/issue/run/review/test）可创建、读取、更新、列出。
- [ ] ID 全局唯一且不复用。
- [ ] 每个 Artifact 有完整状态机，非法转换被拒绝。
- [ ] Issue 不能从非 approved 的 Spec 创建。
- [ ] Issue 的 `completed` 只能从 `reviewing` 且 Gate 全过后进入（Gate 判定在 SPEC-003，本 SPEC 只保证状态可达性）。
- [ ] `queryRelations(id)` 能返回内嵌 relations。
- [ ] 文件存储符合 §8 布局，yaml 为源，md 为渲染。
- [ ] 写操作原子（进程中断不产生半个文件）。
- [ ] 软删除后 ID 不复用，文件进 `.trash/`。
- [ ] 全部校验规则在 save 时生效。
- [ ] 纯内存实现可跑通全部单元测试（无文件系统依赖时）。

# 11. 边界与不做

- 不实现状态机引擎 / DAG（K3，SPEC-002）。
- 不实现 Quality Gate 判定（K6，SPEC-003）。
- 不实现 Context 装配（P3，SPEC-103）。
- 不实现 Agent 执行 / ACP（P1，SPEC-101，依赖 D1–D5 决策）。
- 不实现 idea/prd/qa/release/adr（后续扩展）。
- 不实现图数据库 / 检索索引（M1/M2）。
