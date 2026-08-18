# SpecOS 模块拆分：内核 vs 插件接缝

> 状态：Architecture / Module Decomposition（基线）
> 目的：把蓝图（00）与统一 PRD（01）拆成可独立开发、可替换的模块边界，为逐模块落地 Spec（03）与 Issue（04）打基础。
> 与 00/01/03/04 的关系：00 是愿景，01 是完整产品 PRD，本文件是模块架构拆分，03 是 SPEC 索引，04 是 Issue 积压。
> 设计原则：**契约一次定稿**。本文件定义的每个模块契约即最终形态，交付顺序（Milestone）只决定「先实现哪个」，不改变契约本身，避免 v1→v2→v3 的 API 演进成本。

---

# 1. 拆分动机

不拆的风险：

```text
Code Intelligence、Context v2、Team、DAG、Risk、Eval 单独拿出来都是复杂系统。
如果整体做，任何一块卡住都会阻塞整条线。
```

拆的目的：

- **可插拔**：不同项目 / 团队需要不同实现（builtin vs 外部引擎、本地 vs 远程）。
- **可并行开发**：多个模块不互相阻塞，可以按优先级推进。
- **可独立测试**：内核不依赖具体实现即可测；插件可以单独验证。
- **可替换**：某个实现被更好的替代（如换 Code Intelligence 引擎）时不影响内核。

一句话：

> 内核负责「SpecOS 为什么存在」，插件负责「用哪种方式实现」。内核薄而稳定，插件厚而可换。

---

# 2. 拆分原则

## P1 依赖倒置

```text
内核  ──定义接口──▶  插件
内核  ──✗ 不 import──▶  任何具体插件实现

插件  ──依赖──▶  内核的接口与数据结构
```

内核只能通过 Plugin Registry（K12）拿到插件实例，不能直接 import 某个实现。

## P2 内核薄

内核只放：

- 数据结构与协议（Artifact、Context Pack、Handoff、Run Trace）
- 状态机与判定逻辑（Workflow/DAG、Quality Gate 的 Done 判定）
- 事件与命令定义（Event Bus、Control API）
- 配置与权限模型（分层覆盖、Policy 评估）

不放：

- 具体 AI 调用
- 具体检索算法
- 具体存储实现
- 具体 Agent 执行

## P3 接缝清晰

每个插件 seam 必须同时定义三样东西，缺一不可：

```text
Interface   —— 插件必须实现的接口（最终契约）
默认实现    —— 首个交付的默认实现
可选实现    —— 后续可替换成什么（不改变接口）
```

## P4 插件独立演进

一个插件可以停在默认实现，另一个推进到外部引擎。Milestone 只决定「哪些插件至少要成熟到什么程度」，不强制所有插件同步升级，也不改变任何接口契约。

## P5 垂直切片优先

先跑通一条完整垂直切片，再横向铺其他插件。

首个垂直切片：

```text
issue run
→ resolve agent (规则)
→ compile context (显式来源)
→ allocate worktree (本地 git)
→ create session (ACP)
→ execute
→ verify (quality gate)
→ review
→ done (run trace)
```

该切片对应 M0，是系统可用的最小闭环。

---

# 3. 内核模块（Kernel）

内核共 12 个模块。它们定义「SpecOS 的领域语言」，不依赖任何具体实现。

| 编号 | 模块 | 职责 | 稳定契约 | SPEC |
|---|---|---|---|---|
| K1 | Artifact Core | Artifact 通用模型、ID 生成、状态机、relations、版本、来源 | Artifact 基类字段（id/project_id/type/title/status/version/timestamps/created_by/relations/metadata） | 001 |
| K2 | Artifact Store API | 持久化接口（load/save/list/queryByRelation） | 存储接口定义，不关心实现（文件 / SQLite / 图） | 001 |
| K3 | Workflow / DAG Core | DAG 模型、节点状态机、依赖判定、事件触发 | 节点字段（task_id/issue_id/agent_profile/depends_on/status/...）、状态集合、转换规则 | 002 |
| K4 | Context Pack 协议 | Context Pack 数据结构、来源追踪、token budget | `context_pack` schema + `items[].source/type/relevance/reason` | 004 |
| K5 | Handoff 协议 | Handoff 数据结构 | result / changed / decisions / risks / verification / artifacts | 005 |
| K6 | Quality Gate 模型 | Gate 类型枚举、Gate 结果记录、Done 判定 | Gate 列表 + 判定规则（Agent completed ≠ Issue completed） | 003 |
| K7 | Event Bus | 事件发布/订阅、事件 schema | 事件类型列表（issue.ready / run.created / ...）+ payload 结构 | 006 |
| K8 | Control API / CLI | 命令定义、参数校验、输出格式 | CLI 命令树（spec / issue / run / review / test / ...） | 008 |
| K9 | Config 系统 | 分层配置（global→project→agent→issue→run）、override 解析 | 配置 schema + 解析优先级 | 009 |
| K10 | Permission / Policy 模型 | 权限声明 schema、Policy 评估接口、Human Gate 定义 | permissions schema + approval 模型 | 010 |
| K11 | Run Trace 数据结构 | Run 记录 schema、指标字段 | run schema（issue/agent/model/timestamps/token/cost/changed_files/outcome） | 007 |
| K12 | Plugin Registry | 插件注册 / 发现 / 版本管理 | 注册接口 + 命名规范 | 011 |

> 说明：00/01 中提到的 `ArtifactService`、`RunService`、`ExecutionService` 等是「内核模块 + 插件」之上的薄门面（facade），不是独立内核模块。

## 内核的可测试性

```text
内核测试不需要任何真实 Agent / 真实代码库：
- Artifact Store → 内存实现
- Runtime Provider → Mock
- Code Intelligence → 空实现
- Context Resolver → 固定输入
```

这是「内核薄」的直接收益：状态机和判定逻辑可以在没有 AI 的情况下全部测完。

---

# 4. 插件接缝（Plugin Seams）

每个插件一节，统一格式：

```text
接口    —— 插件必须实现的接口（最终契约）
默认实现 —— 首个交付的默认实现
可选实现 —— 后续可替换的实现方向（不改变接口）
交付    —— 归属哪个 Milestone 交付
```

---

## P1 Agent Runtime Provider — SPEC-101

**意图**：控制 Agent 执行——session / task 生命周期、权限协商、artifact 读取。

**接口**：

```ts
interface AgentRuntimeProvider {
  createSession(profile): Promise<SessionId>
  resumeSession(id): Promise<SessionId>
  sendTask(sessionId, task): Promise<TaskId>
  getStatus(taskId): Promise<TaskStatus>
  cancel(taskId): Promise<void>
  getOutput(taskId): Promise<Output>
  readArtifact(sessionId, path): Promise<ArtifactContent>
}
```

**默认实现**：ACP 客户端（对接 Codeg 暴露的 ACP 端点）。控制面走 ACP，数据面（worktree / diff）由 SpecOS 本地 git 处理（见 §8）。

**可选实现**：多 provider 并发池、provider 健康检查 + 自动 fallback（配合 SPEC-111 Model Router）。

**交付**：M0。

**开放**：ACP 覆盖范围与扩展点、changed files 数据源，见 §8，尚未定案。

---

## P2 Code Intelligence Provider — SPEC-102

**意图**：代码理解——索引、符号、引用、依赖、影响面。

**接口**：

```ts
interface CodeIntelligenceProvider {
  indexProject(): Promise<IndexStatus>
  findSymbol(query): Promise<Symbol[]>
  references(symbol): Promise<Reference[]>
  dependencies(fileOrModule): Promise<Dependency[]>
  impact(symbolOrFile): Promise<Impact>
  semanticSearch(query): Promise<File[]>
}
```

**默认实现**：builtin（轻量 AST / LSP 实现）。

**可选实现**：外部引擎、企业内部索引、远程 indexing service。

**交付**：M1（接口在 M0 提供空实现占位，保证编译与测试可通过）。

---

## P3 Context Resolver — SPEC-103

**意图**：从来源解析出 Context Pack。

**实现层次**（同一接口，逐层增强，不改变协议）：

- **L1（确定性装配）**：只组装显式来源（Issue + Spec + Agent Profile + 显式 files + 项目规则 + Skills + Knowledge），不做自动检索。
- **L2（Code-Intel 驱动）**：scope 解析、symbol 解析、依赖展开、相关性排序、token budget。
- **L3（学习驱动）**：基于 Run Trace / Memory 判断哪些上下文有用、哪些没有。

**交付**：L1 在 M0，L2 在 M1，L3 在 M2。

---

## P4 Agent Resolver / Team Builder — SPEC-104

**意图**：任务 → 能力 → Agent Profile → Team。

**实现层次**：

- **L1（规则）**：Issue 的 `agent_profile` 字段直接指定 / 简单规则匹配。
- **L2（能力评分）**：Capability Resolver + deterministic scoring：

```text
domain exact match       +50
required capability      +20
preferred by project     +15
related skill            +10
permission compatible    required
model available          required
```

- **L3（Eval 感知）**：结合历史成功率 / 成本 / 延迟，支持优化目标（Fast / Balanced / Quality / Budget）。

**交付**：L1 在 M0，L2 在 M1，L3 在 M2。

---

## P5 Risk Engine — SPEC-105

**意图**：评估任务风险，影响 Quality Gate 与 Human Gate。

**实现层次**：

- **L1**：Issue 显式 `risk` 字段 + 简单规则（高风险模块 / public API / migration）。
- **L2**：项目配置高风险模块 + Code Intelligence 的 change impact。
- **L3**：历史数据（失败率 / 返工率）参与评分。

**交付**：L1 在 M0，L2 在 M1，L3 在 M2。

---

## P6 Quality Checker — SPEC-106

**意图**：执行验证命令（build / lint / typecheck / test）。

**接口**：

```ts
interface QualityChecker {
  run(gate: GateType, opts): Promise<GateResult>
  availableGates(): Promise<GateType[]>
}
```

**默认实现**：项目配置 commands（`pnpm build` / `pnpm test` / `pnpm lint`），本地 runner。

**可选实现**：超时、缓存、并行、远程执行。

**交付**：M0。

---

## P7 Review Provider — SPEC-107

**意图**：独立于执行的代码审查。

**实现层次**：

- **L1**：单一 review-agent，可用不同模型，独立 Session。
- **L2**：专业 reviewer 按风险选择（security / database / api / architecture / performance）。

**交付**：L1 在 M0，L2 在 M1。

---

## P8 Storage Engine — SPEC-108

**意图**：Artifact 与索引的持久化。

**实现层次**：

- **L1**：文件系统 `.specos/`（Markdown 给人读，YAML 给机器读）。
- **L2**：SQLite 索引层——symbol、project graph、eval 聚合。
- **L3**：图结构 / 嵌入检索（如需要）。

**交付**：L1 在 M0，L2 在 M1，L3 视需要。

---

## P9 Memory Provider — SPEC-109

**意图**：从 Run 提取经验、按任务注入相关记忆。

**接口**：

```ts
interface MemoryProvider {
  extract(run): Promise<MemoryCandidate[]>
  inject(task, profile): Promise<Memory[]>
  health(): Promise<MemoryHealth>
  cleanup(policy): Promise<void>
}
```

**交付**：M2。M0/M1 提供接口占位。

---

## P10 Skill Registry / Evolution — SPEC-110

**意图**：Skill 的候选检测、验证、提升、降级、归档。

**交付**：M2。

---

## P11 Model Router — SPEC-111

**意图**：根据任务类型 / 风险 / 成本 / Eval 选择模型，含 fallback。

**交付**：M2（M0/M1 允许在 Profile 里手动指定模型，接口保持占位）。

---

## P12 Architecture Intelligence — SPEC-112

**意图**：架构地图、依赖违规 / drift 检测。

**交付**：M2。

---

## P13 Eval Aggregator — SPEC-113

**意图**：按 Agent / Model / Skill / Context 维度聚合 Run 指标。

**实现层次**：

- **L1**：只观测，不做决策。
- **L2**：作为 Router / Resolver 的输入。

**交付**：L1 在 M1，L2 在 M2。

---

# 5. 模块依赖图

```text
                         ┌──────────────┐
                         │  Control API │  K8
                         │    / CLI     │
                         └──────┬───────┘
                                │
                         ┌──────▼───────┐
                         │   Facade     │  ArtifactService / RunService / ExecutionService
                         └──────┬───────┘
                                │
   ┌────────────────────────────▼────────────────────────────┐
   │                         Kernel                          │
   │  K1 Artifact  K3 Workflow  K4 Context  K5 Handoff       │
   │  K6 Quality   K7 Event     K9 Config   K10 Policy       │
   │  K11 RunTrace K12 Registry                              │
   │                        │  K2 Store API                  │
   └───────┬────────┬───────┼────────┬─────────┬─────────────┘
           │        │       │        │         │
   ┌───────▼─┐ ┌────▼───┐ ┌─▼─────┐ ┌▼──────┐ ┌▼──────────┐
   │  Plugin │ │ Plugin │ │Plugin │ │Plugin │ │  Plugin   │
   │ Registry│ │  Seam  │ │ Seam  │ │ Seam  │ │   Seam    │
   └─────────┘ └────────┘ └───────┘ └───────┘ └───────────┘
      (K12 选择具体实现)

   具体实现（实现各自的插件接口）：
   ACP Runtime / builtin CodeIntel / file storage / sqlite /
   local checker / review agent / memory / skill / router ...
```

**依赖规则**：

```text
Kernel → 只依赖 Plugin Registry（K12）
Plugin → 依赖 Kernel 的接口与数据结构
Facade → 依赖 Kernel + 通过 Registry 拿插件
```

---

# 6. 与交付顺序（Milestone）的关系

| Milestone | 内核交付 | 插件交付 | 判定标准 |
|---|---|---|---|
| M0 | K1–K12 全量、K3/K6 状态机完整 | P1 ACP Runtime、P6 Checker、P7 L1、P8 文件存储、P3 L1、P4 L1、P5 L1 | 一条垂直切片稳定跑通 |
| M1 | K2 加 SQLite 索引支持 | P2 CodeIntel、P3 L2、P4 L2、P5 L2、P7 L2、P13 L1 | 系统能理解结构并安全并行 |
| M2 | K2 加图/嵌入支持 | P9 Memory、P10 Skill、P11 Router、P12 ArchIntel、P3 L3、P4 L3、P5 L3、P13 L2 | 系统能学习并受控自治 |

关键点：

- **内核 K1–K12 在 M0 一次建完**（契约先行），M1/M2 只加存储索引能力，不改内核契约。
- 插件按里程碑逐步成熟，未到里程碑的插件提供**空实现 / 接口占位**，保证编译与测试可通过。
- Milestone 之间的「新能力」绝大多数来自插件实现增强，而不是接口变更——这正是可插拔的意义。

---

# 7. 下一层拆分：从模块到可实践 Spec

00 是愿景，01 是完整 PRD，本文件把系统拆成模块。每个模块再拆一层，成为**可直接交付实现**的 SPEC（见 `03-SpecOS-Specs.md` 与 `specs/`）。

## 7.1 Spec 模板

每个模块一份 Spec，固定结构：

```markdown
# SPEC-xxx <模块名>

## 1. 目的与范围
## 2. 接口 / 数据结构定义（契约）
## 3. 状态机（如有）
## 4. 与依赖模块的关系
## 5. 实现方案（文件 / 服务 / 代码位置）
## 6. 数据模型与存储
## 7. 验收标准（可勾选）
## 8. 边界与不做
```

## 7.2 建议拆分顺序

按「首个垂直切片（M0）依赖」排序，先写阻塞切片的：

```text
第一批（M0 切片必需）：
  SPEC-001 Artifact System（K1+K2）
  SPEC-002 Workflow / DAG Core（K3）
  SPEC-003 Quality Gate（K6）
  SPEC-007 Run Trace（K11）

第二批（执行链路）：
  SPEC-101 Agent Runtime Provider（P1，含 ACP 方案定案）
  SPEC-004 Context Pack 协议（K4）
  SPEC-005 Handoff 协议（K5）
  SPEC-103 Context Resolver（P3）

第三批（编排与治理）：
  SPEC-006 Event Bus（K7）
  SPEC-008 Control API / CLI（K8）
  SPEC-009 Config 系统（K9）
  SPEC-010 Permission / Policy（K10）
  SPEC-011 Plugin Registry（K12）
  SPEC-104/105/106/107（P4/P5/P6/P7）

第四批（M1/M2 插件）：
  SPEC-102 Code Intelligence（P2）
  SPEC-108 Storage L2（SQLite 索引）
  SPEC-113 Eval（P13）
  SPEC-109/110/111/112（P9/P10/P11/P12）
```

## 7.3 每个 Spec 的验收标准

以 K3 Workflow / DAG Core（SPEC-002）为例：

```text
- [ ] DAG 支持依赖与拓扑判定
- [ ] 节点状态机覆盖 pending/ready/running/blocked/reviewing/verifying/completed/failed/cancelled
- [ ] 状态转换只由事件触发，不写死
- [ ] 无任何 AI / 存储依赖，可用纯内存测试
```

---

# 8. 开放问题

## 8.1 ACP 作为 Runtime Provider 的方案（待讨论）

> 方向：Codeg 的 CLI 能力改造 / 扩展为 ACP 暴露。具体方案尚未定案，以下是讨论框架。

**ACP 天然覆盖（控制面）**：

```text
Session 生命周期     initialize / session created / destroyed
Task 生命周期        task/send / task/cancel / progress updates
权限协商             permission/request / permission/grant
文件读取             artifact/read / artifact/write / FileView
```

**SpecOS 自持（数据面）**：

```text
Worktree 创建/删除/合并   本地 git（worktree 本来就是 SpecOS 建的）
Changed files / diff     本地 git diff
Context 组装             prompt 层面，不进协议
Build/Test 验证          项目命令，本地 runner
```

**决策点（D1–D5）**：

```text
D1  Codeg 是否原生暴露 ACP server？还是需要一层包装？
D2  changed files 从哪拿——ACP artifact 事件 vs 本地 git diff？
    （倾向本地 git：worktree 自己建的，git 一定最准）
D3  Review 是否需要独立 Session？同一 provider 另一个 session，还是强制不同 provider？
D4  cancel / interrupt 语义在 ACP 上如何映射？
D5  单 provider 起步，多 provider 并发池何时引入？（M1?）
```

## 8.2 其他待定

```text
- Plugin Registry 的配置方式（YAML 声明 vs 代码注册）
- Artifact ID 命名空间（SPEC-xxx / ISSUE-xxx / RUN-xxx 是否全局唯一）
- 内核是否独立成 crate / package，还是与现有代码同仓
- M0 的 .specos 目录与仓库已有历史实现的关系（历史实现仅作参考）
```

---

# 9. 本文档要回答的问题

用一句话回顾本文件的价值：

> 拆完之后，任何一块工作都能回答「我在改内核还是改插件」「我这个插件要保证哪些契约」「我这块能不能单独测试」。
