---
id: BUGRAIL-SPECOS-028-TEST
version: "0.1"
status: approved
sourceSpecId: BUGRAIL-SPECOS-028
sourceSpecVersion: "0.1"
sourceSpecHash: "f4b2884897e864cc28022536fd5b61d05eb2e7fffbee096154bd5bb308cc8c4a"
independentFromImplementation: true
---

# Test Spec: Independent Context Plugin Foundation

## 1. Strategy

Use deterministic Memory, Wiki, CodeGraph and Skill Adapters plus persisted
SQLite job/config facts. Verify only public plugin, Context and command-core
surfaces; Adapter private state and transient event delivery are not oracles.

## 2. Test Cases

| ID | Requirements | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-028.T01` | R01,R03 | Register one Adapter of each kind and attempt cross-kind calls, duplicate IDs, unknown implementations and invalid manifests. | Four isolated capability/health projections exist; invalid configuration makes no Adapter/network call. |
| `BUGRAIL-SPECOS-028.T02` | R02 | Normalize Memory, Wiki, graph and Skill fixtures with duplicates, scope conflicts and exact budget edges. | Existing Context compiler records stable `AssetRef`, inclusion/exclusion, ordering and provenance without vendor DTOs. |
| `BUGRAIL-SPECOS-028.T03` | R04,R05 | Submit duplicate jobs, crash while running, exhaust retry and replay refresh events. | One idempotency fact and bounded attempt history survive restart; no duplicate external mutation or parallel task state appears. |
| `BUGRAIL-SPECOS-028.T04` | R03,R06 | Exercise malformed config, credential-bearing values, oversized errors and malicious provider text. | Secrets/payloads are absent from persisted facts, logs, frontend payloads and diagnostics; stable safe errors remain. |
| `BUGRAIL-SPECOS-028.T05` | R05,R06 | Perform config, health and job operations through command-core, Tauri and Axum with missed/reordered events. | Transport projections are equivalent and reconstruct from persisted facts after restart. |

## 3. Required Evidence

- Contract tests for all four Adapter kinds and normalized Context candidates.
- Migration/repository tests for job uniqueness, leasing, recovery and bounds.
- Shared command-core plus Tauri/Axum parity tests and UI state coverage.
- Existing frontend and Rust checks required by repository handoff.

## 4. Exclusions

Dynamic plugin loading, frontend-only state, successful event receipt or one
catch-all TencentDB mock cannot satisfy this Test Spec.
