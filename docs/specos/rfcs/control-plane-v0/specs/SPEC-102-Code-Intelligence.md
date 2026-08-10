# SPEC-102 — Code Intelligence Provider

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §7（D4）；`02` P2；`03` 里程碑 M1
> 依赖：SPEC-011（Registry）、SPEC-108（存储）
> 实现语言：不限定；接口契约为本 SPEC 主体

---

# 1. 目的与范围

定义代码理解插件接口：索引、符号、引用、依赖、影响面、语义检索。默认实现为 builtin（轻量 AST/LSP）。

Codebase 能力不只做向量检索，要提供结构化图查询。

---

# 2. 接口（契约）

```ts
interface CodeIntelligenceProvider {
  indexProject(): Promise<IndexStatus>
  findSymbol(query: SymbolQuery): Promise<Symbol[]>
  getDefinition(symbol): Promise<Definition>
  getReferences(symbol): Promise<Reference[]>
  getImplementations(symbol): Promise<Symbol[]>
  getCallers(symbol): Promise<Symbol[]>
  getCallees(symbol): Promise<Symbol[]>
  getDependencies(fileOrModule): Promise<Dependency[]>
  getTests(symbolOrFile): Promise<Test[]>
  getGitHistory(fileOrSymbol): Promise<CommitInfo[]>
  impact(symbolOrFile): Promise<Impact>
  semanticSearch(query): Promise<File[]>
}
```

---

# 3. 数据模型

## 3.1 Symbol

```yaml
symbol:
  id:
  language:
  kind:              # function/class/module/interface/...
  name:
  qualified_name:
  file:
  range:
  module:
  parent:
  signature:
```

关系：`defines / references / calls / implements / extends / imports / exports / tests`。

## 3.2 Impact 结果

```yaml
impact:
  target: PaymentService.refund
  direct_callers: [OrderService, RefundRepository]
  dependent_modules: [payment, order]
  tests: [refund.test.ts]
  api_surface: [POST /refund]
  related_specs: [SPEC-021]
  related_issues: [ISSUE-101]
```

## 3.3 Architecture Map

```yaml
architecture_map:
  modules: [frontend, backend, payment, order, identity]
  dependencies:
    - from: backend
      to: payment
  entrypoints: [src/main.ts, src/app.ts]
  public_apis: [POST /refund, GET /order]
  datastores: [postgres]
  external_services: [stripe]
  test_layout: [src/**/*.test.ts]
```

---

# 4. 实现层次

- **L1（索引与基础查询）**：filesystem + git + AST/Symbol 索引，支持定义/引用/依赖。
- **L2（图与影响面）**：Reference/Dependency 图、Change Impact、Architecture Map。
- **可选实现**：外部引擎、企业内部索引、远程 indexing service（经 SPEC-011 Registry 替换，不改接口）。

---

# 5. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-011 | 作为插件注册，可替换 |
| SPEC-103 | Context L2 消费 symbol/dependency/impact |
| SPEC-105 | Risk L2 消费 impact |
| SPEC-112 | 架构图基于本索引 |

---

# 6. 验收标准

- [ ] 项目可完成首次索引。
- [ ] 支持 Definition / References / Dependencies / Impact 查询。
- [ ] Impact 返回 callers/modules/tests/api/related artifacts。
- [ ] Architecture Map 可生成初版。
- [ ] Provider 可替换，不影响内核。

---

# 7. 边界与不做

- 不做 Context 装配（SPEC-103）。
- 不做架构违规检测（SPEC-112，基于本索引的上层）。
- 不做嵌入/向量存储实现（可作可选实现）。
