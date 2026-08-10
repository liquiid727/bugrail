# SPEC-109 — Memory Provider

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §11（D8）；`02` P9；`03` 里程碑 M2
> 依赖：SPEC-007（Run Trace）、SPEC-011（Registry）
> 实现语言：不限定；接口契约为本 SPEC 主体

---

# 1. 目的与范围

定义经验提取与注入插件：从 Run 提取可复用经验，按任务注入相关记忆。

**原则**：自动记忆不等于自动生成 Skill；先分类，再通过证据与验证决定是否提升（见 SPEC-110）。

---

# 2. 接口（契约）

```ts
interface MemoryProvider {
  extract(run: Run): Promise<MemoryCandidate[]>
  inject(task: Issue, profile: AgentProfile): Promise<Memory[]>
  health(): Promise<MemoryHealth>
  cleanup(policy: CleanupPolicy): Promise<void>
}
```

---

# 3. 数据模型

## 3.1 Memory 分类

```text
Fact            项目事实（项目使用 pnpm，Node 24）
Rule            工程规范（Controller 不允许直接访问 Repository）
Decision        技术决策（Refund 用 Saga 而非分布式事务）
Pattern         统计上的重复模式
Preference      用户偏好
Failure Lesson  失败教训
Skill Candidate 可执行 workflow 候选（见 SPEC-110）
```

## 3.2 Memory item

```yaml
memory:
  id: MEM-21
  type: Fact | Rule | Decision | Pattern | Preference | FailureLesson | SkillCandidate
  scope: project | module | agent | system
  content: ...
  source_runs: [RUN-12, RUN-31]
  confidence: 0.88
  last_used:
  usage_count: 3
  success_association: 0.92
  conflicts_with: [MEM-05]
```

---

# 4. 提取流程

```text
Run
 ↓
Outcome Analyzer
 ↓
Extract Candidate Memories
 ↓
Deduplicate
 ↓
Conflict Check
 ↓
Confidence Update
```

## 注入策略

- 不全部注入。
- Context Compiler（SPEC-103 L3）按 task scope / agent role / module / artifact / risk / historical relevance 选择。

---

# 5. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-007 | 消费 Run Trace |
| SPEC-103 | L3 注入 |
| SPEC-110 | 提供 Pattern / Skill Candidate |
| SPEC-113 | 使用 success_association 指标 |

---

# 6. 验收标准

- [ ] Run 结束可自动产生 Memory Candidates。
- [ ] Memory 可分类，含来源 / 置信度 / 冲突。
- [ ] 注入按任务选择，不全量。
- [ ] 有健康度与清理策略（stale/degraded）。

---

# 7. 边界与不做

- 不做 Skill 验证/提升（SPEC-110）。
- 不做指标聚合（SPEC-113）。
