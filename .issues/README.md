# SpecOS Agent Team And Context Issue Index

## Status

This index implements the approved 2026-08-12 PRD/SPEC migration. Issue IDs are
never reused:

- `001-005` retain the original `BUGRAIL-SPECOS-001` delivery history. Issues
  `001-004` are implemented pending verification; `005` owns independent
  verification.
- `006-021` are retained as `superseded` historical plans. Each frontmatter
  block names its replacement Feature and Issue range.
- `022-042` and `061-070` are retained as `superseded` historical plans. They
  were removed from the working tree on 2026-08-19 but remain occupied IDs and
  were restored from Git history so they can never be reused.
- `043-060` and `071-076` are the implementation/verification graph derived
  from `.prd/prd-specos-agent-team-context-system.md` and implemented baseline
  Features `002-009`, `015-016`.
- `077-081` are the approved Memory Plugin MVP01 delivery graph derived from
  `.prd/prd-memory-plugin-mvp01.md` and Feature `017`.
- `082-085` are the verified Feature `028` graph. `086-117` are the draft
  Memory Operating Layer graph for Features `029-036`.

## Current Issue Groups

| Feature | Issues | Delivery closure |
|---|---:|---|
| `001` WorkTask Contract/Gates | `001-005` | exact Spec/AC binding and enforced gates |
| `002` Agent/Model Profiles | `043-045` | catalogs, resolution, UI/verification |
| `003` Run Evidence | `046-047` | durable generations and inspector |
| `004` Dependencies | `048-049` | readiness enforcement and trace |
| `005` Integration/Handoff | `050-051` | structured handoff and integration evidence |
| `006` Context Package | `052-053` | deterministic compiler, run binding and inspector |
| `007` Provider Bootstrap | `054-055` | health/adapter boundary and verification |
| `008` Context Loadouts | `056-058` | definitions, launch integration and security/UI |
| `009` Context Activity | `059-060` | Overview projection and first-level route |
| `015` Static Team Workflow | `071-073` | catalogs, materialization/scheduler and DAG UI |
| `016` Team Operations | `074-076` | controls, node trace/handoff and verification |
| `017` Memory Plugin MVP01 | `077-081` | TencentDB Adapter, capture, recall, UI and verification |
| `028` Context Plugin Foundation | `082-085` | independent contracts, shared assets, provider jobs and verification |
| `029` TencentDB Runtime | `086-089` | pinned install, supervisor, migration/operations and verification |
| `030` Memory Governance | `090-093` | governance overlay, effective recall, Hub and verification |
| `031` Task Context | `094-097` | artifact offload, canvas/resume, operations and verification |
| `032` Wiki Plugin | `098-101` | source registry, sync/Context, UI and verification |
| `033` CodeGraph Plugin | `102-105` | index lifecycle, queries/Context, UI/performance and verification |
| `034` Skill Evolution | `106-109` | candidates, validation/routing, UI and verification |
| `035` Asset Loadouts/ACL | `110-113` | generation resolution, scope/handoff, UI and verification |
| `036` Platform Hardening | `114-117` | operations, failure/performance, packaging/release and verification |

## Execution Rules

- Follow `dependsOn`; numeric order alone is not authority.
- Canonical IDs and references use `issue-NNN`; display aliases such as
  `#0081`, `Issue #81`, or `issue#0081` are not separate identifiers.
- IDs are monotonically allocated and never reused. The current ledger is
  dense through `issue-117`; the next new Issue is `issue-118`.
- Recompute the exact source Feature hash before implementation or verification.
- `planned` means the Issue is not authorized until its source Feature and Test
  Spec are approved.
- `implemented_pending_verification` means code is present but the matching Test
  Spec evidence is not yet accepted; it is not equivalent to verified/done.
- `pending_verification` is owned by a verification Issue whose exact Test Spec
  has not yet been accepted.
- `verified` is the only completed terminal state for active implementation and
  verification Issues. Once the owning exact Test Spec is accepted, covered
  implementation Issues advance to `verified` as well.
- `superseded` is a terminal historical state and never authorizes execution.
- `reopened` means a previously implemented Issue was found to need a bounded
  correction; the reopen record in the Issue body names the scope and the
  closure evidence required.
- Verification Issues execute the exact-version Test Spec and retain commands,
  exit codes, fixture/commit IDs, reports, and durable database/Git oracles.
- Frontend state never substitutes for backend readiness, gates, Team control,
  runtime resolution or Context/Memory provenance.
- Tauri and Axum use the same command-core behavior and must be verified as one
  compatibility contract.

## Local Automation Note

The canonical local Issues are the per-file artifacts in this `.issues/`
directory, as declared by `.specos/manifest.yaml`. Run
`pnpm specos:validate` before delivery; it validates numbering, frontmatter,
source bindings, references, terminal dependencies and cycles. Use this source
directly; do not copy the graph into a second layout or maintain separate
implementation-note artifacts.
