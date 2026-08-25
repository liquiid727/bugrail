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
verification still pending. Features `017` and `028` are verified. Features
`029-036` remain drafts and are not implementation authority.

## Formal Product And Feature Specs

- Product PRD: `../../.prd/prd-specos-agent-team-context-system.md`
- Memory MVP01 PRD: `../../.prd/prd-memory-plugin-mvp01.md`
- Delivery order: `../../.features/roadmap.md`
- Client interaction standard: `../../design/specos-client-interaction-design.md`
- Approved Feature Specs: `001-009`, `015-017`, `028` under
  `../../.features/`
- Draft Feature Specs: `029-036` under `../../.features/`

Delivery Issues are indexed at `../../.issues/README.md`. The former draft
posture for Feature `017` has been superseded by its approved and verified
`0.2` contract; draft Features `029-036` must be reviewed and approved
individually before implementation.

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
