---
id: BUGRAIL-SPECOS-029-TEST
version: "0.1"
status: draft
sourceSpecId: BUGRAIL-SPECOS-029
sourceSpecVersion: "0.1"
sourceSpecHash: "bf4396c4bec625f2501b7499c00d8ec3b97b7194c1ecdb69d8d9b1914e5a634c"
independentFromImplementation: true
---

# Test Spec: Managed TencentDB Memory Runtime

## 1. Strategy

Test a signed local fixture and remote endpoint mode against the exact Memory
contract. Runtime lifecycle evidence must include manifest digest, process
identity, schema/version result and durable operation history.

## 2. Test Cases

| ID | Requirements | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-029.T01` | R01,R03 | Install a valid pin; reject bad signature/checksum, unknown patch, unsafe URL and missing secret reference. | Only the declared digest reaches the managed cache; no credential is persisted or returned. |
| `BUGRAIL-SPECOS-029.T02` | R02,R06 | Start/stop/restart from two windows/processes and race lifecycle calls. | One lock owner/process identity and deterministic lifecycle projection survive restart. |
| `BUGRAIL-SPECOS-029.T03` | R02,R04 | Crash the runtime during capture, recover it and drain the existing outbox. | Runtime recovers with bounded backoff; accepted message IDs are not duplicated in L0. |
| `BUGRAIL-SPECOS-029.T04` | R01,R04,R05 | Exercise exact pin, vanilla/unsupported version, schema mismatch, failed migration and rollback. | Writes remain blocked until compatible; prior pin/data restarts after failed migration with audited state. |
| `BUGRAIL-SPECOS-029.T05` | R05 | Back up scoped fixtures, mutate them, restore, and race restore with capture/upgrade. | Identity-isolated Memory and manifest checksums round trip; mutually exclusive operations cannot overlap. |
| `BUGRAIL-SPECOS-029.T06` | R03,R06 | Inspect settings/diagnostics through Tauri/Axum in local and remote mode under provider failure. | Equivalent last-good/error projections contain no secret, raw payload or upstream DTO. |

## 3. Required Evidence

- Signed/checksummed runtime fixture and exact TencentDB contract run.
- Supervisor concurrency/crash-loop and AppState restart tests.
- Backup/migration/rollback integration evidence with version identities.
- Desktop/server UI, command parity and secret-scanning results.

## 4. Exclusions

Manual terminal startup, an unpinned image, `/health` alone, MemoryPanel or a
configuration screenshot cannot satisfy this Test Spec.
