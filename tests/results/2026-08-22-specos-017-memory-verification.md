# BUGRAIL-SPECOS-017 Memory Plugin MVP01 — 独立验证记录

- 日期: `2026-08-22`
- 范围: Issue `077`–`081`（T01–T08）、`issue-054` reopen 关闭、聚合确认
  `issue-047` / `issue-053` / `issue-055` / `issue-060`
- Spec hash（重算匹配）:
  - `.features/BUGRAIL-SPECOS-017-memory-plugin-mvp01/spec.md` →
    `f62824d31787ff774d962d942ffba0423fe78f28c77fa7b0e2e000d71dacfd58`
  - `.features/BUGRAIL-SPECOS-017-memory-plugin-mvp01/test-spec.md` →
    `480c2e58669ae43c29c8d119b90ac9d6e8c7abeb2ad298a0a043883311480cf6`
- 上游钉住: `TencentCloud/TencentDB-Agent-Memory@v2.0.0`
  commit `0aff21a2d9f2b8a0354aaa80a2e586aab4054562`（GitHub git refs API
  验证 tag `v2.0.0` → 该 commit）+ `bugrail.1` 补丁
  sha256 `4a2d4b514aa9807cb87e83718bb34e7279ac52a7f1fef92a74639c8b63fd2d16`
  （补丁内容与 hash 见
  `tests/fixtures/tencentdb-memory-v2.0.0+bugrail.1/`）
- 镜像: 本地构建 `bugrail-t08-memory:v2.0.0-bugrail.1`
  image digest
  `sha256:969a78b925495d62498ac4e3b9780b7861b110d640adf2d03e6e7ff52977f784`
  （arm64；离线构建仅去掉 Dockerfile 首行 `# syntax=` 指令，无其他改动）

## 1. 命令与退出码

在 `src-tauri/` 下执行（除非另注明）：

| 命令 | 退出码 | 结果 |
|---|---|---|
| `cargo test --features test-utils` | 0 | 2731 lib + 全部集成套件通过；仅 2 个 env-gated pinned 测试 ignored |
| `cargo test --features test-utils --test memory_fake_gateway` | 0 | 16 通过（T01/T02/T03/T06 transport 契约 + 补丁双向证明） |
| `cargo test --features test-utils --test memory_capture_outbox` | 0 | 12 通过（T03 settlement/outbox/重启恢复 + legacy 零行为） |
| `cargo test --features test-utils --test memory_recall_context` | 0 | 8 通过（T04/T05 package 证据、预算、阻塞、不可变包） |
| `cargo test --features test-utils --test specos_agent_team_context` | 0 | 47 通过（Provider T01–T05、recall→package 集成） |
| `BUGRAIL_T08_URL=http://127.0.0.1:18420 … cargo test --features test-utils --test memory_pinned_gateway -- --ignored` | 0 | 1 通过（T08 全链路，见 §3） |

T08 期间发现的实现缺陷与修复（均已补测试）：

- `parse_hits` 只认 `data.hits`；钉住上游 v3 契约（sdk-v3.yaml
  `AtomicSearchData`）返回 `data.items`。已改为 `items` 优先、`hits`
  兼容别名，fake Gateway fixture 同步改为权威 `items` 形状。
- `load_context` 曾对已保存的无 Memory Provider 配置自动注入默认
  Provider，违反 017 AC08（legacy 项目零 Memory 行为）；已移除注入，
  仅新建项目的默认配置包含 Memory Provider。

## 2. T01–T07 摘要（本地确定性 Adapter / fake Gateway / 命令核与 UI）

- **T01** 配置/身份/allowlist/redaction/legacy：
  `tests/memory_fake_gateway.rs`（t01 credentials/redaction 等）、
  `tests/memory_capture_outbox.rs::legacy_project_without_memory_provider_stages_nothing`、
  `tests/specos_agent_team_context.rs` provider 套件。
- **T02** health 分类/trace/版本门槛/redirect 拒绝：
  `memory_fake_gateway.rs`（401/429/5xx/timeout/malformed/vanilla 检测/
  redirect refusal），`provider_health` 走 `GET /health` + 精确
  `v2.0.0+bugrail.1` 可写门槛。
- **T03** capture 幂等/过滤/上限/重启恢复/payload 清除：
  `memory_capture_outbox.rs`（唯一键 `(provider_id, task_id, run_seq)`、
  attempt 预算、backoff、terminal→manual retry、payload 清除、重启
  reconciliation）；补丁双向证明在 `memory_fake_gateway.rs`
  （replay accepted-ids 稳定、vanilla `memory.upstreamUnsupported`）。
- **T04/T05** recall 归一化、预算边界、included/excluded 证据、
  required 阻塞于 ACP dispatch 前、optional 降级、二次 prepare 返回同一
  不可变 package：`tests/memory_recall_context.rs`（8 用例）；
  `work_task_context_pack.memory_evidence` 持久化
  `{providers:[{provider,adapter,queryHash,included[],excluded[]}]}`，
  不含远端内容/查询文本/凭据（有断言）。
- **T06** 安全：redirect 不跨 scheme、credential URL 拒绝、TLS bypass
  拒绝、远端文本按 untrusted 限长渲染、secrets 不进日志/前端序列化：
  `memory_fake_gateway.rs`（t01/t06 系列）+ `memory_recall_context.rs`
  证据列断言。
- **T07** Provider test / delivery list+retry / recall preview 的
  command-core、Tauri/Axum parity 与 Context UI（十 locale、键盘/窄屏、
  last-good）：见 `src-tauri/src/commands/memory.rs`、
  `src-tauri/src/web/`（memory 路由）、`src/components/context-system/`
  与其测试；前端检查命令与结果见 §4。

## 3. T08 — 固定 TencentDB `v2.0.0+bugrail.1` 集成证据

流程（`tests/memory_pinned_gateway.rs`，单用例全链路）：

1. **health**: `GET /health` →
   `{"status":"ok","version":"v2.0.0+bugrail.1",…}`；
   Provider 判定 healthy 且 writable（精确版本门槛）。
2. **capture + 幂等**: `POST /v3/conversation/add` 稳定消息 id
   （`t08-team-alpha-m1/m2`），accepted ids 与请求一致；同一批次重放
   accepted ids 不变，`POST /v3/conversation/count`
   `data.total == 2` 不增长（补丁 upsert 契约）。
3. **身份隔离**: `team-beta` 独立 capture/count==2；两侧 L1 内容带
   `T8TOK-alpha` / `T8TOK-beta` token，recall 互不可见。
4. **重启 + recall**: 以全新 `MemoryService`（模拟 BugRail 重启）每
   2s 轮询 L1、至多 120s；team-alpha 命中
   `content: "memory-of:alpha captured fact for later recall"`（score、
   remote id、created_at provenance 完整），team-beta 同理，且断言无
   跨 team token。

脱敏请求/响应样本（bearer/凭据为 fixture 本地值，非真实凭据）：

```json
// GET /health
{"status":"ok","version":"v2.0.0+bugrail.1","uptime":18,
 "stores":{"vectorStore":true,"embeddingService":false}, …}
// POST /v3/conversation/count  {"team_id":"team-alpha", …}
{"code":0,"message":"ok","data":{"total":2}}
// POST /v3/atomic/search       {"team_id":"team-alpha", …}
{"code":0,"message":"ok","data":{"items":[{
  "id":"m_1787334865942_69bf09b7","type":"episodic",
  "content":"memory-of:alpha captured fact for later recall",
  "team_id":"team-alpha","task_id":"t08-task-team-alpha",
  "created_at":"2026-08-21T17:54:25.943Z","score":2.0e-06}]}}
```

网关日志（脱敏）确认异步 L1 抽取落库：
`[l1-extractor] Extraction complete: extracted=1, stored=1`、
`[checkpoint] markL1ExtractionComplete session=t08-session-team-alpha`。

LLM 说明：fixture 使用本地 mock OpenAI 兼容端点（返回确定性抽取
JSON），网关本体除 `bugrail.1` 补丁外零改动；无真实云凭据、全程
loopback。

## 4. 前端与双模式检查

（由 issue-080 实现同一提交验证）

| 命令 | 退出码 |
|---|---|
| `pnpm eslint .` | 0 |
| `pnpm test` | 0 |
| `pnpm build` | 0 |
| `cargo check --no-default-features --bin codeg-server` | 0 |
| `cargo clippy --all-targets --features test-utils -- -D warnings` | 0 |

## 5. Issue 处置

- `issue-081` → **verified**（T01–T08 全部满足，含钉住上游证据）。
- `issue-077` / `issue-078` / `issue-079` / `issue-080` 保持
  `implemented_pending_verification`，与实现类 Issue 的 canonical 状态
  语义一致。
- `issue-054` 的 reopen 条件已满足，回到
  `implemented_pending_verification`；Feature 007 的最终独立验收仍由
  `issue-055` 负责。
- 聚合补充证据：T08 §3.1 补充 `issue-055` 的真实远端健康证据，§1–§4
  补充 `issue-047` / `issue-053` 的 transport/package 证据，并为
  `issue-060` 提供 locale/UI 测试证据。这些 Issue 保持
  `pending_verification`，直到各自精确 Test Spec 独立执行并接受。

## 6. 发布结论

Feature 017 的接受记录见 `docs/issue#0081.html`。
