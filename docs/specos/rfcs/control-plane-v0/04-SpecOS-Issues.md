# SpecOS Issue 积压（Backlog）

> 状态：Backlog（基线）
> 依据：`01-SpecOS-PRD.md`（需求）、`02-SpecOS-Module-Decomposition.md`（模块）、`03-SpecOS-Specs.md`（契约）
> 组织：按 Milestone（M0/M1/M2）与 Epic 分组；每个 Issue 标注关联 SPEC 与依赖。
> 说明：这是完整产品的一次性积压。实现时按依赖拓扑推进；任何 Issue 不改变已定稿契约。

---

# 1. M0 — 最小垂直切片

## EP0.1 内核基础设施（SPEC-001 / 108）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-001 | Artifact 基类模型与通用字段 | 001 | — | id/project_id/type/title/status/version/timestamps/created_by/relations/metadata |
| ISSUE-002 | 全局唯一 ID 生成器 | 001 | 001 | `<TYPE>-<序列>` 全局递增、不复用 |
| ISSUE-003 | 6 种 Artifact 的 schema 与状态机 | 001 | 001 | spec/ac/issue/run/review/test，非法转换被拒 |
| ISSUE-004 | Store API（K2）接口定义 | 001 | 001 | save/load/list/queryRelations/bumpVersion/remove |
| ISSUE-005 | 文件系统存储实现（.specos/） | 108 | 004 | yaml 为源、md 为渲染、原子写、软删除 |
| ISSUE-006 | 校验规则与引用完整性 | 001 | 003 | save 时校验必填/状态/引用存在 |

## EP0.2 Workflow / DAG（SPEC-002）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-010 | DAG 模型与依赖判定 | 002 | 001 | 节点字段、依赖、拓扑判定 |
| ISSUE-011 | 节点状态机（pending→…→completed） | 002 | 010 | 状态集合完整、只由事件触发 |
| ISSUE-012 | 事件触发状态转换 | 002 | 011 | 无 AI/存储依赖，纯内存可测 |

## EP0.3 Quality Gate（SPEC-003）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-020 | Gate 类型枚举与结果记录 | 003 | 001 | Implementation/Build/Lint/TypeCheck/Unit/Integration/Review/Security/QA/Human |
| ISSUE-021 | Done 判定逻辑 | 003 | 020 | Agent completed ≠ Issue completed |

## EP0.4 Context Pack（SPEC-004）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-030 | Context Pack schema | 004 | 001 | goal/scope/ac/related/symbols/files/rules/skills/memory/constraints |
| ISSUE-031 | 来源追踪（source/type/relevance/reason） | 004 | 030 | 每个 item 可溯 |

## EP0.5 Handoff（SPEC-005）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-040 | Handoff 数据结构 | 005 | 001 | result/changed/decisions/risks/verification/artifacts |

## EP0.6 Event Bus（SPEC-006）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-050 | 事件类型清单与 payload schema | 006 | 001 | issue.ready/run.*/verification.*/review.* 等 |
| ISSUE-051 | 发布/订阅实现 | 006 | 050 | 写操作成功后触发对应事件 |

## EP0.7 Run Trace（SPEC-007）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-060 | Run schema | 007 | 001 | issue/agent/model/timestamps/changed_files/outcome |
| ISSUE-061 | 运行时间线记录 | 007 | 060 | created→…→completed 关键事件 |

## EP0.8 Control API / CLI（SPEC-008）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-070 | CLI 命令树骨架 | 008 | 001 | spec/issue/run/review/test 命令族 |
| ISSUE-071 | `issue run` 黄金路径入口 | 008 | 002/003/101/103 | 完整执行链可由此触发 |
| ISSUE-072 | 输出格式与参数校验 | 008 | 070 | 稳定输出、错误语义 |

## EP0.9 Config（SPEC-009）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-080 | 分层配置与 override 解析 | 009 | 001 | global→project→agent→issue→run |

## EP0.10 Permission / Policy（SPEC-010）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-090 | 权限声明 schema | 010 | 001 | filesystem/shell/git/network/secrets/database/deployment |
| ISSUE-091 | Policy 评估与越权拦截 | 010 | 090 | 越权操作被拒绝 |
| ISSUE-092 | Human Gate 定义与触发 | 010 | 091 | spec/architecture/merge/release 审批 |

## EP0.11 Plugin Registry（SPEC-011）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-100 | 插件注册/发现/命名规范 | 011 | 001 | 通过 Registry 拿插件，不直接 import |

## EP0.12 Runtime Provider（SPEC-101）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-110 | ACP 客户端与 Session 生命周期 | 101 | 090 | createSession/resumeSession |
| ISSUE-111 | 任务发送/状态/取消/输出 | 101 | 110 | sendTask/getStatus/cancel/getOutput |
| ISSUE-112 | Worktree 创建/删除/合并 | 101 | 110 | 本地 git，生命周期 allocated→merged |
| ISSUE-113 | ACP 方案定案（D1–D5 决策） | 101 | — | 见 02 §8.1 开放问题 |

## EP0.13 Context Resolver L1（SPEC-103）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-120 | 确定性装配 | 103 | 030 | Issue+Spec+Profile+显式文件+规则+Skill+Knowledge |

## EP0.14 Agent Resolver L1（SPEC-104）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-130 | Agent Profile Registry 与加载 | 104 | 090/100 | profile 绑定模型/Skill/Knowledge/权限 |
| ISSUE-131 | 规则/手动指定 Agent | 104 | 130 | issue.agent_profile 直接指定 |

## EP0.15 Risk L1（SPEC-105）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-140 | 显式风险字段与简单规则 | 105 | 020 | risk 字段 + 高风险模块/public API/migration 规则 |

## EP0.16 Quality Checker（SPEC-106）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-150 | 项目命令配置与执行 | 106 | 020 | build/lint/typecheck/test 本地 runner |

## EP0.17 Review L1（SPEC-107）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-160 | 独立 Review Session 与判定 | 107 | 040 | review 读 Handoff+Diff+Issue，独立 session |

## EP0.18 Storage L1（SPEC-108）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-170 | 文件存储（与 ISSUE-005 合并实现） | 108 | 001 | 见 EP0.1 ISSUE-005 |

## EP0.19 UI 面板

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-180 | Artifact Inspector | 008 | 070 | 聊天中 Artifact ID 可点击打开 |
| ISSUE-181 | Issue 面板 | 008 | 180 | 状态/Goal/Scope/AC/Agent/Worktree/Session/Gates |
| ISSUE-182 | Session / Worktree 跳转 | 008 | 110 | 从 UI 打开对应 Session 接管 |

---

# 2. M1 — 智能理解与编排

## EP1.1 Code Intelligence（SPEC-102）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-200 | 仓库扫描与语言检测 | 102 | 100 | filesystem/git 索引 |
| ISSUE-201 | AST / Symbol 索引 | 102 | 200 | symbol 模型与存储 |
| ISSUE-202 | Reference / Dependency 图 | 102 | 201 | references/callers/dependencies 查询 |
| ISSUE-203 | Change Impact API | 102 | 202 | 返回 callers/modules/tests/api/related artifacts |
| ISSUE-204 | Architecture Map v1 | 102 | 202 | 生成 modules/dependencies/entrypoints/public APIs |

## EP1.2 Context L2（SPEC-103）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-210 | Scope / Symbol / 依赖解析 | 103 | 202 | 自动选相关代码 |
| ISSUE-211 | 相关性排序与 token budget | 103 | 210 | relevance reason + 超限策略 |

## EP1.3 Capability / Agent L2（SPEC-104）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-220 | Capability Resolver | 104 | 131 | title/goal/scope/ac/impact → 能力清单 |
| ISSUE-221 | deterministic scoring | 104 | 220 | 按能力/领域/偏好排序 |
| ISSUE-222 | Team Builder | 104 | 221 | L3 任务生成多角色 Team |

## EP1.4 Risk L2（SPEC-105）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-230 | 项目高风险模块配置 | 105 | 140 | high_risk_modules |
| ISSUE-231 | Impact 驱动风险评分 | 105 | 203 | 结合 change impact |

## EP1.5 Review L2（SPEC-107）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-240 | 专业 Reviewer 按风险选择 | 107 | 231 | security/database/api/architecture/performance |

## EP1.6 Storage L2（SPEC-108）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-250 | SQLite 索引层 | 108 | 005 | symbol/relations/eval 聚合查询 |

## EP1.7 Eval L1（SPEC-113）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-260 | Run 指标采集 | 113 | 060 | token/cost/duration/test/review/retry |
| ISSUE-261 | 按 Agent/Model 聚合 | 113 | 260 | 只观测不做决策 |

## EP1.8 DAG / Scheduler

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-270 | Scheduler 与并发限制 | 002 | 011 | 依赖满足+资源可用才 ready |
| ISSUE-271 | Execution Pool / Worktree 分配 | 002 | 270 | 多 Worktree 并发、provider 限流 |
| ISSUE-272 | Integration Agent | 002 | 271 | 收集 Handoff/commits/diff，处理冲突 |

## EP1.9 Task Graph UI

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-280 | Task Graph 可视化 | 008 | 270 | 节点状态/Agent/Model/Worktree/质量 |
| ISSUE-281 | Run Timeline / Context Inspect | 008 | 061/211 | 可查看关键事件与上下文原因 |

---

# 3. M2 — 学习与受控自治

## EP2.1 Memory（SPEC-109）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-300 | Memory schema 与分类 | 109 | 060 | Fact/Rule/Decision/Pattern/Preference/FailureLesson/Candidate |
| ISSUE-301 | Experience Extractor | 109 | 300 | 提取→去重→冲突检测→置信度 |
| ISSUE-302 | Memory 注入策略 | 109 | 301 | 按任务/角色/模块/风险选择，不全量注入 |

## EP2.2 Pattern Detection

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-310 | Run 序列归一化 | 109 | 301 | 工具链标准化 |
| ISSUE-311 | 重复 Pattern 检测与证据聚合 | 109 | 310 | 置信度/成功率，禁止一次 Run→Skill |

## EP2.3 Skill Evolution（SPEC-110）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-320 | Skill Candidate 模型 | 110 | 311 | trigger/scope/steps/source_runs/validation_status |
| ISSUE-321 | 验证流程（Shadow Run / Replay） | 110 | 320 | 与基线比较 quality/test/review/cost |
| ISSUE-322 | 提升/降级/归档生命周期 | 110 | 321 | usage/success/failure 指标，自动降级条件 |
| ISSUE-323 | Skill 冲突解析 | 110 | 322 | 显式>项目>模块>Skill>全局，不静默覆盖 |

## EP2.4 Model Router（SPEC-111）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-330 | Model Policy 与 Router 输入 | 111 | 261 | task/domain/risk/context/cost/eval/provider health |
| ISSUE-331 | Provider Fallback | 111 | 330 | timeout/rate limit/model unavailable/quality retry |
| ISSUE-332 | 路由可解释（route explain） | 111 | 331 | agent/model/skill/quality 选择原因 |

## EP2.5 Architecture Intelligence（SPEC-112）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-340 | 架构图维护（modules/boundaries/layers） | 112 | 204 | 数据所有权/风险区域 |
| ISSUE-341 | drift / 依赖违规检测 | 112 | 340 | 循环依赖/层违例/新耦合 |

## EP2.6 Auto Planning

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-350 | Auto Issue Decomposition | 104 | 222/204 | Spec→Issue DAG 建议，人工/Policy 审批 |
| ISSUE-351 | Auto Team Builder v2 | 104 | 261 | 结合 Agent/Model Eval 与优化目标 |

## EP2.7 Governance / Autonomy

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-360 | Autonomy 模式 | 010 | 092 | Manual/Assisted/Balanced/Autonomous/Strict |
| ISSUE-361 | 自动操作受控（merge/push/deploy/secrets） | 010 | 360 | 高风险强制 Human Gate |

## EP2.8 Eval L2（SPEC-113）

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-370 | Failure Taxonomy 统一分类 | 113 | 261 | context_missing/model_failure/merge_conflict/... |
| ISSUE-371 | Eval 作为 Router/Resolver 输入 | 113 | 370 | 数据反哺路由 |

## EP2.9 Context / Agent / Risk L3

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-380 | Context Optimizer（学习有用/无用上下文） | 103 | 302/311 | 减少无效 token，降低 context_missing |
| ISSUE-381 | Agent Resolver L3（Eval 感知） | 104 | 261 | 结合历史成功率/成本/延迟 |
| ISSUE-382 | Risk L3（历史数据参与评分） | 105 | 261 | 失败率/返工率 |

## EP2.10 知识健康与清理

| Issue | 标题 | SPEC | 依赖 | 验收要点 |
|---|---|---|---|---|
| ISSUE-390 | 健康度标记与衰减策略 | 109/110 | 322 | stale/degraded/conflicted，180 天规则 |
| ISSUE-391 | 清理与人工复核 UI | 008 | 390 | approve/merge duplicates/deprecate/archive |

---

# 4. 依赖拓扑（关键路径）

```text
ISSUE-001 → 003 → 004 → 005（存储基础）
ISSUE-001 → 010 → 011 → 012（DAG 内核）
ISSUE-001 → 020 → 021（Quality Gate）
ISSUE-090 → 110 → 111 → 112（Runtime + Worktree）
ISSUE-030 → 120（Context L1）
ISSUE-001 → 070 → 071 → 072（CLI 与黄金路径）
ISSUE-200 → 201 → 202 → 203 → 204（Code Intelligence）
ISSUE-060 → 260 → 261 → 330 → 331（Eval → Router）
ISSUE-311 → 320 → 321 → 322（Skill Evolution 主链）
```

建议从 `ISSUE-001` 开始，沿「存储基础 → DAG → Quality Gate → Runtime → CLI 黄金路径」推进 M0 垂直切片。
