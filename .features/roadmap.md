# BugRail SpecOS Roadmap

## Requirement Source

- `.prd/prd-specos-delivery-control.md` — product outcomes and ten vertical
  Feature slices derived from the archived Control Plane V0 PRD.
- `design/specos-control-plane-design.md` — CodeG extension and module-placement
  decisions.
- `design/specos-client-interaction-design.md` — shared client information
  architecture, interaction, visual, accessibility, and verification contract.

## Planned Sequence

1. `BUGRAIL-SPECOS-001` — Spec-linked WorkTask, trusted preflight/human gates,
   and merge/complete enforcement.
2. `BUGRAIL-SPECOS-002` — durable WorkTask run evidence and trace inspection.
3. `BUGRAIL-SPECOS-003` — WorkTask dependencies, readiness, and DAG view.
4. `BUGRAIL-SPECOS-004` — structured handoff and integration WorkTask.
5. `BUGRAIL-SPECOS-005` — deterministic, inspectable Context Pack.
6. `BUGRAIL-SPECOS-006` — bounded local repository-impact snapshot.
7. `BUGRAIL-SPECOS-007` — explainable Agent/model routing and safe fallback.
8. `BUGRAIL-SPECOS-008` — evidence-qualified run evaluation projection.
9. `BUGRAIL-SPECOS-009` — reviewable project-memory candidates.
10. `BUGRAIL-SPECOS-010` — validated, approved, reversible Skill candidates.

All ten entries have a draft Feature Spec, a feature-specific client interaction
contract, and dependency-bound draft Issues indexed under `.issues/README.md`.
Only `BUGRAIL-SPECOS-001` has a matching draft Test Spec; it is the sole
candidate for the next implementation gate after review and approval.

## Dependency Notes

- `002` and `003` depend on stable contract/gate semantics from `001`.
- `004` depends on `001-003` for source eligibility, run binding, and readiness.
- `005` depends on Spec, run, and handoff provenance from `001/002/004`.
- `006` enriches `005`; it does not replace deterministic context sources.
- `007` depends on durable route/run/context facts from `002/005`.
- `008` aggregates `002` run facts and `007` route decisions.
- `009` requires `005` injection rules and `008` qualified evidence.
- `010` requires `008/009` evidence and reuses existing ACP Skill management.
