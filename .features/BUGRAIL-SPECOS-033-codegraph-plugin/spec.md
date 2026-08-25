---
id: BUGRAIL-SPECOS-033
version: "0.1"
title: "Independent CodeGraph Plugin"
status: draft
changeType: code-intelligence-plugin
prd: ".prd/prd-memory-operating-layer-roadmap.md"
design: ".features/bugrail-specoos-memory/04-KNOWLEDGE-SKILL-WIKI-CODEGRAPH.md"
codeBaseline: "2ab6d5cf"
dependsOn: [BUGRAIL-SPECOS-006, BUGRAIL-SPECOS-009, BUGRAIL-SPECOS-028]
---

# BUGRAIL-SPECOS-033: Independent CodeGraph Plugin

## 1. Outcome

Complete the existing `code_intelligence` module as the independent CodeGraph
plugin, using the pinned `codebase-memory-mcp` Adapter and existing Worktree
lifecycle rather than TencentDB Memory or a second graph store in BugRail.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-033.R01` | `code_intelligence` is the sole interface for repository index, symbol search, references/calls, impact and changed-file Context; callers cannot invoke raw MCP tools. |
| `BUGRAIL-SPECOS-033.R02` | Base-repository and Worktree indexes have explicit scope, revision, enablement and cleanup state with path confinement. |
| `BUGRAIL-SPECOS-033.R03` | Full build and changed-file refresh use durable provider jobs, coalesce duplicate requests and recover after adapter/process restart. |
| `BUGRAIL-SPECOS-033.R04` | Query results are bounded and normalized with file/symbol/revision provenance before entering Context or Agent tools. |
| `BUGRAIL-SPECOS-033.R05` | Planning can request impact evidence before changes; stale/incomplete index state is explicit and never presented as complete truth. |
| `BUGRAIL-SPECOS-033.R06` | CodeGraph UI exposes index state, search, relationships, impact, rebuild and error recovery in both transports. |

## 3. Existing Modules

- Deepen `src-tauri/src/code_intelligence/` and its managed Adapter.
- Reuse existing MCP session confinement, binary pin, WorkTask Worktrees and
  Context engine-item path.
- Do not add a parallel graph database or route through MemoryProvider.

## 4. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-033.AC01` | A multi-language fixture supports symbol, reference/call and impact queries with exact file/revision evidence. |
| `BUGRAIL-SPECOS-033.AC02` | Changed-file refresh updates affected results without a full rebuild and reports stale state until publication. |
| `BUGRAIL-SPECOS-033.AC03` | Worktree indexes are isolated, inherit enablement deliberately and are removed only with eligible Worktree cleanup. |
| `BUGRAIL-SPECOS-033.AC04` | Agent and Context paths expose only the closed read-only query set and bounded normalized output. |
| `BUGRAIL-SPECOS-033.AC05` | Large-repository fixtures satisfy documented query/index budgets without N+1 process spawning. |

## 5. Non-Goals

No Wiki generation, long-term conversational Memory, Skill discovery or
arbitrary Cypher/write access is introduced.
