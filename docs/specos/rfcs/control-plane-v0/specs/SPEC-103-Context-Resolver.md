# SPEC-103 — Context Resolver

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §6（D3）；`02` P3；`03` 里程碑 M0（L1）/ M1（L2）/ M2（L3）
> 依赖：SPEC-004（协议）、SPEC-001
> 实现语言：不限定；接口契约为本 SPEC 主体

---

# 1. 目的与范围

定义 Context Pack 的装配插件。同一接口逐层增强，协议（SPEC-004）不变。

---

# 2. 接口（契约）

```ts
interface ContextResolver {
  compile(issue: Issue, profile: AgentProfile, opts: CompileOptions): Promise<ContextPack>
  inspect(contextPackId): Promise<ContextPackDebug>
}
```

```ts
interface CompileOptions {
  explicitFiles?: string[]
  codeIntel?: CodeIntelligenceProvider   // L2 起注入
  memory?: MemoryProvider                // L3 起注入
  tokenBudget?: number
}
```

---

# 3. 实现层次

## L1 — 确定性装配（M0）

只组装显式来源，不做自动检索：

```text
Issue + Spec + Agent Profile + 显式 files + 项目规则 + Skills + Knowledge
```

重点是协议稳定与来源可追踪。

## L2 — Code-Intel 驱动（M1）

```text
Issue
 ↓
Scope Resolver        # 从 scope/impact 找目标模块
 ↓
Artifact Resolver     # 相关 Spec/AC/ADR
 ↓
Symbol Resolver       # 直接影响的符号
 ↓
Dependency Expansion  # 依赖展开
 ↓
Rule / Skill Resolver
 ↓
Rank                  # relevance 评分
 ↓
Token Budget          # 裁剪，记录 excluded
 ↓
Context Pack
```

## L3 — 学习驱动（M2）

基于 Run Trace / Memory 判断哪些上下文有用、哪些没有（Context Optimizer，见 SPEC-113 与 SPEC-109）。

---

# 4. 可调试性

- `context inspect`（CLI/UI）展示 required / selected / excluded / token budget / 来源原因。
- 这是调试 Agent 质量的关键工具。

---

# 5. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-004 | 产出协议 |
| SPEC-102 | L2 消费 symbol/dependency/impact |
| SPEC-109 | L3 注入 Memory |
| SPEC-007 | L3 消费历史 Run |

---

# 6. 验收标准

- [ ] L1：确定性装配，来源可追踪。
- [ ] L2：自动选相关代码，relevance reason 与 token budget。
- [ ] L3：学习哪些上下文有用/无用。
- [ ] `context inspect` 可解释选择/排除。

---

# 7. 边界与不做

- 不做协议定义（SPEC-004）。
- 不做检索实现（SPEC-102）。
- 不做 Memory 注入策略（SPEC-109）。
