# TencentDB Agent Memory 上游补丁契约：`v2.0.0+bugrail.1`

状态：BUGRAIL-SPECOS-017 Memory MVP01 冻结契约（与
`.features/BUGRAIL-SPECOS-017-memory-plugin-mvp01/spec.md` §4 / §4.1 一致）。

## 1. 基线与钉住版本

- 上游仓库：`TencentCloud/TencentDB-Agent-Memory`
- 基线标签：`v2.0.0`
- 钉住 commit：`0aff21a2d9f2b8a0354aaa80a2e586aab4054562`
- 补丁版本字符串：`v2.0.0+bugrail.1`

上游默认分支仍在快速迭代，MVP01 不跟随分支 HEAD。BugRail 只与
「钉住 commit + 本补丁」组合互通；其他版本按 §4 处理。

## 2. 为什么需要补丁

原生 `v2.0.0` 的类型允许调用方在 `POST /v3/conversation/add` 的消息里携带
`id`，但 Gateway 把该字段视为只读，并在写入时重新生成随机 ID。

BugRail 的 capture 是 at-least-once 投递（outbox + 重试 + 重启恢复），
依赖「同一批消息重放不产生重复 L0 行」。若上游重写消息 ID：

- 每次重试都会写入一组新的 L0 消息；
- 重启恢复会把已投递批次再写一遍；
- `(provider_id, task_id, run_seq)` 的本地唯一键无法阻止上游侧重复。

因此 MVP01 钉住一个最小补丁，而不是放宽重试语义。

## 3. 补丁内容（最小变更）

补丁只修改 `POST /v3/conversation/add` 的消息 ID 语义：

1. **接受调用方 ID**：请求中每条消息的稳定 `id` 被保留，不再重新生成。
2. **按 ID upsert**：同一 `(team_id, agent_id, session_id)` 下，已存在的
   消息 ID 执行幂等 upsert，不新增 L0 行。
3. **重放语义**：重放一个消息 ID 完全已知的请求，返回与首次相同的
   accepted ID 集合，且不产生任何额外 L0 数据；部分缺失时只补缺失消息。

补丁不改动鉴权、envelope（`code != 0` 失败语义）、L1/L3 读取路径或其他
v3 接口。

## 4. 版本报告与检测

- 补丁版本通过 MemoryCore 公共端点 `GET /health` 的 `version` 字段报告，
  同时写入构建清单。
- 健康探测只使用 `GET /health`。`GET /v3/tools/list` 不是健康探针：
  它属于 Knowledge 服务，且在那里是 POST（issue-054）。
- **可写门槛（writable gate）**：`/health` 报告的版本必须与
  `v2.0.0+bugrail.1` **完全一致**，Provider 才进入 writable 状态并允许
  capture。
- 版本不匹配（含原生 `v2.0.0`、旧补丁、未知版本）时：
  - capture 保持禁用，错误类为 `memory.upstreamUnsupported`；
  - recall（只读路径）仍可运行，因为读取不依赖 upsert 契约；
  - Provider 健康状态显示 `degraded` 并携带安全错误码，不透出上游报文。

## 5. 契约测试（双向证明）

实现侧必须用 fake Gateway 证明两个方向：

1. **补丁方向**：相同消息 ID 重放（重试 / 重启恢复场景）不增加 L0 条数，
   accepted ID 集合稳定。
2. **原生方向**：报告 `v2.0.0` 的 Gateway 被识别为
   `memory.upstreamUnsupported`，capture 不可写，且不发出 L0 写请求。

对应测试规格见
`.features/BUGRAIL-SPECOS-017-memory-plugin-mvp01/test-spec.md`
（patch contract 相关 T 项与集成夹具）。

## 6. 运维注意

- 升级上游部署时必须同时带上补丁并验证 `/health` 版本，否则 BugRail 侧
  capture 自动降级为禁用（安全方向，无数据风险）。
- 不允许为「让 capture 可用」而放宽版本精确匹配；该门槛是重试安全的前提。
- 未来上游版本若原生提供幂等消息 ID，可通过新的
  `v2.x.y+bugrail.z` 补丁版本或新的 Adapter 版本契约替换本契约。
