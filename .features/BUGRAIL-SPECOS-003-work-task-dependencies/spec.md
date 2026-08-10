---
id: BUGRAIL-SPECOS-003
version: "0.1"
title: "WorkTask Dependencies"
status: draft
changeType: work-task-deepening
prd: ".prd/prd-specos-delivery-control.md"
design: "design/specos-control-plane-design.md"
clientDesign: "design/specos-client-interaction-design.md"
codeBaseline: "55545d43"
dependsOn: [BUGRAIL-SPECOS-001]
---

# BUGRAIL-SPECOS-003: WorkTask Dependencies

## 1. Summary

Add an acyclic dependency graph to existing WorkTasks and make readiness an
explainable projection consumed by current manual start and folder scheduler
paths. Existing WorkTask statuses remain authoritative; no parallel DAG node
state machine is added.

### Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-003.R01` | A WorkTask can declare same-project dependencies with stable kind and order. |
| `BUGRAIL-SPECOS-003.R02` | Self edges, duplicates, cross-project edges, and cycles are rejected atomically. |
| `BUGRAIL-SPECOS-003.R03` | Readiness returns eligible plus explicit unmet/failed dependency reasons. |
| `BUGRAIL-SPECOS-003.R04` | Manual start, start-all, and auto-process cannot claim an ineligible task. |
| `BUGRAIL-SPECOS-003.R05` | Board and graph views expose dependencies without inventing additional task statuses. |

PRD coverage: `P-DC-06`, `P-DC-08`, `P-DC-16`, `P-DC-18`.

## 2. Architecture And Interface

The seam stays inside the WorkTask module:

```text
set_dependencies(task_id, expected_revision, edges) -> DependencyGraph
get_dependency_graph(folder_id) -> DependencyGraph
get_readiness(task_id) -> WorkTaskReadiness
```

| Existing module | Change |
|---|---|
| `db/service/work_task_service.rs` | Transactional edge replacement, cycle query, readiness, eligible queue selection. |
| `work_task/engine.rs` | Recheck readiness before claim/launch and continue using current per-folder concurrency. |
| `commands/work_task.rs` | Add dependency/readiness operations; existing start errors become explainable. |
| `Tasks UI` | Add dependency editor, compact blocked reason, and folder graph view. |

## 3. Data Contract

### `work_task_dependency`

```text
task_id             INTEGER NOT NULL FK work_task(id) ON DELETE CASCADE
depends_on_task_id  INTEGER NOT NULL FK work_task(id) ON DELETE RESTRICT
kind                TEXT NOT NULL  -- completion|integration_source
sort_order          INTEGER NOT NULL
created_at          TIMESTAMP NOT NULL
PRIMARY KEY(task_id, depends_on_task_id)
CHECK(task_id <> depends_on_task_id)
```

`completion` is satisfied only by dependency status `done`.
`integration_source` is stored now but is satisfiable only by the eligibility
contract introduced in `BUGRAIL-SPECOS-004`; before that Feature is installed,
such an edge returns `unsupported_dependency_kind` and cannot become ready.

`WorkTaskReadiness`:

```text
eligible: boolean
revision: string                 -- hash of ordered live edges/status/run_seq
unmet: [{task_id, kind, status, reason}]
terminal_blockers: [{task_id, status, reason}]
```

## 4. Rules And Concurrency

1. Both tasks must be live, undeleted, and resolve to the same root project
   folder. Worktree child folders do not form separate projects.
2. Edge replacement uses a transaction, validates the full proposed graph, and
   records one `dependencies_changed` WorkTask event.
3. A depth-first/recursive-CTE cycle check is bounded to the folder graph;
   maximum 500 live tasks and 2,000 edges per folder.
4. `work_task_start`, `start_all`, and `auto_claim_next` evaluate readiness in
   the same transaction as the current claim CAS. A UI precheck is advisory.
5. Dependency failure/cancellation does not mutate the dependent task. It
   remains in its current state and readiness reports a terminal blocker until
   the user retries/removes the dependency.
6. Once a task is `queued` or later, its edges are immutable until it returns to
   `todo`, `failed`, or `canceled`. This prevents the execution contract from
   changing in flight.
7. Deleting a prerequisite with live dependents is rejected with dependent IDs.

## 5. Errors And UI

| Error key | Condition |
|---|---|
| `workTask.dependency.invalid` | Self, duplicate, cross-project, unsupported kind, or limit violation. |
| `workTask.dependency.cycle` | Proposed edges create a cycle; payload contains the cycle path. |
| `workTask.dependency.unmet` | A claim is attempted while prerequisites are unmet. |
| `workTask.dependency.inUse` | Delete attempted while live dependents exist. |
| `workTask.dependency.conflict` | Editor revision is stale. |

UI covers empty graph, loading, ready, waiting, terminal blocker, cycle error,
stale edit, and transport failure. Graph rendering is folder-scoped and virtualized
after 100 nodes; the board continues to use its existing columns.

## 6. Client Interaction Contract

This Feature implements the Board blocker projection, Task Detail `Plan`
dependency editor, and Tasks `Graph` view.

- A blocked card shows one highest-priority `N blockers` chip; opening it lands
  on Plan with every unmet or terminal dependency and its source task status.
- Plan shows an ordered dependency list. `Edit dependencies` is available only
  in allowed task states and opens a compare-and-save dialog containing task
  search, kind selection, order controls, and the current graph revision.
- Saving sends the complete proposed edge set and `expected_revision`. Cycle,
  in-use, or stale-revision errors keep edits intact and show the cycle path or
  changed dependency facts inline.
- Graph is folder-scoped with ready/waiting/blocked/integration filters. Nodes
  expose title, status, readiness, Agent, and gate summary; labeled edges expose
  `completion` or `integration_source`. Selecting a node opens Task Detail.
- Graph editing is never enabled by ordinary drag. A deliberate Edit mode uses
  the same compare-and-save contract as Plan.
- At more than 100 nodes, the surface switches to a virtualized topology list;
  the 500-node/2,000-edge backend limits remain visible in error copy.

`src/lib/api.ts` exposes `workTaskDependenciesSet`,
`workTaskDependencyGraph`, and `workTaskReadiness`; wire DTOs live in
`src/lib/types.ts`. `dependency-plan`, `dependency-editor-dialog`,
`dependency-graph`, and `readiness-summary` live under
`src/components/tasks/specos/`.

Required states are empty, loading, ready, waiting, terminal blocker, cycle
preview/error, stale edit, over-limit fallback, and transport failure. Graph
nodes and edges have equivalent keyboard/list representations and do not rely
on color alone.

## 7. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-003.AC01` | Valid acyclic edges persist in stable order and survive restart. |
| `BUGRAIL-SPECOS-003.AC02` | Invalid, cross-project, cyclic, over-limit, or stale-revision edits leave the old graph unchanged. |
| `BUGRAIL-SPECOS-003.AC03` | Manual, bulk, automatic, and concurrent claim paths cannot start an unmet task. |
| `BUGRAIL-SPECOS-003.AC04` | A completed prerequisite makes the dependent eligible without a manual state rewrite. |
| `BUGRAIL-SPECOS-003.AC05` | Failed/canceled/deleted prerequisites yield explicit stable blockers and no cascade status mutation. |
| `BUGRAIL-SPECOS-003.AC06` | Existing tasks with no edges preserve scheduling, concurrency, retry, and recovery behavior. |
| `BUGRAIL-SPECOS-003.AC07` | Tauri/Axum and Task UI expose equivalent graph/readiness facts and all required states. |

## 8. Testing And Implementation Order

1. Migration, edge repository, recursive cycle/limit, and delete-protection tests.
2. Claim-transaction and concurrent scheduler tests.
3. Command/transport wire and error tests.
4. Dependency editor/graph/board tests including CAS conflict, keyboard
   navigation, virtualized fallback, responsive layout, and every UI state.
5. Existing WorkTask engine recovery and concurrency regression suites.
