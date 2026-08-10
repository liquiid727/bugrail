# SPEC-112 — Architecture Intelligence

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §11（D8）；`02` P12；`03` 里程碑 M2
> 依赖：SPEC-102（Code Intelligence）、SPEC-108（存储）
> 实现语言：不限定；接口契约为本 SPEC 主体

---

# 1. 目的与范围

定义架构理解插件：维护架构地图、检测依赖违规 / drift，并在跨模块 / 高风险场景建议 Architecture Node。

---

# 2. 接口（契约）

```ts
interface ArchitectureIntelligence {
  getMap(): Promise<ArchitectureMap>
  detectDrift(): Promise<DriftReport[]>
  impact(change: ChangeRef): Promise<Impact>
  suggestArchitectureNode(change: ChangeRef): Promise<Suggestion | null>
}
```

---

# 3. 数据模型

## 3.1 架构地图（基于 SPEC-102 §3.3）

```text
modules
boundaries
dependencies
layers
public APIs
data ownership
service ownership
risk zones
```

## 3.2 Drift 报告

```yaml
drift:
  type: circular_dependency | layer_violation | architecture_drift | unexpected_coupling
  from: backend/order
  to: payment/legacy
  detail: ...
  severity: low | medium | high
  suggested_action: 重构边界 / 新增 ADR / 插入 Architecture Node
```

---

# 4. 触发 Architecture Node

当检测到：

```text
cross-module
new service
new database
public API
high dependency fanout
architecture boundary change
```

自动建议或插入 Architecture Node，输出：

- Architecture Artifact
- ADR
- Impact
- required Issues
- migration strategy

---

# 5. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-102 | 消费符号/依赖/影响面 |
| SPEC-108 | 存储架构图 |
| SPEC-002 | 插入 Architecture 节点 |
| SPEC-105 | 影响风险 |
| SPEC-104 | 影响 Team 生成 |

---

# 6. 验收标准

- [ ] 可维护架构地图（modules/boundaries/layers/public APIs）。
- [ ] 可识别基础 dependency drift（循环/层违例/新耦合）。
- [ ] 跨模块/高危变更可建议 Architecture Node。
- [ ] 输出含 ADR 与 migration strategy。

---

# 7. 边界与不做

- 不做代码索引（SPEC-102）。
- 不做风险评分（SPEC-105）。
- 不做自动重构（只建议）。
