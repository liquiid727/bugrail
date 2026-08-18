# BugRail SpecOS Documentation

## Authority Order

1. `product-vision.md`: long-term product intent; does not freeze interfaces.
2. `../../design/specos-control-plane-design.md`: stable system design.
3. `../../design/specos-client-interaction-design.md`: shared client UX and
   frontend delivery standard.
4. `../../design/adr/`: accepted or proposed architecture decisions.
5. `../../.prd/`: scoped product requirement drafts.
6. `../../.features/roadmap.md`: delivery order and dependencies.
7. `../../.features/<FEATURE-ID>/spec.md`: versioned implementation contract.
8. Matching `test-spec.md`, `.issues/`, `tests/results/`, and review evidence.

Naming and path rules are defined by
`../../rules/specos-artifact-conventions.md`; artifact directories are declared
in `../../.specos/manifest.yaml`.

## Current Slice

`BUGRAIL-SPECOS-001` adds Spec references and evidence-backed quality gates to
the existing CodeG-derived WorkTask flow. It extends the current Rust WorkTask
engine, SeaORM persistence, Tauri/Axum commands, and Tasks UI. It does not add a
parallel workflow engine, event bus, artifact database, or agent runtime.

## Formal Product And Feature Specs

- Product PRD: `../../.prd/prd-specos-delivery-control.md`
- Delivery order: `../../.features/roadmap.md`
- Client interaction standard: `../../design/specos-client-interaction-design.md`
- Implementable Feature Specs: `../../.features/BUGRAIL-SPECOS-001-*` through
  `../../.features/BUGRAIL-SPECOS-010-*`

The ten Specs are vertical CodeG-extension slices and their draft delivery
Issues are indexed at `../../.issues/README.md`. Only `BUGRAIL-SPECOS-001`
currently has its matching Test Spec, so later groups remain dependency-planned
backlog rather than implementation authorization.

## Design Exploration

`rfcs/control-plane-v0/` contains the earlier broad control-plane decomposition.
Its concepts may inform later Feature Specs, but its module contracts and
backlog are not authoritative.
