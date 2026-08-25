---
id: BUGRAIL-SPECOS-030-TEST
version: "0.1"
status: draft
sourceSpecId: BUGRAIL-SPECOS-030
sourceSpecVersion: "0.1"
sourceSpecHash: "35429d89b8f40478dd1775daa4608a1c29adf32355e3d39ac2775e6c16528eea"
independentFromImplementation: true
---

# Test Spec: Memory Governance And Hub

## 1. Strategy

Use deterministic and pinned TencentDB fixtures to verify the normalized
Memory interface, local governance overlay and immutable Context evidence.
Remote Memory content must not be mirrored into SQLite.

## 2. Test Cases

| ID | Requirements | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-030.T01` | R01,R05,R06 | Search/page/get mixed project, team, agent and user fixtures and drill into L0 evidence. | Bounded stable pages expose only authorized records and safe evidence references. |
| `BUGRAIL-SPECOS-030.T02` | R02-R04 | Correct one record twice, replay requests and recall during partial remote success. | One idempotent supersession chain identifies exactly one effective record with traceable mutation outcomes. |
| `BUGRAIL-SPECOS-030.T03` | R02-R04 | Delete/invalidate a record while stale upstream search still returns it and while retry is pending. | Local policy suppresses it before Context injection and records the reason in later packages. |
| `BUGRAIL-SPECOS-030.T04` | R02,R03 | Exercise conflict, TTL expiry, duplicates and exact budget boundaries. | Deterministic inclusion/exclusion and content hashes survive restart without storing remote body content locally. |
| `BUGRAIL-SPECOS-030.T05` | R04-R06 | Run Hub search/mutations through Tauri/Axum with timeout, malformed text, XSS fixtures and missed events. | Equivalent last-good/error state, authenticated mutation audit and text-safe UI reconstruct after restart. |

## 3. Required Evidence

- Memory Adapter contract and governance repository/migration tests.
- Recall/Context integration tests for suppression and immutable reasons.
- Backend scope matrix and command-core/Tauri/Axum parity tests.
- Hub interaction, accessibility, localization and provider-failure evidence.

## 4. Exclusions

Frontend filtering, local copies of all Memory content, provider screenshots or
a mutation response without later recall proof cannot satisfy this Test Spec.
