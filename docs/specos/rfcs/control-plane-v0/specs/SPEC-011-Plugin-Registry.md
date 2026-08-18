# SPEC-011 — Plugin Registry

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §13（D10）；`02` K12；`03` 里程碑 M0
> 依赖：SPEC-001
> 实现语言：不限定；本 SPEC 只定义契约

---

# 1. 目的与范围

定义 SpecOS 的插件注册 / 发现 / 版本管理。内核只能通过 Registry 拿到插件实例，不能直接 import 具体实现（依赖倒置，见 02 §P1）。

---

# 2. 契约

## 2.1 插件声明

```yaml
plugin:
  id: code-intelligence.builtin
  seam: CodeIntelligenceProvider     # 对应对接缝接口
  version: 1.0.0
  entrypoint: specos-codeintel-builtin
  implements: [indexProject, findSymbol, references, dependencies, impact, semanticSearch]
  provides: [code_intelligence]
  config_schema: ...
```

## 2.2 注册接口

```ts
interface PluginRegistry {
  register(plugin: PluginSpec): Promise<void>
  discover(seam: SeamId): Promise<PluginSpec[]>
  resolve(seam: SeamId, constraints?: { provider?: string }): Promise<PluginInstance>
  health(): Promise<Record<PluginId, PluginHealth>>
}
```

## 2.3 解析规则

- 按配置（SPEC-009 `code_intelligence.provider` 等）选择实现。
- 多个实现可选时按版本与优先级解析。
- 未匹配到实现 → 返回空实现 / 接口占位（保证编译与测试可通过）。

## 2.4 命名规范

```text
<seam>.<impl>        # 如 code-intelligence.builtin
<seam>.<org>.<name>  # 如 code-intelligence.acme.internal
```

---

# 3. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-009 | 读取 provider 配置选择实现 |
| SPEC-101..113 | 每个插件 seam 通过 Registry 暴露 |
| SPEC-001 | 插件元数据可存为 Artifact |

---

# 4. 验收标准

- [ ] 插件可注册 / 发现 / 解析。
- [ ] 内核不直接 import 具体实现。
- [ ] 未实现 seam 返回占位，不影响编译与测试。
- [ ] 可按配置选择不同实现。

---

# 5. 边界与不做

- 不做具体插件的实现。
- 不做插件生命周期管理（升级/卸载）。
