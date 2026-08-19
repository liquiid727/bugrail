# SpecOS Agent Team And Context Issue Index

## Status

This index implements the approved 2026-08-12 PRD/SPEC migration. Issue IDs are
never reused:

- `001-005` retain the original `BUGRAIL-SPECOS-001` delivery history. Issues
  `001-004` are implemented pending verification; `005` owns independent
  verification.
- `006-021` are retained as `superseded` historical plans. Each frontmatter
  block names its replacement Feature and Issue range.
- `043-060` and `071-076` are the implementation/verification graph derived
  from `.prd/prd-specos-agent-team-context-system.md` and implemented baseline
  Features `002-009`, `015-016`.
- `077-081` are the draft Memory Plugin MVP01 graph derived from
  `.prd/prd-memory-plugin-mvp01.md` and Feature `017`.

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

## Execution Rules

- Follow `dependsOn`; numeric order alone is not authority.
- Recompute the exact source Feature hash before implementation or verification.
- `implemented_pending_verification` means code is present but the matching Test
  Spec evidence is not yet accepted; it is not equivalent to verified/done.
- `reopened` means a previously implemented Issue was found to need a bounded
  correction; the reopen record in the Issue body names the scope and the
  closure evidence required.
- Verification Issues execute the exact-version Test Spec and retain commands,
  exit codes, fixture/commit IDs, reports, and durable database/Git oracles.
- Frontend state never substitutes for backend readiness, gates, Team control,
  runtime resolution or Context/Memory provenance.
- Tauri and Axum use the same command-core behavior and must be verified as one
  compatibility contract.
