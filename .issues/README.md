# SpecOS Delivery Control Issue Index

## Status

All Issues are local draft delivery units generated from
`.prd/prd-specos-delivery-control.md` and the exact Feature Spec versions/hashes
under `.features/`. Draft status does not authorize implementation or claim
verification evidence.

## Feature Groups

| Feature | Issues | Delivery sequence | Client Issue |
|---|---:|---|---:|
| `BUGRAIL-SPECOS-001` Contract and Gates | `001-005` | schema -> bind/preview -> enforce -> client -> verify | `004` |
| `BUGRAIL-SPECOS-002` Run Evidence | `006-009` | persistence -> trace API -> client -> verify | `008` |
| `BUGRAIL-SPECOS-003` Dependencies | `010-013` | graph store -> readiness -> client -> verify | `012` |
| `BUGRAIL-SPECOS-004` Integration | `014-017` | task/handoff -> integration -> client -> verify | `016` |
| `BUGRAIL-SPECOS-005` Context Pack | `018-021` | compiler -> run integration -> client -> verify | `020` |
| `BUGRAIL-SPECOS-006` Impact | `022-025` | analyzer -> cache/context -> client -> verify | `024` |
| `BUGRAIL-SPECOS-007` Routing | `026-029` | resolver -> fallback -> client -> verify | `028` |
| `BUGRAIL-SPECOS-008` Evaluation | `030-033` | projection -> reports -> client -> verify | `032` |
| `BUGRAIL-SPECOS-009` Memory | `034-037` | extraction -> file/context -> client -> verify | `036` |
| `BUGRAIL-SPECOS-010` Skills | `038-042` | candidate -> validation -> activation -> client -> verify | `041` |

## Execution Rules

- Follow each file's `dependsOn`; do not execute only by numeric order.
- Recompute the source Spec SHA-256 before promoting an Issue from `draft`.
- A mismatched Spec version/hash makes the Issue stale.
- A verification Issue first derives and approves the exact-version Test Spec,
  then executes evidence; implementation-owned output alone cannot close it.
- Frontend Issues follow `design/specos-client-interaction-design.md` and must
  retain backend authority for gates, readiness, routing, evaluation, approval,
  activation, and rollback.

## First Implementable Slice

Only `BUGRAIL-SPECOS-001` currently has an independently derived draft Test Spec.
Its ADR, Feature Spec, and Test Spec still require approval before Issues
`001-005` can move out of draft. Later groups remain dependency-planned backlog.
