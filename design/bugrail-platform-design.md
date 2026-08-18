# Code: Bugrail Platform Design

## Meta

- Platform: `Code: Bugrail`
- Status: `accepted`
- Owner: `architecture-agent`
- Project mode: `GoalSpec`
- Decision source: user-approved product direction, 2026-08-05
- Bootstrap spec: `.features/BUGRAIL-001-fork-bootstrap/spec.md`
- Upstream baseline: `xintaofei/codeg` release tag `v0.23.2`
- Fork: `liquiid727/bugrail`
- Repository: standalone checkout of `liquiid727/bugrail` (sibling of SpecOS, not a submodule)

## Purpose

Define the durable product, repository, legal, architecture, coexistence, and
upstream-integration contracts for Code: Bugrail. Feature-level behavior belongs
in versioned Bugrail Feature Specs; this document governs how those features can
evolve without losing upstream traceability or user data.

## Scope

- Code: Bugrail is a distinct product derived from the Codeg codebase.
- The product source lives in this repository (`liquiid727/bugrail`). SpecOS
  (`specos-ai`) is a sibling project and does not vendor or pin this tree.
- The fork begins at upstream release `v0.23.2`, commit
  `159f68e42e6b9d81d9135d47a3879033446b824d`.
- Apache-2.0 license and attribution obligations remain part of every source and
  binary distribution.
- Upstream changes enter through reviewed release-tag syncs, never by implicitly
  following upstream `main`.

## Non-Goals

- Claiming that the bootstrap is already a fully redesigned or release-ready product.
- Rewriting the full Codeg UI, Rust runtime, wire identifiers, or SQLite data in
  `BUGRAIL-001`.
- Replacing upstream history with a source copy or vendored snapshot.
- Treating a clean checkout or a passing local test as release evidence.

## Product Identity Contract

The canonical display name is `Code: Bugrail`; the short product name is
`Bugrail`. `Codeg` remains the upstream project name and may remain temporarily in
internal compatibility identifiers until their migration phase.

Identity has three classes that must not be changed as one search-and-replace:

| Class | Examples | Rule |
|---|---|---|
| Product identity | visible name, package metadata, app identifier, default data roots, keyring service, release channel | Must resolve from the frontend/Rust product manifests and use Bugrail-owned values. |
| Compatibility identity | binary names, `codeg://`, WebSocket protocols, environment variables, database filenames | Change only with an explicit compatibility and rollback contract. |
| Provenance identity | Codeg name in attribution, license history, upstream links, historical migrations and comments | Preserve where needed to describe origin or compatibility. |

The legacy `Bugrail` theme that once lived in SpecOS predates this fork and does
not define this product's identity or architecture.

## Repository And Fork Contract

```text
xintaofei/codeg release tag
          |
          v
liquiid727/bugrail standalone repository
```

- This repository is the product source. It is not a submodule of SpecOS and
  must not be copied back into `specos-ai`.
- The fork uses `origin` for `liquiid727/bugrail` and `upstream` for
  `xintaofei/codeg`.
- Product commits are made and released here. SpecOS may link to a published
  Bugrail revision, but it does not record a gitlink.
- Local development expects a sibling checkout:

```text
~/code/specos-ai/
~/code/bugrail/
```

## Deep Product Modules And Seams

Bugrail should deepen existing modules instead of adding forwarding wrappers.
The interface at each seam is also its test surface.

| Module | Interface and seam | Adapters or implementations | Ownership rule |
|---|---|---|---|
| Product identity | Resolve target-specific product metadata and legal projection from one identity definition. | Next.js UI metadata, Tauri bundle/updater metadata, Rust runtime/version output, distribution attribution. | Callers do not hard-code competing product names or release endpoints. |
| Workbench Core | Workspace, conversation, task, file, Git, and settings operations exposed as typed commands/results. | Tauri command adapter and Axum HTTP adapter over shared Rust command-core functions. | Business rules stay behind the shared interface; transports only translate. |
| Event Stream | Emit and subscribe to typed lifecycle events with ordering, redaction, and backpressure semantics. | Tauri event, WebSocket broadcast, internal event bus, and test adapters. | UI and in-process consumers do not derive product state from raw vendor frames. |
| Agent Runtime | Start, attach, resume, approve, cancel, and observe an agent run through normalized lifecycle results. | ACP/CLI integrations and deterministic test fixtures. | Vendor protocol and process details remain local to the runtime implementation. |
| Persistence | Read and mutate versioned product facts through repository operations and ordered migrations. | SQLite production database plus in-memory/on-disk test databases. | UI and transport modules do not own SQL, database filenames, or migration order. |
| Host Capability | Request only declared host actions and report supported, denied, or unavailable states. | Tauri desktop and standalone-server adapters with test substitutes. | Product modules do not branch directly on global Tauri/browser state. |

No new seam is justified by a single pass-through adapter. Internal seams remain
private unless production and test or multiple runtime adapters actually vary at
that location.

## Data And Compatibility Contract

- `BUGRAIL-001` performs no schema/database-file migration, but assigns new
  installs Bugrail-owned default desktop/server roots and a distinct keyring
  service so CodeG data is not silently opened.
- Existing Codeg data is never silently adopted, moved, or deleted by a branded
  Bugrail build.
- The later data phase must define detection, backup, forward migration,
  idempotency, interrupted-run recovery, downgrade behavior, and an explicit
  user choice when both Codeg and Bugrail stores exist.
- Binary names, URL schemes, WebSocket subprotocols, environment variables, and
  database filenames remain compatibility contracts, not cosmetic strings.
- Bugrail owns `io.liquiid.bugrail`, its GitHub release endpoint, updater signing
  key, `.bugrail`/platform `bugrail` defaults, and `bugrail` keyring service.
- Until independent package verification passes, Bugrail must not publish a
  branded installer even though the identity collision risks are removed.

## Apache-2.0 And Attribution

- Preserve the upstream root `LICENSE` and include the Apache License 2.0 in
  source and binary distributions.
- Preserve applicable copyright, attribution, and license notices in upstream
  source. If an upstream release adds a root `NOTICE`, preserve and distribute it.
- Mark modified files or distribution notices as changed where Apache-2.0
  section 4 requires it; do not represent Bugrail modifications as upstream work.
- Product About and source-distribution materials must state that Code: Bugrail
  is derived from Codeg and link to the upstream source and the Bugrail fork.
- Keep third-party notices, including nested dependency notices, with the
  artifacts to which they apply.
- Product naming does not grant or imply upstream trademark endorsement.

## Upstream Release-Tag Sync

Each upstream sync is a deliberate change package:

1. Fetch tags from `upstream` and select one published release tag.
2. Record the selected tag and its peeled commit; reject branch-only or moving
   references as a release baseline.
3. Create a fork sync branch from the current Bugrail baseline and integrate the
   selected tag without rewriting published Bugrail history.
4. Review conflicts by deep module: identity, workbench core, event stream, agent
   runtime, persistence, and host capability.
5. Reconcile upstream license and notice changes before product changes.
6. Run frontend, Rust desktop, server, MCP, migration, and compatibility gates
   applicable to the delta. Record raw output and normalized evidence separately.
7. Merge and push the validated fork commit on this repository.

Sync automation must stop on a missing tag, changed tag target, dirty worktree,
license/notice delta, merge conflict, migration delta without a plan, or failed
blocking gate. It must never auto-resolve product-identity, updater, app-identifier,
protocol, or database conflicts.

## Product Surface

- This repository is the sole Code: Bugrail product surface.
- SpecOS remains a separate spec/catalog/workbench platform. Bugrail may embed
  SpecOS delivery control inside WorkTask; it does not re-home the SpecOS
  repository.
- A future package-level dependency on published `@specos/*` artifacts requires
  its own approved Feature/Test Specs.

## Delivery Phases

| Phase | Scope | Status |
|---|---|---|
| `BUGRAIL-001` Fork Bootstrap | Pin fork baseline, establish product/provenance contracts, separate display/bundle/data/keyring/update identity, define release-tag discovery, preserve coexistence, and capture baseline gates. | Active implementation; independent evidence pending. |
| `BUGRAIL-002` Full UI Identity And Experience | Replace visible Codeg identity, establish Bugrail design language and complete empty/loading/success/failure states across user flows. | Deferred; no active Feature Spec. |
| `BUGRAIL-003` Runtime And Distribution Compatibility | Migrate inherited binaries, protocols, environment variables, server metadata, packaging details, and operational compatibility. | Deferred; no active Feature Spec. |
| `BUGRAIL-004` Data Migration And Legacy Interop | Define explicit import/coexistence/rollback behavior for existing CodeG data. | Deferred; no active Feature Spec. |

The phases are ordered. A later phase may prepare fixtures earlier, but it cannot
promote behavior or release claims before its own Feature/Test Specs are approved.

## Baseline And Release Gates

`BUGRAIL-001` establishes, but does not itself satisfy, these gates:

- provenance: a fresh clone of `liquiid727/bugrail` resolves the recorded
  upstream tag, fork `origin`, and upstream remote;
- legal: root license and applicable notices are present in source and packaged
  attribution output;
- frontend: frozen install, lint, unit tests, and static production build;
- Rust desktop: check, tests with test utilities, and clippy with warnings denied;
- server and MCP: no-default-feature checks/tests/clippy for supported binaries;
- compatibility: current database migrations, transports, events, updater, URI,
  environment, and package identifiers are inventoried before any rename;
- evidence: independent blocking results are normalized under `tests/results/`
  before review or ship gates can report `ready`.

## Feature Spec Mapping

- `.features/BUGRAIL-001-fork-bootstrap/`: fork and governance bootstrap.
- `.features/roadmap.md`: phase order and deferred status.
- `current/bugrail-bootstrap.md`: active handoff and evidence posture.

## Open Questions

- Whether compatibility identifiers receive aliases, one-time migration, or a
  clean break in `BUGRAIL-003`.
- Whether `BUGRAIL-004` imports Codeg data, runs side by side, or supports both.
- Which operating systems form the first packaged release matrix.
