# SPEC-106 — Quality Checker

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §9（D6）；`02` P6；`03` 里程碑 M0
> 依赖：SPEC-003
> 实现语言：不限定；接口契约为本 SPEC 主体

---

# 1. 目的与范围

定义验证命令执行插件：build / lint / typecheck / test 的本地 runner，为 Quality Gate（SPEC-003）产出 GateResult。

---

# 2. 接口（契约）

```ts
interface QualityChecker {
  run(gate: GateType, opts: RunOpts): Promise<GateResult>
  availableGates(): Promise<GateType[]>
}
```

```ts
interface RunOpts {
  issueId: string
  runId: string
  cwd: string                // worktree 路径
  command?: string           // override 项目默认命令
  timeoutMs?: number
}
```

---

# 3. 命令配置

```yaml
# .specos/project.yaml（SPEC-009）
quality:
  commands:
    build: pnpm build
    test: pnpm test
    lint: pnpm lint
```

Issue 可 override（SPEC-001 `commands`）。

---

# 4. 语义

- 每个 gate 一次 run 产出 `GateResult`（SPEC-003 §2.2）。
- 失败需带错误摘要与退出码。
- 支持超时（默认可配）、缓存（同输入跳过）、并行（多 gate 并发）。
- 结果必须落盘作为证据（SPEC-001 Test Artifact / Run Trace）。

---

# 5. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-003 | 产出 GateResult |
| SPEC-009 | 读取 commands |
| SPEC-001 | 落盘 Test Artifact |
| SPEC-002 | 验证阶段触发 |

---

# 6. 验收标准

- [ ] 支持 build/lint/typecheck/test 执行。
- [ ] 命令可项目配置与 Issue override。
- [ ] 结果含退出码、摘要、证据引用。
- [ ] 支持超时与缓存。
- [ ] 结果落盘可追踪。

---

# 7. 边界与不做

- 不做 Review（SPEC-107）。
- 不做 Done 判定（SPEC-003）。
- 不做远程执行（可选实现）。
