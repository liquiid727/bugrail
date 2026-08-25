---
id: BUGRAIL-SPECOS-034-TEST
version: "0.1"
status: draft
sourceSpecId: BUGRAIL-SPECOS-034
sourceSpecVersion: "0.1"
sourceSpecHash: "af88e70654e54f0f1d7b4d1ba89d92885ea9fabe31f607d06b44168644e1bcd2"
independentFromImplementation: true
---

# Test Spec: Independent Skill Evolution Plugin

## 1. Strategy

Generate candidates from deterministic WorkTask traces and validate them in a
constrained fixture. Version files, lifecycle facts, evidence links and later
Context Packages are the acceptance oracles.

## 2. Test Cases

| ID | Requirements | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-034.T01` | R01-R03 | Feed one trace, repeated matches, near-duplicates and unrelated successes. | One trace never publishes; threshold matches create one deduplicated draft with scores and source links. |
| `BUGRAIL-SPECOS-034.T02` | R03,R04 | Validate success, timeout, permission denial and malicious candidate resources; crash/retry validation. | Generation-safe states retain evidence; validation cannot publish or mutate the current published version. |
| `BUGRAIL-SPECOS-034.T03` | R03-R05 | Approve, publish, disable and roll back versions while runs resolve Skills. | Audited immutable versions and later Context Packages select only the effective authorized generation. |
| `BUGRAIL-SPECOS-034.T04` | R05,R06 | Route overlapping triggers/scopes with missing or stale Wiki/CodeGraph refs. | Bounded Top-K result explains version/scope/reason and fails safely on unavailable refs. |
| `BUGRAIL-SPECOS-034.T05` | R01,R02,R04 | Inject secrets, prompt instructions and unsafe executable resources into traces. | Unsafe material is absent from publishable procedure content and requires explicit rejection/review evidence. |
| `BUGRAIL-SPECOS-034.T06` | R07 | Operate inbox/diff/evidence/validation/publish/rollback through Tauri/Axum and restart. | Equivalent last-good/error state reconstructs from version and lifecycle facts. |

## 3. Required Evidence

- Schema/version migration and candidate detector/dedup tests.
- Constrained validation, lifecycle concurrency and rollback tests.
- Context routing/scope/provenance and security fixtures.
- Skill UI, transport parity, accessibility and localization evidence.

## 4. Exclusions

One-shot self-publication, model assertions, Memory atoms used as procedures or
copied Wiki/CodeGraph snapshots cannot satisfy this Test Spec.
