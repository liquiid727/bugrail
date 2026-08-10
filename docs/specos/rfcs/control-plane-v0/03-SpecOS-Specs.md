# SpecOS SPEC 索引与交付顺序（已归档）

> **状态：Superseded。** 下列 `SPEC-001..113` 是早期横向模块拆分，不能
> 作为实现合同。正式 Specs 位于 `../../../../.features/`，按纵向能力闭环
> 拆为 `BUGRAIL-SPECOS-001..010`；完整映射与依赖顺序见
> `../../../../.features/roadmap.md`。

> 状态：Spec Index（基线）
> 本文件是 `docs/specs/` 下全部 SPEC 的索引、依赖关系与交付顺序。
> 模块边界见 `02-SpecOS-Module-Decomposition.md`；需求来源见 `01-SpecOS-PRD.md`；落地任务见 `04-SpecOS-Issues.md`。
> 原则：**契约一次定稿**。每个 SPEC 定义最终契约；Milestone 只排实现顺序。

---

# 1. SPEC 清单

## 1.1 内核 SPEC（K）

| SPEC | 模块 | 名称 | 里程碑 | 依赖 | 状态 |
|---|---|---|---|---|---|
| SPEC-001 | K1+K2 | Artifact System（模型 / ID / 状态机 / 关系 / Store API / 文件布局） | M0 | — | Baseline ✅ |
| SPEC-002 | K3 | Workflow / DAG Core | M0 | 001 | Draft |
| SPEC-003 | K6 | Quality Gate 模型 | M0 | 001 | Draft |
| SPEC-004 | K4 | Context Pack 协议 | M0 | 001 | Draft |
| SPEC-005 | K5 | Handoff 协议 | M0 | 001 | Draft |
| SPEC-006 | K7 | Event Bus | M0 | 001 | Draft |
| SPEC-007 | K11 | Run Trace 数据结构 | M0 | 001 | Draft |
| SPEC-008 | K8 | Control API / CLI | M0 | 001, 002, 003 | Draft |
| SPEC-009 | K9 | Config 系统 | M0 | 001 | Draft |
| SPEC-010 | K10 | Permission / Policy 模型 | M0 | 001, 009 | Draft |
| SPEC-011 | K12 | Plugin Registry | M0 | 001 | Draft |

## 1.2 插件 SPEC（P）

| SPEC | 模块 | 名称 | 里程碑 | 依赖 | 状态 |
|---|---|---|---|---|---|
| SPEC-101 | P1 | Agent Runtime Provider（ACP） | M0 | 010, 001 | Draft |
| SPEC-102 | P2 | Code Intelligence Provider | M1 | 011, 108 | Draft |
| SPEC-103 | P3 | Context Resolver | M0(M1/M2) | 004, 001 | Draft |
| SPEC-104 | P4 | Agent Resolver / Team Builder | M0(M1/M2) | 010, 011 | Draft |
| SPEC-105 | P5 | Risk Engine | M0(M1/M2) | 001, 003 | Draft |
| SPEC-106 | P6 | Quality Checker | M0 | 003 | Draft |
| SPEC-107 | P7 | Review Provider | M0(M1) | 005, 001 | Draft |
| SPEC-108 | P8 | Storage Engine | M0(M1/M2) | 001 | Draft |
| SPEC-109 | P9 | Memory Provider | M2 | 007, 011 | Draft |
| SPEC-110 | P10 | Skill Evolution | M2 | 007, 011, 113 | Draft |
| SPEC-111 | P11 | Model Router | M2 | 010, 113 | Draft |
| SPEC-112 | P12 | Architecture Intelligence | M2 | 102, 108 | Draft |
| SPEC-113 | P13 | Eval Aggregator | M1(M2) | 007, 011 | Draft |

> 里程碑括号内表示该插件的增强层次（如 P3 L1@M0 → L2@M1 → L3@M2），接口不变。

---

# 2. 依赖图（构建顺序）

```text
SPEC-001 Artifact
  │
  ├─▶ SPEC-002 Workflow/DAG ──▶ SPEC-008 Control API/CLI
  ├─▶ SPEC-003 Quality Gate ──▶ SPEC-106 Quality Checker
  ├─▶ SPEC-004 Context Pack ──▶ SPEC-103 Context Resolver
  ├─▶ SPEC-005 Handoff ──────▶ SPEC-107 Review Provider
  ├─▶ SPEC-006 Event Bus
  ├─▶ SPEC-007 Run Trace ────▶ SPEC-113 Eval ──▶ SPEC-110 Skill
  ├─▶ SPEC-009 Config ───────▶ SPEC-010 Permission
  │                              │
  └─▶ SPEC-011 Plugin Registry ─┴─▶ SPEC-101 Runtime
       │
       ├─▶ SPEC-102 Code Intelligence ──▶ SPEC-112 Architecture Intel
       ├─▶ SPEC-104 Agent Resolver/Team
       ├─▶ SPEC-105 Risk Engine
       ├─▶ SPEC-108 Storage Engine
       └─▶ SPEC-109 Memory ──▶ SPEC-111 Model Router
```

---

# 3. 交付顺序（Milestones）

## M0 — 最小垂直切片

目标：跑通 `Spec → Issue → Agent → Context → Worktree → Session → Review/Test → Done`。

SPEC 集合：

```text
内核：SPEC-001, 002, 003, 004, 005, 006, 007, 008, 009, 010, 011
插件：SPEC-101（Runtime/ACP）、SPEC-103（L1 确定性）、SPEC-104（L1 规则）、
      SPEC-105（L1 显式风险）、SPEC-106（Checker）、SPEC-107（L1 Review）、SPEC-108（L1 文件）
```

验收：

- [ ] 一条真实工程任务稳定跑通完整链路。
- [ ] 内核状态机全部可用纯内存测试覆盖。
- [ ] Context Pack 来源可追踪。
- [ ] Agent 完成 ≠ Issue 完成（Quality Gate 判定）。

## M1 — 智能理解与编排

目标：系统理解工程结构并安全并行。

```text
新增：SPEC-102（Code Intelligence）、SPEC-113（Eval L1）
增强：SPEC-103（L2）、SPEC-104（L2）、SPEC-105（L2）、SPEC-107（L2）、SPEC-108（L2 SQLite）
```

验收：

- [ ] 项目可完成首次索引并支持定义/引用/依赖/影响面查询。
- [ ] Context Pack 自动选相关代码并有 relevance reason 与 token budget。
- [ ] Team Builder 能对 L3 任务生成多角色 Team，Scheduler 支持并发。
- [ ] Run Timeline 可查看关键事件，Agent/Model 有基础 Eval 指标。

## M2 — 学习与受控自治

目标：系统能学习并受控地优化自身。

```text
新增：SPEC-109（Memory）、SPEC-110（Skill Evolution）、SPEC-111（Model Router）、SPEC-112（Architecture Intel）
增强：SPEC-103（L3）、SPEC-104（L3）、SPEC-105（L3）、SPEC-113（L2）、SPEC-108（L3 图/嵌入）
```

验收：

- [ ] Run 结束可自动产生 Memory Candidates，分类 / 来源 / 置信度齐全。
- [ ] 重复 Pattern 可形成 Skill Candidate，经验证提升为 Skill。
- [ ] Model Router 按策略选择模型且可解释，Provider 失败可 fallback。
- [ ] Architecture Intelligence 可识别基础 dependency drift。
- [ ] Autonomy Policy 限制自动操作，高风险 merge 强制 Human Gate。

---

# 4. 变更流程

- 修改某个 SPEC 契约 = 变更该 SPEC 的 `状态` 为 `Draft` 并新增版本号，同时更新依赖它的 SPEC 与 `04-SpecOS-Issues.md`。
- 新增能力域 = 先在 `01-SpecOS-PRD.md` 定义，再拆模块（02）与 SPEC（本文件），最后拆 Issue（04）。
- 禁止用「实现排期」理由改变已定稿契约；实现不足走插件增强，不走接口演进。
