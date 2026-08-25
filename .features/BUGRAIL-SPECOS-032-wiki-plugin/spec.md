---
id: BUGRAIL-SPECOS-032
version: "0.1"
title: "Independent Wiki Plugin"
status: draft
changeType: knowledge-plugin
prd: ".prd/prd-memory-operating-layer-roadmap.md"
design: ".features/bugrail-specoos-memory/04-KNOWLEDGE-SKILL-WIKI-CODEGRAPH.md"
codeBaseline: "2ab6d5cf"
dependsOn: [BUGRAIL-SPECOS-006, BUGRAIL-SPECOS-009, BUGRAIL-SPECOS-028, BUGRAIL-SPECOS-029]
---

# BUGRAIL-SPECOS-032: Independent Wiki Plugin

## 1. Outcome

Add a source-backed, revisioned Wiki plugin that can use a pinned TencentDB
Knowledge Adapter without becoming part of MemoryProvider or duplicating the
source repository as untraceable generated knowledge.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-032.R01` | A `WikiProvider` interface owns source registration, sync/rebuild, bounded search and page retrieval; Memory has no Wiki methods. |
| `BUGRAIL-SPECOS-032.R02` | Source registry records project scope, canonical root, include/exclude policy, revision, status and last successful index. |
| `BUGRAIL-SPECOS-032.R03` | Full and incremental pipelines retain page/section citations, source revision and stale/error state through durable provider jobs. |
| `BUGRAIL-SPECOS-032.R04` | Search results normalize into Context candidates with source citations; current source/accepted ADR outranks generated Wiki on conflict. |
| `BUGRAIL-SPECOS-032.R05` | The first production Adapter is pinned and contract-tested independently from the Memory Adapter even when both share one deployment. |
| `BUGRAIL-SPECOS-032.R06` | Wiki UI supports sources, sync/rebuild, page browsing, search, citations and visible stale/degraded states. |

## 3. Existing Modules

- Reuse Context budget/provenance, provider jobs, workspace file confinement and
  command-core transport patterns.
- Reuse the managed runtime connection profile only; do not call through
  `MemoryProvider`.
- Markdown source remains authoritative over generated pages.

## 4. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-032.AC01` | Importing a mixed Markdown/ADR fixture produces revisioned pages whose citations open the exact source. |
| `BUGRAIL-SPECOS-032.AC02` | A changed or deleted source incrementally updates only affected pages and visibly marks stale results during failure. |
| `BUGRAIL-SPECOS-032.AC03` | Wiki search contributes bounded cited Context items without Memory or vendor DTO leakage. |
| `BUGRAIL-SPECOS-032.AC04` | Project isolation, symlink/path confinement and malicious document tests prevent cross-root reads and instruction injection. |
| `BUGRAIL-SPECOS-032.AC05` | Desktop/server UI and job recovery remain equivalent after restart. |

## 5. Non-Goals

No CodeGraph facts, Memory mutation, Skill publishing or generic document
editor is included.
