# SPEC-110 — Skill Evolution

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §11（D8）；`02` P10；`03` 里程碑 M2
> 依赖：SPEC-007、SPEC-011、SPEC-113
> 实现语言：不限定；接口契约为本 SPEC 主体

---

# 1. 目的与范围

定义 Skill 的候选检测、验证、提升、降级、归档全生命周期。

**禁止**：一次 Agent 执行 = 自动 Skill。只有重复、稳定、有证据、可泛化的行为才可能成为 Skill。

---

# 2. 接口（契约）

```ts
interface SkillRegistry {
  list(): Promise<Skill[]>
  register(skill: Skill): Promise<SkillId>
  validate(candidateId, via: ReplayResult): Promise<ValidationResult>
  promote(candidateId): Promise<Skill>
  deprecate(skillId, reason): Promise<void>
  archive(skillId): Promise<void>
  health(): Promise<SkillHealth>
}
```

---

# 3. Skill Candidate 模型

```yaml
skill_candidate:
  id: SKILL-C-12
  name: prisma-migration-workflow
  purpose: 修改 Prisma schema 后的标准流程
  trigger: 检测到 schema.prisma 变更
  scope: project
  steps:
    - 1. generate
    - 2. migration
    - 3. update repository
    - 4. update tests
  source_runs: [RUN-12, RUN-31, RUN-42]
  success_rate: 0.92
  confidence: 0.88
  risks: []
  required_tools: [prisma, pnpm]
  validation_status: candidate | testing | active | validated | degraded | deprecated | archived
```

---

# 4. 演化管线

```text
Observation → Pattern → Candidate → Evidence Accumulation
→ Offline Validation → Shadow Run → Human/Policy Approval
→ Active Skill → Continuous Eval → Validated / Deprecated
```

## 候选阈值

```text
至少 N 次重复
+ 成功率达到阈值
+ 步骤具有稳定性
+ 适用条件可描述
```

## 自动降级条件

- success rate 下降
- project architecture changed
- dependencies changed
- repeated review rejection
- tool unavailable
- rule conflict

---

# 5. 验证与安全

- **Shadow Run / Replay**：同一历史 Task 用新 Skill 与基线比较（output/tests/review/cost/latency），这是 Skill 自动升级的重要安全机制。
- 提升需 Human / Policy 审批（SPEC-010）。

---

# 6. 冲突解析

```text
explicit current task > project rule > module rule > project skill > global skill
```

冲突不静默覆盖，可在 Inspector 查看。

---

# 7. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-109 | Pattern / Candidate 来源 |
| SPEC-007 | Replay 数据源 |
| SPEC-113 | success/failure 指标 |
| SPEC-010 | 提升审批 |

---

# 8. 验收标准

- [ ] 一次 Run 不直接提升为正式 Skill。
- [ ] Skill 有完整生命周期与版本。
- [ ] 有 usage / success / failure 指标。
- [ ] 新 Skill 通过 Replay / Shadow Run 验证。
- [ ] 冲突解析不静默覆盖。
- [ ] 自动降级条件生效。

---

# 9. 边界与不做

- 不做记忆提取（SPEC-109）。
- 不做指标聚合（SPEC-113）。
