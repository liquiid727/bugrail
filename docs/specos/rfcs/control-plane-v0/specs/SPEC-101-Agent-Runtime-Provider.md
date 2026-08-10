# SPEC-101 — Agent Runtime Provider（ACP）

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §5（D2）；`02` P1；`03` 里程碑 M0
> 依赖：SPEC-010（权限）、SPEC-001
> 实现语言：不限定；接口契约为本 SPEC 主体

---

# 1. 目的与范围

定义控制 Agent 执行的插件接口：session / task 生命周期、权限协商、artifact 读取。默认实现为 ACP 客户端（对接 Codeg）。

上层 Workflow（SPEC-002）不直接依赖某个具体 CLI。

---

# 2. 接口（契约）

```ts
interface AgentRuntimeProvider {
  createSession(profile: AgentProfile): Promise<SessionId>
  resumeSession(id: SessionId): Promise<SessionId>
  sendTask(sessionId: SessionId, task: TaskContract): Promise<TaskId>
  getStatus(taskId: TaskId): Promise<TaskStatus>
  cancel(taskId: TaskId): Promise<void>
  getOutput(taskId: TaskId): Promise<Output>
  readArtifact(sessionId: SessionId, path: string): Promise<ArtifactContent>
}
```

```ts
interface TaskContract {
  issue_id: string
  context_pack_id: string      // SPEC-004
  goal: string
  acceptance_criteria: string[]
  constraints: string[]
  permissions: Permissions      // SPEC-010
}
```

---

# 3. 控制面 vs 数据面

```text
控制面（走 ACP）：
  Session 生命周期     initialize / session created / destroyed
  Task 生命周期        task/send / task/cancel / progress updates
  权限协商             permission/request / permission/grant
  文件读取             artifact/read / artifact/write / FileView

数据面（SpecOS 自持，本地 git）：
  Worktree 创建/删除/合并
  Changed files / diff
  Context 组装（prompt 层，不进协议）
  Build/Test 验证（本地 runner，SPEC-106）
```

## 3.1 决策点（开放，见 02 §8.1）

```text
D1  Codeg 是否原生暴露 ACP server？
D2  changed files 从哪拿（倾向本地 git diff）？
D3  Review 是否独立 Session？
D4  cancel/interrupt 在 ACP 上如何映射？
D5  多 provider 并发池何时引入（M1?）？
```

---

# 4. Worktree 生命周期

```text
allocated → active → ready-to-merge → merged → cleanup
```

- 默认 1 Issue = 1 Worktree + 1 执行 Session，允许附加 Review Session。
- 从 UI 可打开对应 Worktree / Session（SPEC-008）。

---

# 5. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-010 | 执行前权限协商 |
| SPEC-004 | 接收 Context Pack |
| SPEC-005 | 完成后生成 Handoff 输入 |
| SPEC-007 | 记录 run 会话 |
| SPEC-002 | 节点执行经此触发 |

---

# 6. 验收标准

- [ ] 实现接口 createSession / resumeSession / sendTask / getStatus / cancel / getOutput / readArtifact。
- [ ] 控制面走 ACP，数据面走本地 git。
- [ ] Worktree 生命周期完整。
- [ ] 权限协商生效，越权任务被拒绝。
- [ ] 上层不直接依赖具体 CLI。

---

# 7. 边界与不做

- 不做 Context 装配（SPEC-103）。
- 不做验证（SPEC-106）。
- 不做模型路由（SPEC-111）。
