# ADR-004: Memory Plugin With TencentDB Agent Memory As The First Adapter

- Status: accepted
- Date: 2026-08-18
- Upstream baseline: TencentDB Agent Memory `v2.0.0`
- Sources: `.prd/prd-memory-plugin-mvp01.md`,
  `docs/specos/research/tencentdb-agent-memory-mvp01.md`
- Refines: `ADR-003-context-orchestrator-provider-boundary.md`

## Context

The implemented Context path already owns provider configuration, health,
loadouts, budgets, immutable packages, provenance, prompt injection and UI.
It does not retrieve or write remote memory. TencentDB Agent Memory provides a
v3 HTTP data plane with strict team/agent/user/session isolation and L0-L3
memory operations. Its Proxy and Panel also perform identity, prompt and UI
work that overlaps BugRail responsibilities.

The earlier draft Memory design proposed a second SQLite candidate lifecycle
and Git-written Markdown memory. That design duplicates capabilities now
provided by the selected Memory Engine and is retired before implementation.

## Decision

1. Add one deep `memory` module behind a small interface:
   `health`, `capture` and `recall`. Callers do not construct vendor requests,
   parse envelopes or apply retry rules.
2. Implement TencentDB Agent Memory v3 as the first production Adapter. Use a
   deterministic in-memory Adapter only at the internal test seam. MVP01 uses a
   static allowlist, not a dynamic binary/plugin registry.
3. Connect directly to the MemoryCore Gateway. Do not route ACP/CLI model
   traffic through MemoryProxy and do not embed MemoryPanel.
4. BugRail owns capture policy, redaction, stable source IDs, delivery evidence,
   identity mapping, recall query construction, budgets, Context Package
   persistence, prompt injection and user-facing state.
5. TencentDB owns durable L0-L3 memory, extraction and retrieval. BugRail stores
   delivery/selection evidence, not a mirror of the remote memory database.
6. Memory, Wiki, CodeGraph and Skill Evolution remain separate modules. They may
   share identity and the `ContextItem` envelope and may use one TencentDB
   deployment, but they do not share one catch-all interface.
7. Pin the Adapter contract to upstream `v2.0.0`. Upgrades require contract
   tests against a named tag or image digest before changing the pin.

## Consequences

- The existing `context` module calls Memory recall and applies the same
  required/optional, provenance and budget rules as local sources.
- WorkTask run settlement queues bounded capture after durable local state is
  available. Memory failure does not rewrite a settled WorkTask outcome.
- A required recall failure blocks before prompt dispatch; an optional failure
  creates an explicit degraded package and activity record.
- A second production Memory Adapter can be added without changing WorkTask,
  Context Package or UI callers. Wiki, CodeGraph and Skill work can evolve
  independently.

## Rejected Alternatives

- **Use MemoryProxy as the Agent endpoint.** This creates two owners for prompt
  injection, session identity and retry behavior.
- **Embed MemoryPanel.** It duplicates the BugRail Context UI and transport
  model and would expose a second source of operational state.
- **Keep Git Markdown as the primary Memory Engine.** It reimplements
  extraction, retrieval and lifecycle while the selected provider already owns
  those concerns.
- **Expose raw TencentDB endpoints throughout the codebase.** This produces a
  shallow pass-through and spreads vendor behavior across WorkTask, Context
  and UI callers.

