# UI/UX — Memory Hub 完整交互规范

# 1. Navigation

Desktop 左侧主导航：

```text
Chat
Tasks

Project
 ├─ Memory
 ├─ Skills
 ├─ Wiki
 ├─ Code
 └─ Agents

Settings
```

也可在 Project 下使用一个统一 `Memory Hub` 页签容器。

---

# 2. Chat Toolbar

显示：

```text
Memory ●
Context 8.4k
Task Active
```

Memory 状态：

```text
Healthy
Degraded
Offline
Disabled
Indexing
```

---

# 3. Context Used

每个 turn 的 assistant message 菜单：

```text
View Context Used
```

Drawer：

```text
Context Used — Turn 284

Rules                 1,210 tokens
Task Canvas              420
Memory                 1,730
Skills                   910
Wiki                   1,120
CodeGraph              1,380

Total                  6,770
```

展开每项看到具体记录及 source。

---

# 4. Memory Page

顶部：

```text
Search memories...
[All] [Facts] [Scenarios] [Persona] [Conversation]
```

Filters：

- scope
- agent
- task
- date
- validity
- source
- recalled recently

---

# 5. Memory Detail

显示：

```text
Content
Layer
Type
Scope
Confidence
Validity
Created
Updated
Last recalled
Recall count
```

Evidence：

```text
Persona
 ↓
Scenario
 ↓
Atom
 ↓
Conversation
```

操作：

```text
Correct
Supersede
Mark incorrect
Delete
Copy
Open source session
```

---

# 6. Persona Editor

Persona 是可读 Markdown，但用户修改应生成 overlay/revision。

页面：

```text
Current Persona
History
Sources
```

不要直接无版本修改文件。

---

# 7. Task Memory

Tasks 页面：

```text
Active
Paused
Completed
Failed
```

Task detail：

- Canvas
- timeline
- refs
- changed files
- participating agents
- next actions
- Resume

Canvas 可 Mermaid render + raw toggle。

---

# 8. Skill Manager

列表：

```text
Enabled
Draft
Candidate
Deprecated
Failed validation
```

Detail：

- trigger
- steps
- resources
- validation
- source traces
- versions
- execution history

Actions：

```text
Enable
Disable
Edit draft
Validate
Publish
Rollback
Delete
```

---

# 9. Skill Candidate Inbox

类似 review inbox：

```text
3 new skill candidates
```

每个 candidate：

```text
Why detected
Repeated tasks
Expected benefit
Similar skill
Risk
Validation result
```

用户一键：

```text
Approve
Merge
Reject
Ignore pattern
```

---

# 10. Wiki

布局：

```text
Tree | Page | Sources
```

Search 支持：

- semantic
- exact text
- title
- tag

Page header：

```text
Generated from 4 sources
Last rebuilt 5m ago
Current / Stale
```

---

# 11. CodeGraph

完整 IDE 图谱不必首版做炫酷无限画布。

优先高价值 UI：

### Symbol Search

```text
PaymentService.retry
```

### Relationship

```text
Callers
Callees
References
Tests
Imports
```

### Impact

按钮：

```text
Analyze Impact
```

显示：

```text
Direct: 4
Transitive: 12
Tests: 7
Risk: Medium
```

必要时再加 graph visualization。

---

# 12. Sources

统一资产来源：

```text
Repository
Docs folder
ADR folder
External URL
Manual note
```

显示：

```text
status
revision
last sync
items
errors
```

---

# 13. Agent Loadout UI

Agent 页面用 chips：

```text
Memory: Project + User
Skills: 6
Wiki: Architecture + API
CodeGraph: Full
```

点击编辑。

---

# 14. Settings

## Global

```text
Memory Engine
Storage
Extraction LLM
Embedding
Auto Start
Backup
Telemetry
Privacy
```

## Project

```text
Memory enabled
Task offload
Wiki
CodeGraph
Skill discovery
Retention
Excluded files
```

---

# 15. Diagnostics

必须内置，不要让用户自己翻日志。

```text
MemoryCore      Healthy
Knowledge       Healthy
Gateway         24ms
DB              OK
Pipeline L1     Idle
Pipeline L2     Running
Pipeline L3     Idle
Wiki            Ready
CodeGraph       Ready
Jobs            2 running / 0 failed
```

Actions：

```text
Run health check
Open logs
Restart service
Reindex Wiki
Rebuild CodeGraph
Export diagnostics
```

---

# 16. Backup UI

Settings → Memory → Backup

```text
Last backup
Backup location
Size
Schema version
Provider version
```

Actions：

```text
Backup now
Restore
Export
```

Restore 必须先预检版本。

---

# 17. Onboarding

首次启用不要暴露 40 个参数。

Wizard：

```text
1. Enable Memory
2. Local / Remote
3. Extraction model
4. Project sources
5. Privacy
6. Build
```

高级参数放 Advanced。

---

# 18. Notifications

只提示需要用户处理的：

- provider cannot start
- migration required
- repeated job failure
- backup failure
- skill candidate ready（可关）
- knowledge stale

不要每次 L1 extraction 都 toast。

---

# 19. Empty States

Memory：

```text
No memories yet.
They will be built automatically as you work.
```

Skill：

```text
No reusable skills detected yet.
```

Wiki：

```text
Add project documentation sources to build Wiki.
```

---

# 20. UX Acceptance

- [ ] 用户能在 3 点击内找到一个 memory 的原始来源
- [ ] 用户能知道当前 turn 用了哪些 memory
- [ ] 用户能停用整个系统
- [ ] 用户能独立停用 Wiki/Skill/CodeGraph
- [ ] 错误状态不会只存在日志
- [ ] service restart 不要求命令行
- [ ] backup/restore 可从 UI 做

---

## 上游参考与实现注意

本方案依据 2026-08-18 可见的 TencentCloud/TencentDB-Agent-Memory 项目设计整理，重点参考：

- `README.md` / `README_CN.md`
- Releases 1.x / 2.0.0-beta.1
- `MemoryCore/README.md` (`feat/server_team`)
- Codex / Claude Code / OpenCode adapter work

上游：
https://github.com/TencentCloud/TencentDB-Agent-Memory

实现时必须固定经过测试的 tag/commit，不应直接依赖持续变化的 branch HEAD。
