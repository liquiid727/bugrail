# PRD: Memory Operating Layer Roadmap

## Meta

- Status: product roadmap draft
- Date: 2026-08-23
- Product: Code: BugRail
- Implemented baseline: `BUGRAIL-SPECOS-006` through `009` and `017`
- Product vision: `.features/bugrail-specoos-memory/`
- Decisions: ADR `003-004`

This PRD converts the full Memory/Knowledge/Skill vision into independent
delivery slices. It does not authorize implementation by itself. Every slice
requires an approved Feature Spec, exact-version Test Spec, implementation
Issues, verification evidence and review.

## 1. Product Decision

TencentDB Agent Memory is BugRail's primary long-term Memory Engine. BugRail
owns capture policy, identity, delivery, recall requests, Context selection,
prompt injection, observability and UI. TencentDB owns durable L0-L3 memory,
extraction and retrieval behind the Memory interface established by
`BUGRAIL-SPECOS-017`.

Wiki, CodeGraph and Skill Evolution are independent plugins. They may reuse a
managed TencentDB deployment, shared identity, `AssetRef`, durable provider
jobs and the normalized Context item envelope, but they do not become methods
on `MemoryProvider` and do not share one catch-all plugin manifest.

## 2. Current Baseline

The repository already has:

- immutable Context Packages, budgets, loadouts and provenance;
- provider configuration and health projection;
- a deep Memory interface with TencentDB v3 health/capture/recall;
- a restart-safe capture outbox and WorkTask settlement hook;
- pre-ACP Memory recall and Context injection;
- Context UI operations for health, delivery and recall preview;
- an independent `code_intelligence` module backed by
  `codebase-memory-mcp`;
- existing Agent Profiles, custom Skills, WorkTasks, events and transports.

The baseline does not include a managed TencentDB runtime, Memory governance,
short-term task offload, Wiki, Skill Evolution, complete CodeGraph delivery,
cross-asset ACL/loadouts, Memory Hub or full operational hardening.

## 3. Product Goals

| ID | Goal |
|---|---|
| `MOL-G01` | A user can install, start, inspect, back up and recover the pinned TencentDB Memory runtime without a terminal. |
| `MOL-G02` | Cross-session Memory can be searched, traced, corrected, invalidated and deleted through BugRail policy and UI. |
| `MOL-G03` | Long WorkTasks offload large outputs into bounded references and resume from durable summaries and a task canvas. |
| `MOL-G04` | Wiki, CodeGraph and Skill Evolution run as independent plugins and contribute normalized Context candidates. |
| `MOL-G05` | Agent loadouts and backend ACLs control which scoped assets each run may receive. |
| `MOL-G06` | Provider failure, restart, upgrade and high-volume operation are diagnosable and release-tested across desktop and server modes. |

## 4. Delivery Features

| Feature | Outcome | Depends on |
|---|---|---|
| `BUGRAIL-SPECOS-028` | Independent plugin/asset/job foundation | `006-009`, `017` |
| `BUGRAIL-SPECOS-029` | Managed TencentDB Memory runtime | `017`, `028` |
| `BUGRAIL-SPECOS-030` | Memory governance and Memory Hub | `017`, `028-029` |
| `BUGRAIL-SPECOS-031` | Task context offload and resume | `003`, `006`, `009`, `028-029` |
| `BUGRAIL-SPECOS-032` | Independent Wiki plugin | `006`, `009`, `028-029` |
| `BUGRAIL-SPECOS-033` | Independent CodeGraph plugin | `006`, `009`, `028` |
| `BUGRAIL-SPECOS-034` | Independent Skill Evolution plugin | `003`, `009`, `028` |
| `BUGRAIL-SPECOS-035` | Agent asset loadouts and ACL | `002`, `008`, `028`, `030`, `032-034` |
| `BUGRAIL-SPECOS-036` | Operations and release hardening | `029-035` |

## 5. Ownership Rules

1. `MemoryProvider` remains a deep Memory-only interface.
2. Wiki, CodeGraph and Skill each own a separate interface and Adapter.
3. Context remains the only prompt-composition authority.
4. WorkTask remains the task and recovery state machine.
5. Existing EventEmitter and internal bus remain refresh transports; SQLite and
   validated project configuration are reconstructible truth.
6. Provider jobs may add durable job rows only for external asynchronous work;
   they do not replace WorkTask or Automation.
7. Renderer code never calls a vendor endpoint directly.
8. Credentials are references to backend secure storage and never enter Git,
   logs, Context Packages or frontend payloads.

## 6. Release Strategy

Deliver `028` and `029` first. Memory governance and task offload may then
advance independently. Wiki and CodeGraph may run in parallel after the plugin
foundation; Skill Evolution follows once repeatable run evidence is available.
Loadout/ACL integration follows the individual plugins, and `036` is the final
release gate.

No Feature may claim another Feature's behavior from configuration fields or
UI placeholders. The full product is complete only when all Features have
accepted Test Spec evidence and the final operations/hardening gate passes.

