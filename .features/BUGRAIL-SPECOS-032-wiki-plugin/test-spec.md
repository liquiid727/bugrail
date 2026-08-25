---
id: BUGRAIL-SPECOS-032-TEST
version: "0.1"
status: draft
sourceSpecId: BUGRAIL-SPECOS-032
sourceSpecVersion: "0.1"
sourceSpecHash: "44862b315a4620bb49f998eccc3083ea7d18659484f464602e1be0eedb9b64d0"
independentFromImplementation: true
---

# Test Spec: Independent Wiki Plugin

## 1. Strategy

Index revisioned Markdown/ADR fixtures through deterministic and pinned Wiki
Adapters. Canonical source files and citations, not generated Wiki text, are
the authority.

## 2. Test Cases

| ID | Requirements | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-032.T01` | R01-R03,R05 | Register/import mixed sources and reject malformed or wrong-kind Adapter responses. | Revisioned pages and section citations map to exact source hashes without Memory DTOs. |
| `BUGRAIL-SPECOS-032.T02` | R02,R03 | Change, rename and delete sources; crash during incremental publication and retry. | Only affected pages publish atomically; last-success/stale/error facts survive restart. |
| `BUGRAIL-SPECOS-032.T03` | R04 | Search duplicate/conflicting Wiki pages at exact Context budget edges. | Bounded candidates retain citations; current source/accepted ADR wins with a recorded exclusion reason. |
| `BUGRAIL-SPECOS-032.T04` | R02,R04 | Exercise project isolation, symlinks, excluded roots, malicious documents and embedded instructions. | No cross-root read/query occurs and generated text remains marked untrusted. |
| `BUGRAIL-SPECOS-032.T05` | R03,R06 | Sync/rebuild/search/browse through Tauri/Axum during provider failure and restart. | Equivalent last-good/stale/job state and safe citations reconstruct from persisted facts. |

## 3. Required Evidence

- Independent `WikiProvider` contract suite including pinned production Adapter.
- Source registry and provider-job migration/restart tests.
- Context provenance/conflict and filesystem confinement tests.
- Wiki UI, transport parity, accessibility and localization evidence.

## 4. Exclusions

Memory search renamed as Wiki, uncited generated pages or a shared catch-all
TencentDB mock cannot satisfy this Test Spec.
