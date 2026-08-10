# SpecOS 产品需求文档（PRD）— 完整形态（已归档）

> **状态：Superseded / 仅保留概念来源。** 本文把 BugRail 当成绿地控制面，
> 不能授权实现。结合现有 CodeG WorkTask、ACP、Worktree、SQLite、事件、
> Provider、Skill 与 Tasks UI 后的正式产品需求见
> `../../../../.prd/prd-specos-delivery-control.md`，可实现 Feature Specs 见
> `../../../../.features/BUGRAIL-SPECOS-001-*` 至
> `../../../../.features/BUGRAIL-SPECOS-010-*`。

> 原状态：Product Baseline（现已由上述正式 PRD 取代）
> 上位文档：`00-SpecOS-Overall-Blueprint.md`
> 本文档取代原 `01/02/03` 三份阶段 PRD，合并为一份完整产品定义。
> 架构与模块边界见 `02-SpecOS-Module-Decomposition.md`；数据契约见 `03-SpecOS-Specs.md` 及各 SPEC；落地任务见 `04-SpecOS-Issues.md`。
> 关键原则：**契约一次定稿，实现按交付顺序展开，不搞 v1→v2→v3 的 API 演进**。交付顺序只是「先做什么」的实现排期，不是「先设计成什么样」。

---

# 1. 产品定位与目标

SpecOS 是一个基于 Codeg 的 **软件工程 Agent 操作系统（Software Engineering Agent OS）**。

它不是「更多 Agent」的聚合器，而是把用户需求转换成**可追踪的工程 Artifact 与 Issue DAG**，根据任务能力、风险、上下文动态选择 Agent / 模型 / Skill / Knowledge，在隔离的 Worktree / Session 中执行，并通过 Review / Test / QA / Acceptance Criteria 完成工程验证，最后把有效经验沉淀成项目 Memory、Rule、Pattern 或 Skill 的控制平面。

一句话：

> **从「让 Agent 帮忙改代码」升级为「让一个受控的工程团队帮你完成一次可验证的交付」。**

## 1.1 产品目标

1. **稳定**：一个结构化 Issue 能被指定 Agent 接收，在独立 Worktree / Session 中执行，有完整上下文、运行状态和质量验证，最终可追踪地完成。
2. **智能**：系统知道「该让谁做、哪些任务可并行、这次修改会影响哪里、怎样才算安全完成」。
3 **学习**：系统从历史运行持续学习，并受控地优化自身对 Agent / 模型 / Skill / Context / Workflow 的选择。

## 1.2 非目标

- 重做完整代码编辑器、Terminal、Git Client。
- 自研大模型 Runtime，或替代 Codex / Claude CLI。
- 无人工介入的全自动自治。
- 每次 Run 自动生成 Skill。
- 无限制跨项目共享 Memory。

---

# 2. 成功示例（端到端黄金路径）

> 这是贯穿整个产品的验收主线：完整形态必须能跑通它。

## 2.1 场景

用户：

```text
「给支付模块增加部分退款功能，并在订单详情显示退款记录。」
```

## 2.2 系统行为（完整链路）

```text
用户请求
  ↓
1. 分析 Complexity / Risk（L3，payment → high）
  ↓
2. 生成或关联 Spec（SPEC-021）
  ↓
3. 建立 Acceptance Criteria（AC-001..004）
  ↓
4. 拆分 Issue DAG（Backend / Frontend / Integration / Review / Test）
  ↓
5. Capability Resolver 推导所需能力（backend、payment、frontend、order、test）
  ↓
6. Agent Resolver 选择 Agent Profile（backend-payment-agent / frontend-order-agent）
  ↓
7. Model Router 选择模型与 Skill
  ↓
8. Context Compiler 基于 Code Intelligence 构建 Context Pack（作用域符号、依赖、相关测试、规则、ADR）
  ↓
9. Worktree Allocator 创建隔离 Worktree + Codeg Session
  ↓
10. Backend / Frontend 并行执行（各自独立 Session / Worktree）
  ↓
11. Agent 完成后生成 Handoff（changed / decisions / risks / verification）
  ↓
12. Integration Agent 合并，处理冲突，跑集成测试
  ↓
13. Quality Engine 按风险组合 Gate（unit + integration + security-review + human-merge）
  ↓
14. Acceptance Traceability：每个 AC 追踪到 Issue / Commit / Test / Review
  ↓
15. Human Gate 按风险触发（高风险 merge 需人工确认）
  ↓
16. Ship 后生成 Run Trace
  ↓
17. Experience Extraction 从 Run 提炼 Fact / Rule / Decision / Pattern
  ↓
18. 重复成功 Pattern 可成为 Skill Candidate → 验证 → 提升
  ↓
19. Eval 数据反哺 Agent / Model / Skill / Context 路由
```

用户在 Task Graph UI 中查看整个过程，可随时进入对应 Session 人工介入。

## 2.3 验收主线

- [ ] 上述 19 步在系统内全部有对应的一等对象与可操作入口。
- [ ] 任意一步可被人工接管、重试、取消、回放。
- [ ] 每个 AC 都能从「需求 → Issue → Commit → Test → Review」完整追踪。

---

# 3. 能力域总览

系统按能力域组织。每个域对应一组内核模块（K，见 02）与插件（P），并有对应 SPEC。

| # | 能力域 | 关键能力 | 内核模块 | 插件 | SPEC |
|---|---|---|---|---|---|
| D1 | 工程 Artifact 与存储 | Artifact 模型 / ID / 状态机 / 关系 / 持久化 | K1, K2 | P8 | 001, 108 |
| D2 | Agent Profile 与执行运行时 | Profile / Registry / ACP 执行 / Worktree / Session | K10 | P1 | 101, 010 |
| D3 | 上下文编译 | Context Pack / 来源追踪 / token budget | K4 | P3 | 004, 103 |
| D4 | 代码智能 | 索引 / 符号 / 引用 / 依赖 / 影响面 / 架构图 | — | P2 | 102 |
| D5 | 编排 | Capability Resolver / Team / DAG / Scheduler / 集成 | K3 | P4 | 002, 104 |
| D6 | 质量与风险 | Quality Gate / Risk / Review / Test / QA | K6 | P5, P6, P7 | 003, 105, 106, 107 |
| D7 | 运行追踪与 Eval | Run Trace / 指标 / Failure Taxonomy / Eval | K11 | P13 | 007, 113 |
| D8 | 记忆与演化 | Memory / Pattern / Skill / Architecture Intel | — | P9, P10, P12 | 109, 110, 112 |
| D9 | 模型路由 | 模型选择 / Provider Fallback / 可解释路由 | — | P11 | 111 |
| D10 | 控制面 | Control API / CLI / UI / 配置 / 权限 / 事件 | K7, K8, K9, K10, K12 | — | 006, 008, 009, 010, 011 |

> 插件名与编号对应关系以 `02-SpecOS-Module-Decomposition.md` §4 为准，本表只做导航。

---

# 4. D1 — 工程 Artifact 与存储

## 4.1 目标

PRD、Spec、Issue、Run、Review、Test、QA、ADR、Release 都是**一等对象**，而非散落的聊天文本。它们有稳定的 ID、状态机、版本、关系，可查询、可追踪、可回放。

## 4.2 用户故事

- 作为工程师，我能在聊天中点击 `SPEC-021` / `ISSUE-101` 直接打开对应对象。
- 作为工程师，我能从 Spec 一键拆出 Issue，且 Issue 只能从 approved 的 Spec 产生。
- 作为工程师，我能看到每个 Issue 依赖哪些前置 Issue、被哪些 AC 约束。
- 作为工程师，我能追溯「这个 AC 是被哪个 Commit / Test / Review 满足的」。

## 4.3 功能需求

1. **Artifact 基类**：id / project_id / type / title / status / version / created_at / updated_at / created_by / relations / metadata。不可变字段：id、type、project_id、created_at、created_by。
2. **ID 规则**：`<TYPE>-<序列号>`，序列号全局递增、不复用。
3. **类型**：spec / ac / issue / run / review / test 为一等类型；idea / prd / qa / release / adr 纳入同一模型。
4. **状态机**：每个类型有完整状态机，非法转换被拒绝（详见 SPEC-001 / 002）。
5. **关系**：belongs_to / depends_on / satisfies / produced_by / covered_by / reviewed_by 等，支持 `queryRelations(id)`。
6. **存储**：文件系统 `.specos/`（yaml 为源、md 为渲染）为默认实现，SQLite 索引为查询层，接口与实现解耦（K2）。
7. **验收追踪**：每个 AC 必须可追踪到 Issue / Commit / Test / Review；无实现或验证的 AC 阻止 Spec 进入完整 Done。

## 4.4 数据契约

- SPEC-001（Artifact 系统：模型 / ID / 状态机 / 关系 / Store API / 文件布局）
- SPEC-108（Storage Engine 插件：文件 → SQLite → 图）

## 4.5 验收标准

- [ ] 全部 Artifact 类型可创建 / 读取 / 更新 / 列出 / 软删除。
- [ ] ID 全局唯一且永不复用。
- [ ] 状态机覆盖全部类型，非法转换被拒绝。
- [ ] `queryRelations(id)` 可返回所有关联。
- [ ] Issue 不能从非 approved 的 Spec 创建。
- [ ] 每个 AC 可被追踪，缺失实现/验证时 Spec 不能完整 Done。

---

# 5. D2 — Agent Profile 与执行运行时

## 5.1 目标

Agent 是**可配置的 Profile**，不是固定实例。运行时根据任务动态实例化。执行通过统一的 Runtime Provider 控制（v1 为 ACP 对接 Codeg），上层不依赖具体 CLI。

## 5.2 用户故事

- 作为工程师，我能为项目配置 backend-agent / review-agent 等 Profile，绑定角色、模型策略、Skill、Knowledge、权限、质量要求。
- 作为工程师，我能指定某个 Issue 用哪个 Agent，也能让系统自动解析。
- 作为工程师，我能从 UI 打开执行 Session 手工接管。

## 5.3 功能需求

1. **Agent Profile**：role / capabilities / model_policy / skills / knowledge / tools / permissions / quality / cost-latency。通过 Registry 管理，不写死在 Orchestrator。
2. **Profile 解析顺序**：global → project → agent → issue → run（层层覆盖）。
3. **内置 Profile 集合**：idea / architecture / plan / spec / execution / frontend / backend / database / security / review / test / qa / integration / release，均来自 Registry 可配置。
4. **Runtime Provider**（P1，SPEC-101）：
   - createSession / resumeSession / sendTask / getStatus / cancel / getOutput / readArtifact。
   - 控制面走 ACP；数据面（worktree / diff）由 SpecOS 本地 git 处理。
5. **Worktree + Session**：默认 1 Issue = 1 Worktree + 1 执行 Session，允许附加 Review Session。Worktree 生命周期：allocated → active → ready-to-merge → merged → cleanup。
6. **权限模型**（SPEC-010）：Agent 必须声明能力边界（filesystem / shell / git / network / secrets / database / deployment），Human Gate 按风险触发。

## 5.4 数据契约

- SPEC-010（Permission / Policy）
- SPEC-101（Agent Runtime Provider / ACP）
- SPEC-001 §5.3 / §5.4（Issue / Run schema 中的 agent_profile、session、worktree 字段）

## 5.5 验收标准

- [ ] Profile 可配置并可被 Issue 引用或自动解析。
- [ ] `issue run` 自动创建 Worktree 与 Session，用户可进入 Session 接管。
- [ ] Agent 完成 ≠ Issue 完成：Done 由 Quality Gate 决定。
- [ ] 权限声明生效，越权操作被拒绝；高风险动作触发 Human Gate。

---

# 6. D3 — 上下文编译（Context Compiler）

## 6.1 目标

Context 按任务编译，不向 Agent 灌入全部知识。每次 Run 生成独立 Context Pack，来源可追踪，支持 token budget 与相关性评估。

## 6.2 用户故事

- 作为工程师，我能用 `codeg context inspect ISSUE-101` 查看这次任务到底给了 Agent 哪些上下文、为什么给。
- 作为工程师，我能排查「Agent 为什么没找到那个文件」——因为 Context Pack 缺了它。

## 6.3 功能需求

1. **Context Pack 协议**（SPEC-004）：Goal / Scope / AC / 相关 Spec / ADR / 符号 / 文件 / 调用图 / 相关测试 / 项目规则 / Skill / Memory / 先前决策 / 执行约束。
2. **来源追踪**：每个 item 记录 source / type / relevance / reason / token_cost / required。
3. **Compiler 管线**（P3，SPEC-103）：
   - v1 确定性装配：Issue + Spec + Profile + 显式文件 + 规则 + Skill + Knowledge。
   - 智能阶段：Scope Resolver → Artifact Resolver → Symbol Resolver → Dependency Expansion → Rule/Skill Resolver → Rank → Token Budget。
   - 学习阶段：基于 Run Trace / Memory 判断哪些上下文有用、哪些没有。
4. **Token budget**：支持上限与超限策略。
5. **Context Inspect**：CLI 与 UI 都能查看 required / selected / excluded / token budget / 来源原因。

## 6.4 数据契约

- SPEC-004（Context Pack 协议）
- SPEC-103（Context Resolver 插件）

## 6.5 验收标准

- [ ] 每次 Run 生成独立 Context Pack，来源可追踪。
- [ ] Context Pack 有 relevance 评分与 token budget。
- [ ] `context inspect` 可展示为什么选择 / 排除某项。
- [ ] 智能阶段能基于符号 / 依赖自动扩展作用域。

---

# 7. D4 — 代码智能（Code Intelligence）

## 7.1 目标

Codebase 能力不只是向量检索。目标是提供「定义 / 引用 / 调用者 / 被调用者 / 实现 / 测试 / 历史 / 影响面」的结构化查询，并产出 Architecture Map。

## 7.2 用户故事

- 作为工程师，我输入 `PaymentService.refund`，能拿到它的定义、引用、调用者、测试、历史、相关 Issue / Spec / ADR。
- 作为工程师，我提交一个改动前，能知道它会影响哪些模块和测试。

## 7.3 功能需求

1. **能力组成**：Filesystem Index + Git Index + AST/Symbol Index + LSP + Reference Graph + Dependency Graph + Semantic Retrieval + Change Impact。
2. **查询 API**（SPEC-102）：findSymbol / getDefinition / getReferences / getImplementations / getCallers / getCallees / getDependencies / getTests / getGitHistory / impact。
3. **Change Impact**：输入符号或文件，返回 Direct Callers / Dependent Modules / Tests / API Surface / Related Specs / Related Issues。Risk Engine 与 Context Compiler 共用。
4. **插件化**：Provider 接口定义后，实现可替换（builtin LSP/AST、外部引擎、企业索引、远程服务）。
5. **Architecture Map v1**：首次索引后生成 modules / dependencies / entrypoints / public APIs / datastores / external services / test layout，输出初版 Architecture Artifact。

## 7.4 数据契约

- SPEC-102（Code Intelligence Provider）

## 7.5 验收标准

- [ ] 项目可完成首次索引。
- [ ] 支持 Definition / References / Dependencies / Impact 查询。
- [ ] Impact 结果可用于 Risk Engine 与 Context Compiler。
- [ ] Provider 可替换，不影响内核。

---

# 8. D5 — 编排（Capability / Team / DAG / Scheduler / Integration）

## 8.1 目标

让系统知道「该让谁做、哪些可并行、依赖是什么」，并在隔离环境里安全并发执行与集成。

## 8.2 用户故事

- 作为工程师，我提出一个跨模块 Feature，系统自动拆出并行 Issue DAG，并分别分配 Agent。
- 作为工程师，我能在 Task Graph 看到每个节点状态、Agent、模型、Worktree、质量结果，并对任意节点介入。

## 8.3 功能需求

1. **Capability Resolver**（SPEC-104）：从 Issue 的 title / goal / scope / AC / impacted modules / risk 推导 required / optional capabilities 与 verification / review 需求。
2. **Agent Resolver**：按 capability match / domain match / model availability / permissions / project preference / cost 对 Profile 排序。基础版 deterministic scoring；进阶版结合历史 Eval。
3. **Team Builder**：任务动态生成 Team（简单任务可单 Agent）。可优化目标：Fast / Balanced / Quality / Budget。
4. **Task DAG**（SPEC-002）：节点含 task_id / issue_id / agent_profile / capabilities / depends_on / status / priority / risk / worktree / session / run。状态：pending / ready / running / blocked / reviewing / verifying / completed / failed / cancelled。
5. **Scheduler**：依赖满足 + 资源可用才进入 ready。负责并发限制、provider 并发、worktree 限制、priority、retry。
6. **Execution Pool**：Task Scheduler → Execution Pool → Worktree Allocator → Session Allocator → Agent Runtime。支持 `max_parallel_tasks` 与各 provider 限流。
7. **Integration Agent**：收集子 Handoff / commits / diffs，处理冲突，跑集成测试，输出 merged / conflicts / decisions / verification / unresolved。高风险 merge 默认需人工确认。
8. **Handoff Protocol**（SPEC-005）：Agent 间不依赖长聊天上下文传递结果，通过结构化 handoff（result / changed / decisions / risks / verification / artifacts）。

## 8.4 数据契约

- SPEC-002（Workflow / DAG Core）
- SPEC-005（Handoff）
- SPEC-104（Agent Resolver / Team Builder）

## 8.5 验收标准

- [ ] Issue 可自动推导 capabilities。
- [ ] Agent Resolver 可基于能力选择 Profile（含解释）。
- [ ] Team Builder 能对 L3 任务生成多角色 Team。
- [ ] DAG 支持依赖与拓扑判定，状态机完整。
- [ ] Scheduler 支持多 Worktree 并发与限流。
- [ ] Integration 可收集多 Handoff 并处理冲突。
- [ ] 每个 DAG 节点可跳转到对应 Session / Worktree。

---

# 9. D6 — 质量与风险（Quality / Risk / Review / Test / QA）

## 9.1 目标

Agent 不能自证完成。Done 必须由 Acceptance Criteria、Build、Test、Review 等 Quality Gates 决定，且 Gate 组合随风险动态调整。

## 9.2 用户故事

- 作为工程师，我能看到每个 Issue 的 Quality Gates 状态（✓ Build / ● Test / ○ Review）。
- 作为工程师，当改动涉及支付等高危模块时，系统自动加安全 Review 并要求人工 merge。

## 9.3 功能需求

1. **Quality Gate**（SPEC-003）：Implementation / Build / Lint / Type Check / Unit Test / Integration Test / Acceptance Verification / Review / Security Review / QA / Human Approval。按风险与任务动态组合。
2. **Done 判定**：`Agent completed ≠ Issue completed`。Issue completed = 所有 required gate 通过。
3. **Risk Engine**（P5，SPEC-105）：风险维度包括 domain risk / change size / dependency fanout / public API / database / auth / payment / security / infra / production config / test gap。结果影响 Gate 组合与 Human Gate。
4. **Quality Checker**（P6，SPEC-106）：执行 build / lint / typecheck / test 命令，超时 / 缓存 / 并行 / 远程执行。
5. **Review Provider**（P7，SPEC-107）：独立于执行的 Review Session；按风险选择专业 reviewer（code / architecture / security / database / api / performance / test）。
6. **Test 执行**：Test Artifact 记录命令、结果、失败用例、AC 覆盖。命令来自项目配置或 Issue override。
7. **Acceptance Traceability**：AC → Issue → Commit → Test → Review 完整可追踪，无实现/验证的 AC 阻止 Spec 完整 Done。

## 9.4 数据契约

- SPEC-003（Quality Gate）
- SPEC-105（Risk Engine）
- SPEC-106（Quality Checker）
- SPEC-107（Review Provider）

## 9.5 验收标准

- [ ] Gate 列表按风险动态生成。
- [ ] Agent 自报完成不直接导致 Issue Done。
- [ ] 高危模块自动触发 security-review 与 human-merge。
- [ ] Review 与 Execution 使用独立 Session。
- [ ] Test 结果落盘作为 Gate 证据。
- [ ] AC 可追踪到 Issue / Test / Review。

---

# 10. D7 — 运行追踪与 Eval

## 10.1 目标

所有重要 Agent Run 必须有 Trace，支持回放与评估，为模型路由和 Skill 演化提供真实数据。

## 10.2 用户故事

- 作为工程师，我提交一次 Run，能在 Run Inspector 看到 prompt、Context Pack、Skill、工具调用时间线、改动文件、成本、失败原因。
- 作为工程师，我按 Agent / Model / Skill 查看成功率、成本、延迟，判断该不该换。

## 10.3 功能需求

1. **Run Trace**（SPEC-007）：run 记录 issue / agent / model / provider / context_sources / skills / knowledge / tools / session / worktree / timestamps / token_usage / estimated_cost / changed_files / commits / tests / review_result / outcome。
2. **Run Timeline**：created → context_compiled → worktree_allocated → session_started → agent_running → tool_call → subagent_started → commit_created → verification_started → review_started → completed。
3. **Metrics**：input/output tokens、estimated cost、duration、tool calls、changed files、test result、review verdict、retry count。无法统一拿到的字段允许为空。
4. **Eval Aggregator**（P13，SPEC-113）：按 Agent / Model / Provider / Skill / Context Policy / Workflow / Task Type / Domain / Risk Level 聚合；指标含 success rate / first-pass / review score / test pass / rework / latency / cost / token / failure category / human intervention。
5. **Failure Taxonomy**：context_missing / context_noise / wrong_agent / model_failure / tool_failure / build_failure / test_failure / review_failure / requirement_misunderstanding / architecture_mismatch / merge_conflict / timeout / provider_failure。
6. **Replay / Simulation**：通过 Run Trace 做 Replay / Compare / Shadow Run（新 Skill 与基线比较）。

## 10.4 数据契约

- SPEC-007（Run Trace）
- SPEC-113（Eval Aggregator）

## 10.5 验收标准

- [ ] 每次 Run 生成完整 Trace，可查看时间线与失败原因。
- [ ] Agent / Model / Skill 有 Eval 数据。
- [ ] 失败有统一分类，可反哺路由与优化。
- [ ] 新 Skill 可通过 Replay / Shadow Run 与基线对比验证。

---

# 11. D8 — 记忆与演化（Memory / Pattern / Skill / Architecture Intelligence）

## 11.1 目标

经验先分类，再通过证据与验证决定是否提升为 Skill。自动记忆不等于自动生成 Skill。

## 11.2 用户故事

- 作为工程师，我能查看项目积累了哪些 Rule / Decision / Pattern，以及它们的置信度与来源。
- 作为工程师，我能审阅 Skill Candidate，决定是否提升为正式 Skill。

## 11.3 功能需求

1. **Memory 分类**：Fact / Rule / Decision / Pattern / Preference / Failure Lesson / Skill Candidate。项目级优先。
2. **Memory Provider**（P9，SPEC-109）：extract(run) / inject(task, profile) / health() / cleanup(policy)。提取后经 Deduplicate → Conflict Check → Confidence Update。
3. **Memory 注入策略**：不全部注入；Context Compiler 按 task scope / agent role / module / artifact / risk / historical relevance 选择。
4. **Pattern 检测**：重复、稳定、有证据、可泛化的行为才可能成为 Pattern；禁止一次 Run → Skill。
5. **Skill Evolution**（P10，SPEC-110）：候选 → 证据积累 → 离线验证 → Shadow Run → 审批 → Active → 持续 Eval → Validated / Deprecated。完整生命周期与版本、usage / success / failure 指标。
6. **Skill 冲突解析**：显式当前任务 > 项目规则 > 模块规则 > 项目 Skill > 全局 Skill；冲突不静默覆盖，可在 Inspector 查看。
7. **Architecture Intelligence**（P12，SPEC-112）：维护 modules / boundaries / dependencies / layers / public APIs / data ownership / risk zones，自动检测依赖违规 / 循环依赖 / 层违例 / drift。
8. **Auto Issue Decomposition**：根据 Spec + Architecture + Code Intelligence 自动提出 Issue DAG，经人工或 Policy 审批。
9. **知识健康与清理**：stale / degraded / conflicted / low-confidence 标记与衰减策略，避免无限膨胀。

## 11.4 数据契约

- SPEC-109（Memory Provider）
- SPEC-110（Skill Evolution）
- SPEC-112（Architecture Intelligence）

## 11.5 验收标准

- [ ] Run 结束可自动产生 Memory Candidates。
- [ ] Memory 有来源、置信度、冲突检测。
- [ ] 重复 Run 可形成 Pattern，Pattern 可形成 Skill Candidate。
- [ ] 一次 Run 不直接提升为正式 Skill。
- [ ] Skill 有生命周期、版本与指标。
- [ ] 新 Skill 通过 Replay / Shadow Run 验证。
- [ ] Architecture Intelligence 可识别基础 dependency drift。
- [ ] Memory / Skill 可被标记 stale / degraded 并清理。

---

# 12. D9 — 模型路由与 Provider Fallback

## 12.1 目标

Profile 不再固定具体模型。根据任务类型 / 风险 / 成本 / Eval 选择模型，支持 fallback，且路由决策可解释。

## 12.2 用户故事

- 作为工程师，我运行 `codeg route explain ISSUE-101`，能知道为什么选这个 Agent / 模型 / Skill / 需要哪些 Review。
- 作为工程师，Provider 失败或超时时，系统自动 fallback 到备选模型。

## 12.3 功能需求

1. **Model Policy**（SPEC-111）：quality / max_cost / latency / fallback / diversity_review，取代 Profile 里的固定模型。
2. **Router 输入**：task type / domain / risk / context size / cost budget / model availability / historical eval / provider health。
3. **Router 输出**：primary model / fallback chain / review model。
4. **Provider Fallback**：处理 timeout / rate limit / model unavailable / context overflow / quality retry / budget exceeded。策略示例：低风险简单任务 → 便宜快模型；复杂实现 → 编码专精模型；高危架构 → 高质量模型；review → 不同家族模型。
5. **可解释路由**：`route explain` 输出完整的 agent / model / skill / quality 选择原因。

## 12.4 数据契约

- SPEC-111（Model Router）

## 12.5 验收标准

- [ ] 模型按策略选择而非固定。
- [ ] Provider 失败可 fallback。
- [ ] 路由决策可解释（`route explain`）。
- [ ] 高风险任务不因历史成功而绕过强制 Gate。

---

# 13. D10 — 控制面（Control API / CLI / UI / 配置 / 权限 / 事件）

## 13.1 目标

为 Desktop / Web / Mobile / CI / MCP 暴露统一控制面；配置分层覆盖；权限受控；事件驱动所有状态变化。

## 13.2 功能需求

1. **Control API / CLI**（SPEC-008）：覆盖完整命令树：
   - `codeg idea new`、`codeg spec create|validate|show`、`codeg issue list|show|run|retry|cancel`、`codeg team plan|run|status`、`codeg run list|inspect`、`codeg review|test`、`codeg context inspect`、`codeg risk inspect`、`codeg skill list|evolve`、`codeg eval agent|model|skill`、`codeg route explain`、`codeg architecture inspect|drift`、`codeg replay`、`codeg ship`。
   - 关键命令 `codeg issue run ISSUE-101` 是完整黄金路径入口。
2. **事件总线**（SPEC-006）：所有状态变化通过事件驱动（issue.ready / run.created / run.started / run.completed / run.failed / verification.started / verification.failed / review.requested / review.completed / issue.completed / task.* / team.created / dag.updated / risk.updated / context.compiled / integration.* / quality.failed …）。
3. **配置系统**（SPEC-009）：Global → Project → Agent → Issue → Run 分层覆盖；`.specos/project.yaml` 定义 workflow / runtime / quality / agents / commands / orchestration / risk / code_intelligence / context / review / autonomy。
4. **权限与治理**（SPEC-010）：Agent 声明能力边界；Human Gate（spec / architecture / merge / release）按风险触发。
5. **UI**：
   - **Artifact Inspector**：聊天与 Agent 输出中的 Artifact ID 可点击，右侧统一面板展示 Idea / PRD / Plan / Spec / Issue / AC / Diff / Review / Test / QA / ADR / Release / Run / Context。
   - **Task Graph**：可视化 DAG，节点显示 status / agent / model / session / worktree / cost / duration / tests / review；支持 open session / pause / cancel / retry / reassign / change model / manual takeover / inspect context / open diff。
   - **Run Inspector**：prompt / context pack / skills / tool calls / timeline / subagents / changed files / tests / cost / final output / failure reason。
   - **Memory / Skill / Eval / Routing Explain / Architecture Health / Project Intelligence** 面板。
6. **Autonomy 模式**：Manual / Assisted / Balanced / Autonomous / Strict Enterprise。自动操作受 Risk / Permission / Human Gate 控制。

## 13.3 数据契约

- SPEC-006（Event Bus）、SPEC-008（Control API/CLI）、SPEC-009（Config）、SPEC-010（Permission）、SPEC-011（Plugin Registry）

## 13.4 验收标准

- [ ] 命令树完整，`issue run` 走通黄金路径。
- [ ] 状态变化全部通过事件发布，支持订阅。
- [ ] 配置分层覆盖生效，override 可解释。
- [ ] 权限越权被拒绝，Human Gate 按风险触发。
- [ ] Task Graph / Run Inspector / Artifact Inspector 可操作与跳转。

---

# 14. 完整产品验收标准

## 14.1 端到端黄金路径

- [ ] 用户提出需求后，系统完成 Complexity / Risk 分析并建议 Spec。
- [ ] Spec 可生成结构化 AC 与 Issue DAG。
- [ ] Capability → Agent → Model → Skill → Context → Worktree → Session 全链路自动解析。
- [ ] 并行任务在隔离环境执行，Handoff 结构化交接。
- [ ] Integration 合并并处理冲突。
- [ ] Quality Gates 按风险执行，Review / Test / QA 留痕。
- [ ] AC 全部可追踪；无验证的 AC 阻止完整 Done。
- [ ] Human Gate 按风险触发。
- [ ] Run Trace 生成，经验可提炼，Eval 数据反哺路由。

## 14.2 工程效率

- [ ] 从 Spec 到完成的 Cycle Time 可度量。
- [ ] 并行任务比例可度量。
- [ ] 人工介入次数可度量。
- [ ] Agent 探索代码所耗 Token / 时间可度量。

## 14.3 工程质量

- [ ] First-pass Success Rate、Review 一次通过率、Regression Rate、AC 覆盖率可度量。

## 14.4 Agent / 学习效果

- [ ] 每类 Agent 成功率 / 成本 / 耗时、Context 命中率、Model Fallback 率可度量。
- [ ] Memory 使用率、Skill Candidate → Validated 比例、Skill 使用后成功率提升、失效 Skill 淘汰率可度量。

---

# 15. 交付顺序（Milestones）

> 契约已经一次定稿。这里的里程碑只是**实现排期**：先打通最小闭环，再横向铺能力。任何里程碑都不改变契约，只增加插件实现的深度与覆盖面。

| 里程碑 | 目标 | 交付内容 |
|---|---|---|
| M0 | 最小垂直切片 | K1/K2/K3/K6/K11 内核、P1 ACP Runtime、P6 Checker、P7 Review v1、P8 文件存储、P3 确定性装配、P4 规则解析、P5 显式风险、SPEC-001..011 + 101/103/104/105/106/107/108 |
| M1 | 智能理解与编排 | P2 Code Intelligence、P3 智能装配、P4 能力评分、P5 影响面风险、P7 专业 Review、P13 Eval 观测、Task Graph UI |
| M2 | 学习与受控自治 | P9 Memory、P10 Skill Evolution、P11 Router、P12 Architecture Intel、P3 学习装配、P4 Eval 感知、P13 决策输入、Autonomy / Human Gate |
| M3 | 生态与治理 | 外部 Provider、企业索引、远程执行、深度可观测、安全 / 合规强化 |

交付顺序映射到 `04-SpecOS-Issues.md` 的 Issue 积压。

---

# 16. 明确不做（约束边界）

- 重做完整代码编辑器 / Terminal / Git Client。
- 自研大模型 Runtime，不替代 Codex / Claude CLI。
- 第一阶段就构建复杂知识图数据库 / 全自动自治 / 全自动部署。
- 每个任务强制多 Agent；每次 Run 自动生成 Skill。
- 无限制跨项目共享 Memory / Secrets。
- Agent 自动删除高价值 Rule / ADR；仅凭单次成功自动修改 Workflow。
- 使用不可解释的黑盒评分直接控制高风险操作。

原则：

> 自动化优先处理重复、低风险、可验证工作；高风险工程决策始终保留可解释和可干预路径。

---

# 17. 北极星指标

## 工程效率
- 从 Spec 到完成的 Cycle Time
- 并行任务比例
- 人工介入次数
- Agent 探索代码所占 Token / 时间

## 工程质量
- First-pass Success Rate
- Review 一次通过率
- Regression Rate
- Acceptance Criteria 覆盖率

## Agent 效率
- 每类 Agent 成功率
- 平均成本 / 平均耗时
- Context 命中率
- Model Fallback 率

## 学习效果
- Memory 使用率
- Skill Candidate → Validated 比例
- Skill 使用后的成功率提升
- 失效 Skill 淘汰率

---

# 18. 名词表

| 术语 | 含义 |
|---|---|
| Artifact | 一等工程对象（Spec / Issue / Run / Review / Test / …） |
| AC | Acceptance Criteria，验收标准 |
| Issue | 任务契约：goal / scope / AC / depends_on / agent / verification / risk |
| Context Pack | 一次 Run 的独立上下文包 |
| Handoff | Agent 间的结构化交接结果 |
| Quality Gate | 判定 Issue 是否可完成的一组验证 |
| Human Gate | 需要人工审批的节点 |
| Run Trace | 一次执行的可回放记录 |
| Skill Candidate | 待验证的可复用工作流候选 |
| Worktree | 隔离执行用的 git worktree |
| Codeg | 底层多智能体编码工作台（Runtime） |
