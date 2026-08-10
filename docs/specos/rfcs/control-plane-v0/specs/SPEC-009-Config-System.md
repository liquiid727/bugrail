# SPEC-009 — Config 系统

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §13（D10）；`02` K9；`03` 里程碑 M0
> 依赖：SPEC-001
> 实现语言：不限定；本 SPEC 只定义契约

---

# 1. 目的与范围

定义 SpecOS 的分层配置系统：配置来源、解析优先级、override 语义。配置文件以 `.specos/project.yaml` 为核心。

---

# 2. 配置分层（契约）

```text
Global Defaults      ~/.specos/config.yaml
     ↓
Project Config       .specos/project.yaml
     ↓
Agent Profile        .specos/agents/*.yaml
     ↓
Issue Override       issue.metadata / issue.yaml
     ↓
Run Override         run 级参数
```

后层覆盖前层；每个键记录来源，支持 explain。

---

# 3. project.yaml schema

```yaml
project:
  name: specos-demo
  type: fullstack

workflow:
  default: standard          # L0..L5 或自定义

runtime:
  provider: codeg            # 或 future: acp / claude / codex

quality:
  default:
    - build
    - review
  commands:
    build: pnpm build
    test: pnpm test
    lint: pnpm lint

agents:
  execution: backend-agent
  review: review-agent

orchestration:
  max_parallel_tasks: 4
  auto_team: true
  providers:
    openai: { max_parallel: 2 }
    anthropic: { max_parallel: 2 }

risk:
  high_risk_modules: [payment, auth]
  require_human_merge: [high, critical]

code_intelligence:
  provider: builtin

context:
  max_tokens: 60000

review:
  policies:
    security:
      when: [payment, auth]

autonomy:
  mode: balanced              # Manual/Assisted/Balanced/Autonomous/Strict
  auto:
    create_issues: true
    create_worktrees: true
    start_agents: true
    run_tests: true
  approval:
    architecture: true
    high_risk_spec: true
    high_risk_merge: true
    release: true
```

---

# 4. 解析语义

- 标量后层覆盖前层。
- 数组默认合并（可 `override: replace` 强制替换）。
- 每个解析结果可输出来源链：`config explain KEY`。
- 未知键：允许但告警；非法值：拒绝启动。

---

# 5. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-001 | 读取 issue 级 override（metadata） |
| SPEC-010 | 权限声明纳入配置 |
| SPEC-105 | 读取 high_risk_modules |
| SPEC-111 | 读取 model_policy |

---

# 6. 验收标准

- [ ] 五层配置解析，后层覆盖前层。
- [ ] 数组合并/替换语义明确。
- [ ] 每个键可 explain 来源。
- [ ] 非法配置拒绝启动。

---

# 7. 边界与不做

- 不做权限策略（SPEC-010）。
- 不做命令执行（SPEC-106）。
