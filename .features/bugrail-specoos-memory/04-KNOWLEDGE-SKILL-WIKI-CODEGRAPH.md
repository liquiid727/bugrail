# KNOWLEDGE — Skill / Wiki / CodeGraph 完整设计

# 1. 三者分工

```text
Chat Memory = 我们过去知道/决定了什么
Wiki        = 项目文档知识是什么
CodeGraph   = 当前代码事实和结构是什么
Skill       = 遇到某类任务时应该怎么做
```

禁止混为一个向量库。

---

# 2. Knowledge Registry

每个 source：

```ts
KnowledgeSource {
  id
  projectId
  type: "wiki" | "codegraph" | "docs" | "external"
  name
  rootUri
  status
  revision
  lastIndexedAt
  providerId
  excludePatterns[]
}
```

状态：

```text
new
indexing
ready
stale
error
disabled
```

---

# 3. Wiki Pipeline

```text
Source
  ↓
Discover
  ↓
Parse
  ↓
Normalize
  ↓
Page/Section extraction
  ↓
Link graph
  ↓
Index
  ↓
Publish revision
```

---

# 4. Wiki Incremental Update

触发：

```text
files.changed
git.commit.detected
manual refresh
scheduled integrity check
```

只重建 changed source 的影响页面。

---

# 5. Wiki Page

```yaml
id:
title:
summary:
source_refs:
revision:
updated_at:
tags:
links:
entities:
```

正文保持 Markdown。

---

# 6. Wiki Source Priority

当 Wiki 和源码冲突：

```text
current source code > generated Wiki
```

当 Wiki 和 ADR 冲突：

```text
newer accepted ADR > generated summary
```

所以上下文中必须携带 source timestamp/revision。

---

# 7. CodeGraph Model

节点：

```text
Repo
Package
Directory
File
Symbol
Class
Interface
Function
Method
API
Test
```

边：

```text
contains
imports
calls
references
implements
extends
tests
depends_on
exports
```

---

# 8. CodeGraph Queries

```ts
findSymbol()
findReferences()
findCallers()
findCallees()
dependencyPath()
impactAnalysis()
neighborhood()
changedFileContext()
```

---

# 9. Impact Analysis

输入：

```text
symbol/file
change type
optional diff
```

输出：

```text
direct dependents
transitive dependents
tests
API consumers
risk areas
confidence
evidence
```

Plan Agent 应默认在跨模块更改前调用。

---

# 10. CodeGraph Refresh

首次：

```text
full scan
```

后续：

```text
git diff / file watcher
  ↓
changed files
  ↓
reparse
  ↓
update affected nodes
  ↓
invalidate affected edges
```

避免全量 rebuild。

---

# 11. Skill Model

完整 Skill：

```yaml
apiVersion: specos/v1
kind: Skill

metadata:
  id:
  name:
  version:
  scope:
  owner:
  status:

trigger:
  intents: []
  filePatterns: []
  technologies: []
  conditions: []

negativeTrigger:
  conditions: []

inputs:
  required: []
  optional: []

procedure:
  steps: []

resources:
  files: []
  templates: []
  commands: []
  references: []

validation:
  checks: []
  requiredEvidence: []

recovery:
  rollback: []
  knownFailures: []

provenance:
  sourceTasks: []
  sourceSessions: []
  sourceMemories: []
```

---

# 12. Skill Discovery

候选输入：

- 成功 task trace
- repeated workflows
- repeated tool sequence
- recurring problem/solution scenarios
- manually marked “save as skill”

不是简单总结用户意图。

---

# 13. Candidate Scoring

```text
repeatability
successRate
specificity
costSaving
stability
validationStrength
scopeSafety
```

达到阈值才生成 Draft。

---

# 14. Skill Candidate Pipeline

```text
Trace
  ↓
Pattern Detection
  ↓
Candidate Draft
  ↓
Find Similar Skills
  ↓
Merge or New
  ↓
Sandbox Validation
  ↓
Review
  ↓
Publish
```

---

# 15. 自动化等级

```text
OFF
DISCOVER_ONLY
AUTO_DRAFT
AUTO_VALIDATE
AUTO_PUBLISH
```

默认推荐：

```text
AUTO_VALIDATE
```

但 `AUTO_PUBLISH` 只允许低风险、可自动验证 Skill。

---

# 16. Skill Versioning

语义：

```text
major = 行为/接口不兼容
minor = 新步骤/能力
patch = 文案/小修
```

每次运行记录使用版本。

失败可回滚。

---

# 17. Skill Routing

Context Router 根据：

- user intent
- current files
- task type
- agent role
- technologies
- previous successful usage

匹配 Skill。

只注入 Top-K。

---

# 18. Skill vs Rules

Rules 是必须遵守。

Skill 是可选择执行方法。

不能把 Skill 当系统 policy。

---

# 19. Knowledge + Skill 联动

Skill 可以引用 Wiki/CodeGraph：

```yaml
resources:
  wiki:
    - release-process
  codeQueries:
    - "find callers of ..."
```

执行时动态获取最新知识，而非把源码固化进 Skill。

---

# 20. 更新联动

```text
code changed
  ↓
CodeGraph incremental

docs changed
  ↓
Wiki incremental

successful task
  ↓
Skill candidate

conversation closed
  ↓
Memory extraction
```

这些是独立 pipeline，由 Job Scheduler 编排。

---

# 21. 完整验收

## Wiki

- [ ] import docs
- [ ] generated pages readable
- [ ] source links work
- [ ] search works
- [ ] stale detection works
- [ ] incremental update works

## CodeGraph

- [ ] symbol index
- [ ] reference/call lookup
- [ ] impact analysis
- [ ] changed-file refresh
- [ ] evidence points to actual files

## Skill

- [ ] candidate created from trace
- [ ] similar skill detection
- [ ] validation
- [ ] approval
- [ ] enable/disable
- [ ] version
- [ ] rollback
- [ ] source trace

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
