# SPEC-108 — Storage Engine

> 状态：Draft
> 对应上位文档：`01-SpecOS-PRD.md` §4（D1）；`02` P8；`03` 里程碑 M0（L1）/ M1（L2）/ M2（L3）
> 依赖：SPEC-001
> 实现语言：不限定；接口契约为本 SPEC 主体

---

# 1. 目的与范围

定义 Artifact 与索引的持久化插件。接口与实现解耦（SPEC-001 K2 只定义接口，本插件提供实现）。

---

# 2. 接口（契约）

```ts
interface StorageEngine extends ArtifactStore {
  // SPEC-001 §7.1：save/load/loadByType/list/queryRelations/bumpVersion/remove/exists
  // + 索引层（L2 起）
  searchIndex(query: IndexQuery): Promise<IndexHit[]>
  aggregate(query: AggregateQuery): Promise<Aggregation>
}
```

---

# 3. 实现层次

## L1 — 文件系统（M0）

```text
.specos/
├── project.yaml
├── specs/SPEC-001/{spec.yaml, spec.md}
├── acs/AC-001/{ac.yaml, ac.md}
├── issues/ISSUE-101/{issue.yaml, issue.md}
├── runs/ / reviews/ / tests/
└── .trash/            # 软删除
```

原则：yaml 为源、md 为渲染、原子写（临时文件 + rename）、git 可追踪。

## L2 — SQLite 索引层（M1）

- symbol / relations / project graph / eval 聚合查询。
- 文件仍为可移植格式；SQLite 只做查询加速。

## L3 — 图 / 嵌入（M2，视需要）

- 图结构存储与嵌入检索。

---

# 4. 语义

- `remove` 为软删除（进 `.trash/`，ID 不复用）。
- 写操作原子；中断不产生半个文件。
- 写成功后触发 Event（SPEC-006）。

---

# 5. 与依赖模块的关系

| 模块 | 关系 |
|---|---|
| SPEC-001 | 实现其 K2 接口 |
| SPEC-006 | 写后发事件 |
| SPEC-102 | 索引层供符号/图查询 |
| SPEC-113 | 聚合查询供 Eval |

---

# 6. 验收标准

- [ ] L1：文件布局符合 SPEC-001 §8，原子写、软删除。
- [ ] L2：SQLite 索引支持 symbol/relations/eval 查询。
- [ ] 接口不变，可替换实现。
- [ ] 写操作触发事件。

---

# 7. 边界与不做

- 不做数据模型定义（SPEC-001）。
- 不做查询业务（SPEC-102/113 消费方）。
