---
id: BUGRAIL-SPECOS-033-TEST
version: "0.1"
status: draft
sourceSpecId: BUGRAIL-SPECOS-033
sourceSpecVersion: "0.1"
sourceSpecHash: "a47f798b27680d8bf313f602abf8d4175396780832ac6d50bddf416ea846801f"
independentFromImplementation: true
---

# Test Spec: Independent CodeGraph Plugin

## 1. Strategy

Deepen the existing `code_intelligence` contract with pinned
`codebase-memory-mcp` fixtures and multi-language repositories. Query output,
index revisions and Worktree lifecycle facts are public oracles.

## 2. Test Cases

| ID | Requirements | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-033.T01` | R01,R04,R05 | Build a multi-language index and query symbol, references/calls and impact. | Bounded results retain exact file, symbol and revision evidence with explicit incomplete state. |
| `BUGRAIL-SPECOS-033.T02` | R02,R03 | Change/rename/delete files; coalesce refreshes; crash before index publication. | Only affected results publish atomically and stale state remains until the recovered job succeeds. |
| `BUGRAIL-SPECOS-033.T03` | R02 | Create base/worktree indexes, switch enablement and clean Worktrees. | Indexes are scope-isolated and cleanup follows eligible Worktree facts without deleting the base index. |
| `BUGRAIL-SPECOS-033.T04` | R01,R04 | Attempt raw MCP, write, unbounded and path-escaping calls through Agent/Context surfaces. | Only the closed read-only query set executes and output is confined, normalized and capped. |
| `BUGRAIL-SPECOS-033.T05` | R03,R05 | Run representative large-repository index/query workloads repeatedly. | Declared budgets hold with bounded process reuse, jobs/database growth and no N+1 process spawn. |
| `BUGRAIL-SPECOS-033.T06` | R06 | Inspect/search/rebuild/impact through Tauri/Axum under adapter restart. | Equivalent last-good/stale/error UI reconstructs from durable index/job state. |

## 3. Required Evidence

- Existing module plus pinned Adapter contract tests.
- Job/revision/Worktree cleanup integration tests.
- Agent tool and Context normalization/security tests.
- Performance budgets and desktop/server UI parity evidence.

## 4. Exclusions

A second graph store, arbitrary MCP/Cypher access, Memory provenance or
small-fixture timing alone cannot satisfy this Test Spec.
