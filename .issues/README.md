# SpecOS Agent Team And Context Issue Index

## Status

This index implements the approved 2026-08-12 PRD/SPEC migration. Issue IDs are
never reused:

- `001-005` retain the original `BUGRAIL-SPECOS-001` delivery history. Issues
  `001-004` are implemented pending verification; `005` owns independent
  verification.
- `006-042` are retained as `superseded` historical plans. Each frontmatter
  block names its replacement Feature and Issue range.
- `043-076` are the current implementation/verification graph derived from
  `.prd/prd-specos-agent-team-context-system.md` and Features `002-016`.

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
| `010` Repository Impact | `061-062` | bounded analyzer and inspector |
| `011` Explainable Routing | `063-064` | policy/decision and inspector |
| `012` Run Evaluation | `065-066` | qualified facts and insights |
| `013` Project Memory | `067-068` | candidate lifecycle and review UI |
| `014` Skill Experience | `069-070` | evidence/validation/promotion and UI |
| `015` Static Team Workflow | `071-073` | catalogs, materialization/scheduler and DAG UI |
| `016` Team Operations | `074-076` | controls, node trace/handoff and verification |

## Execution Rules

- Follow `dependsOn`; numeric order alone is not authority.
- Recompute the exact source Feature hash before implementation or verification.
- `implemented_pending_verification` means code is present but the matching Test
  Spec evidence is not yet accepted; it is not equivalent to verified/done.
- Verification Issues execute the exact-version Test Spec and retain commands,
  exit codes, fixture/commit IDs, reports, and durable database/Git oracles.
- Frontend state never substitutes for backend readiness, gates, Team control,
  runtime resolution, Context provenance, Memory approval, or Skill activation.
- Tauri and Axum use the same command-core behavior and must be verified as one
  compatibility contract.
