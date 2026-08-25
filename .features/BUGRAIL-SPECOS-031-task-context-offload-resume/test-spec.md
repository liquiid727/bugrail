---
id: BUGRAIL-SPECOS-031-TEST
version: "0.1"
status: draft
sourceSpecId: BUGRAIL-SPECOS-031
sourceSpecVersion: "0.1"
sourceSpecHash: "537c3e75a4432ce8b56a87656198a990ba7b98eed55eb1ec9f1b8fb0e317ddf4"
independentFromImplementation: true
---

# Test Spec: Task Context Offload And Resume

## 1. Strategy

Drive real WorkTask/run evidence with bounded deterministic summarization.
Artifact bytes, hashes, generations and Context Packages are the oracles; an
Agent assertion that it remembers the task is not evidence.

## 2. Test Cases

| ID | Requirements | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-031.T01` | R01,R02 | Emit 10 MiB tool/terminal/file results around exact thresholds. | Large bytes stay outside the prompt; stable refs, safe bounded summaries and source hashes remain usable. |
| `BUGRAIL-SPECOS-031.T02` | R01,R03 | Crash before/during artifact write, summarization and canvas publication; replay the event. | At most one complete artifact/job is published and incomplete writes cannot enter Context. |
| `BUGRAIL-SPECOS-031.T03` | R03,R04 | Resume after process restart and after source evidence changes. | A new WorkTask generation binds the effective canvas/summaries and selected refs without replaying the transcript. |
| `BUGRAIL-SPECOS-031.T04` | R02,R03,R05 | Feed prompt injection, secrets, symlinks, excluded paths and malicious encodings. | Stored/derived data remains scoped and untrusted; excluded content is absent from summary, Memory and package evidence. |
| `BUGRAIL-SPECOS-031.T05` | R05 | Exceed quota and run cleanup across active, reviewable, superseded and expired generations. | Active/reviewable refs remain; only eligible artifacts are reclaimed with audited counts. |
| `BUGRAIL-SPECOS-031.T06` | R04,R06 | Inspect refs/canvas/checkpoints and resume through Tauri/Axum under degraded Memory. | Equivalent UI/API state preserves WorkTask authority and supports resume without Memory availability. |

## 3. Required Evidence

- Artifact boundary, atomic-write, hash and confinement tests.
- Provider-job restart/idempotency and WorkTask generation integration tests.
- Context budget/injection tests and retention/quota database oracles.
- Task UI, transport parity, accessibility and localization evidence.

## 4. Exclusions

Silent truncation, full transcript replay, mutable same-generation resume or
Memory-provider-owned task status cannot satisfy this Test Spec.
