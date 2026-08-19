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

## Current Delivery Posture

Features `001-009` and `015-016` have implementation baselines with independent
verification still pending. Feature `017` is the active Memory Plugin MVP01
draft.

## Formal Product And Feature Specs

- Product PRD: `../../.prd/prd-specos-agent-team-context-system.md`
- Memory MVP01 PRD: `../../.prd/prd-memory-plugin-mvp01.md`
- Delivery order: `../../.features/roadmap.md`
- Client interaction standard: `../../design/specos-client-interaction-design.md`
- Active Feature Specs: `001-009`, `015-017` under `../../.features/`

Delivery Issues are indexed at `../../.issues/README.md`. Draft Feature `017`
is not implementation authority until its PRD/Spec/Test Spec are reviewed and
approved.

## Design Exploration

`rfcs/control-plane-v0/` contains the earlier broad control-plane decomposition.
Its concepts may inform later Feature Specs, but its module contracts and
backlog are not authoritative.

## Research And Upstream Contracts

- `research/tencentdb-agent-memory-mvp01.md`: integration research baseline for
  the Memory Plugin (Feature `017`).
- `upstream/tencentdb-agent-memory-patch-contract.md`: frozen
  `v2.0.0+bugrail.1` upstream patch contract (pinned commit, replay/upsert
  semantics, version detection via `/health`).

