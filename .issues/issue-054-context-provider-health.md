---
id: issue-054
title: "Context Provider bootstrap and health boundary"
status: reopened
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-007
sourceSpecVersion: "0.2"
sourceSpecHash: "fc10e29a5c2be849a1573875aee7b3fb73f0292dcbfb5e23623c88318f6d669f"
requirements: [BUGRAIL-SPECOS-007.R01]
dependsOn: [issue-052]
---

# Context Provider bootstrap and health boundary

## Outcome

Add project Provider definitions and normalize local/Tencent-compatible health without coupling Agents to provider APIs.

## Scope

Use credential references, bounded timeout, required fail-closed and optional degraded activity; remote retrieval remains out of scope.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.


## Reopen Record (2026-08-18, BUGRAIL-SPECOS-017)

The implemented health probe used `GET {endpoint}/v3/tools/list` for
non-local Providers. That endpoint belongs to the upstream Knowledge service
and is a POST there; it is not a MemoryCore health endpoint and never
produced real remote health evidence (`issue-055` T02). TencentDB Agent
Memory v2.0.0 MemoryCore exposes a public `GET /health` endpoint instead.

Reopen scope:

- Probe `GET {endpoint}/health` for non-local Providers; providers with
  `kind: memory` are delegated to the Memory module, which additionally
  verifies the pinned `v2.0.0+bugrail.1` version before capture is writable.
- Keep credential references, bounded timeout, required fail-closed and
  optional degraded activity semantics unchanged.

Closure requires the corrected probe plus the pinned upstream health evidence
recorded under `issue-081`.

## Reopen Fix Record (2026-08-19, issue-077)

- `src/context/mod.rs::check_provider_health` now probes
  `GET {endpoint}/health` for non-local Providers with
  `reqwest::redirect::Policy::none()`; `kind: memory` Providers delegate to
  the Memory module (`MemoryService::provider_health`), which adds the pinned
  `v2.0.0+bugrail.1` version/writability gate on top of the same probe.
- Credential references, bounded 5s timeout, required fail-closed and
  optional degraded semantics are unchanged.
- Transport oracles: `tests/memory_fake_gateway.rs` (health classes, trace
  id, vanilla detection, redirect refusal) and
  `tests/specos_agent_team_context.rs` provider T01-T05.
- Status stays `reopened` until `issue-081` records the pinned upstream
  health evidence.
