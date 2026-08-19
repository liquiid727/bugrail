# TencentDB Agent Memory 接入研究：BugRail MVP01

## 结论

BugRail 的第一版接入应直接调用 TencentDB Agent Memory `v3` Gateway，
而不是把上游 Proxy 放到 ACP/CLI 请求链路中，也不嵌入上游 MemoryPanel。
BugRail 保留采集策略、身份映射、Context Package、提示词注入、运行证据和 UI；
TencentDB Agent Memory 负责 L0-L3 的存储、提取和检索。

研究基线锁定上游 `v2.0.0` 标签
`0aff21a2d9f2b8a0354aaa80a2e586aab4054562`。上游默认分支仍在快速迭代，
MVP 实现不能跟随分支 HEAD。

## 上游事实

1. 上游由 MemoryCore、MemoryKnowledge、MemoryPanel、MemoryProxy 组成，并提供
   Docker 部署。完整安装会启动 memory-core、memory-hub 和 proxy；Gateway
   默认作为独立 HTTP 数据面供 SDK 调用。
   [README_CN](https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/v2.0.0/README_CN.md)
   [INSTALL_CN](https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/v2.0.0/INSTALL_CN.md)
2. 新接入推荐 `v3`。构造客户端必须提供 `teamId`、`agentId`、`userId`；
   L0 写入还必须提供 `sessionId`，`taskId` 可选。
   [TypeScript SDK](https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/v2.0.0/sdk/memory-core/typescript/README_CN.md)
   [v3 client](https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/v2.0.0/sdk/memory-core/typescript/src/v3/client.ts)
3. Gateway 鉴权使用 `Authorization: Bearer <api-key>` 与
   `x-tdai-service-id`；返回业务 envelope，`code != 0` 也应视为失败。
   [v3 HTTP transport](https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/v2.0.0/sdk/memory-core/typescript/src/v3/http.ts)
4. MVP 所需接口已存在：L0 `POST /v3/conversation/add`，L1
   `POST /v3/atomic/search`，L2 `POST /v3/scenario/read`，L3
   `POST /v3/core/read`。消息允许调用方提供稳定 `id`、角色、内容和时间。
   [SDK types](https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/v2.0.0/sdk/memory-core/typescript/src/types.ts)
5. 上游把 Chat Memory、Skill、Wiki、CodeGraph 作为不同资产能力；Wiki 与
   CodeGraph 通过 tools list/call 按需读取。它们可由同一部署提供，但并不要求
   接入方把四类能力压成一个写入/检索接口。
   [README technical implementation](https://github.com/TencentCloud/TencentDB-Agent-Memory/blob/v2.0.0/README_CN.md#技术实现)

## 对 BugRail 的推导

- 上游 Proxy 会修改 system prompt、管理 session identity 并代理模型请求。
  BugRail 已在 WorkTask/ACP/Context Package 中拥有这些职责，因此 MVP01 直接
  调 Gateway，避免出现两套注入、身份和重试语义。
- 上游 MemoryPanel 是管理面参考实现。BugRail 已有 Context 一级页面与
  Tauri/Axum command-core，所以第一版只复用上游 Engine，不嵌入 Panel。
- `teamId/agentId/userId` 必须显式配置或映射。不能直接用本地自增 folder ID
  作为跨安装身份，也不能在缺失身份时退化到共享默认桶。
- BugRail 应持久化写入回执和每次 recall 的来源/hash，但不复制 TencentDB
  的长期记忆库。真正进入 prompt 的远程结果仍固化为 Context Package item。

## MVP01 边界

- 写入：WorkTask run 结束后，把经过过滤、限长的 user/assistant 文本批量写入
  L0；稳定 message ID 保证重试安全。
- 检索：run 启动时，以任务目标检索 L1，并可读取 L3 Core；结果先经过现有
  item/byte/token budget，再进入不可变 Context Package。
- UI：配置和测试 Provider、查看 capture delivery、查看 recall provenance；
  不复制 MemoryPanel 的 Team/ACL/资产编辑功能。
- 暂不实现：Proxy 接管、上游 Team/Agent 自动创建、L2 编辑、Memory CRUD、
  Wiki、CodeGraph、Skill Evolution 和跨项目共享。

