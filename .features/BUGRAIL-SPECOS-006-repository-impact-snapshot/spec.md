---
id: BUGRAIL-SPECOS-006
version: "0.1"
title: "Repository Impact Snapshot"
status: draft
changeType: work-task-context-deepening
prd: ".prd/prd-specos-delivery-control.md"
design: "design/specos-control-plane-design.md"
clientDesign: "design/specos-client-interaction-design.md"
codeBaseline: "55545d43"
dependsOn: [BUGRAIL-SPECOS-005]
---

# BUGRAIL-SPECOS-006: Repository Impact Snapshot

## 1. Summary

Expand an explicit file/module scope into a bounded snapshot of related source,
manifests, tests, ownership instructions, and recent Git change evidence. The
first implementation uses local repository facts already available to BugRail;
it does not promise a universal semantic symbol graph or require a remote index.

### Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-006.R01` | A run can request impact analysis from explicit seed paths that exist inside the project. |
| `BUGRAIL-SPECOS-006.R02` | The snapshot explains every relationship and records repository HEAD, policy version, limits, and completeness. |
| `BUGRAIL-SPECOS-006.R03` | Supported relations include containing module/package, declared imports, reverse textual imports, adjacent tests, manifests, ownership instructions, and recent Git co-change. |
| `BUGRAIL-SPECOS-006.R04` | Analysis is bounded, cancellable, cacheable by repository revision, and degrades explicitly when a parser/tool is unavailable. |
| `BUGRAIL-SPECOS-006.R05` | Selected impact items feed Context Pack as optional candidates; impact analysis never bypasses Context security/budget rules. |

PRD coverage: `P-DC-10`, `P-DC-09`, `P-DC-16`, `P-DC-18`.

## 2. Module And Internal Adapters

`RepositoryImpact` is an internal module with one interface:

```text
analyze(root, head, seeds, policy, cancellation) -> ImpactSnapshot
```

It owns traversal, deduplication, scoring, limits, and explanations. Two real
language adapters are included at its private seam:

- TypeScript/JavaScript import extraction for `.ts/.tsx/.js/.jsx`;
- Rust module/use extraction for `.rs` plus Cargo workspace/package manifests.

All other text files receive only language-neutral path, manifest, test-name,
and Git relations. Unsupported language is a partial snapshot, not a guessed
semantic result. Adapters use repository files and existing process helpers;
no daemon or public Plugin Registry is added.

## 3. Snapshot Contract

```text
ImpactSnapshot
  root_revision: git SHA
  policy_version: string
  seeds: relative paths[]
  nodes: [{path, kind, content_hash}]
  edges: [{from, to, relation, reason, confidence}]
  selected: [{path, score, reasons[]}]
  truncated: boolean
  omissions: [{scope, reason}]
  duration_ms: integer
  snapshot_hash: sha256
```

Relations are stable strings: `same_package`, `imports`, `imported_by_text`,
`test_for`, `manifest_for`, `instructions_for`, and `recent_cochange`.
Confidence is `exact` or `heuristic`; consumers must not collapse the two.

Default limits: 20 seeds, 2,000 files inspected, 200 selected nodes, depth 3,
5 seconds interactive / 20 seconds run preparation, and 2 MiB total text read.

## 4. Integration And Cache

- A repository-revision cache lives under BugRail's cache directory, not the
  project tree or SQLite delivery facts. Cache loss changes performance only.
- The immutable snapshot metadata/hash is stored with the Context Pack; the
  full recomputable graph is not copied into a new Artifact database.
- `work_task_impact_get(task_id, run_seq)` returns the stored run snapshot.
- Context Pack records which selected nodes were included or budget-excluded.
- Repository HEAD change causes a cache miss and new snapshot on the next run;
  an in-flight run keeps its recorded revision.

## 5. Errors And Security

| Error key | Condition |
|---|---|
| `workTask.impact.invalidSeed` | Missing, outside-root, secret, or over-limit seed. |
| `workTask.impact.repositoryChanged` | HEAD changes during required snapshot construction. |
| `workTask.impact.unavailable` | Git/filesystem cannot provide a safe minimum result. |

The analyzer applies Context Pack path/secret/binary rules before reading. Git
commands are argument-safe and non-interactive. Generated/vendor directories
are excluded by default and can only be opted in through reviewed project
policy.

## 6. Client Interaction Contract

This Feature implements Run Inspector `Impact`; it visualizes the stored run
snapshot and never performs analysis during board rendering.

- Summary shows repository revision, snapshot/policy hash, seeds, duration,
  limits, completeness, truncation, and omissions.
- Default desktop presentation is a relationship table grouped by seed. Each
  row shows related path, relation, exact/heuristic confidence, explanation,
  score, and whether Context Pack included or budget-excluded it.
- A bounded graph toggle is available only up to 100 displayed nodes. Above
  that threshold the table remains authoritative and explains why graph mode
  is unavailable.
- Exact and heuristic edges have explicit text badges and filter controls; they
  are not distinguished only by line style or color.
- Selecting a path uses the existing repository file-opening behavior. It does
  not fetch stored source content from the impact API.
- Partial/truncated views list every omission and configured limit. Stale means
  the snapshot revision differs from current HEAD, not that the recorded run is
  corrupt.

`src/lib/api.ts` exposes `workTaskImpactGet`; DTOs live in
`src/lib/types.ts`. `impact-tab`, `impact-summary`, `impact-relation-table`, and
the optional `impact-graph` live under `src/components/tasks/specos/`.

Required states are loading, complete, partial, truncated, unavailable,
stale-revision, empty relation set, and transport failure. The table provides
the accessible and narrow-screen representation for every graph fact.

## 7. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-006.AC01` | Rust and TypeScript fixtures produce exact import/module relations and language-neutral fixtures produce labeled heuristic relations. |
| `BUGRAIL-SPECOS-006.AC02` | Same revision/seeds/policy yields the same ordered snapshot and cache hit result. |
| `BUGRAIL-SPECOS-006.AC03` | Limits, cancellation, unsupported languages, unavailable tools, and repository changes yield explicit truncation/omission/error facts. |
| `BUGRAIL-SPECOS-006.AC04` | Outside-root, symlink, secret, binary, generated, and vendor content is not leaked. |
| `BUGRAIL-SPECOS-006.AC05` | Context Pack includes or excludes impact candidates with both impact and budget reasons preserved. |
| `BUGRAIL-SPECOS-006.AC06` | Impact Inspector covers loading, complete, partial, truncated, unavailable, stale-revision, and transport-error states. |
| `BUGRAIL-SPECOS-006.AC07` | No task status, gate, or merge outcome depends on a live cache entry. |

## 8. Testing And Implementation Order

1. Language-neutral graph/limit/security core and fixture repositories.
2. TypeScript and Rust internal adapters with exact/heuristic assertions.
3. Revision cache and cancellation/performance tests.
4. Context Pack integration, transport, Inspector table/graph parity,
   exact/heuristic labeling, and responsive/accessibility tests.
5. Large-repository performance target: p95 under the configured interactive
   limit on the checked-in benchmark fixture; truncation is valid, hanging is not.
