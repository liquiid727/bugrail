# ADR-003: CodeG Owns Context Orchestration; Providers Supply Assets

- Status: accepted
- Date: 2026-08-12
- Refined by: `ADR-004-memory-plugin-tencentdb-mvp01.md`
- Sources: `docs/codeg-memory-context-system-spec.md`,
  `.prd/prd-specos-agent-team-context-system.md`

## Context

Memory, Wiki, code intelligence, Skills, rules, and project files may come from
different local or remote systems. Binding Agent code directly to TencentDB or
letting a Provider compose prompts would make runtime behavior vendor-specific,
hard to budget, and impossible to reproduce after restart.

## Decision

1. CodeG owns the Context domain model, loadout resolution, provider routing,
   budgets, normalization, deduplication, provenance, immutable Context Package,
   prompt injection, and observability.
2. Providers expose capabilities and health through adapters. TencentDB Agent
   Memory is the first bootstrap/reference Provider, not the architecture.
3. Every prompt-bearing WorkTask generation binds one immutable package. Local
   sources are project-canonical, hash-addressed, bounded, and persisted.
4. Required Provider/source failure blocks before prompt dispatch. Optional
   failure is recorded as degradation and remains visible in Context Activity.
5. Credentials are references resolved at runtime; packages and activity never
   persist secrets or uncapped remote output.

## Consequences

- ACP/CLI adapters receive compiled context but do not choose Memory, Wiki,
  Skill, or CodeGraph policy.
- The implemented baseline validates local orchestration and Provider health.
  Feature `BUGRAIL-SPECOS-017` adds remote Memory capture/recall through the
  separate Memory Plugin interface while preserving this Context authority.
- Dynamic ranking/compression, ContextFS, semantic CodeGraph, Memory writes,
  Wiki synchronization, and Skill evolution remain independently replaceable
  later slices.
