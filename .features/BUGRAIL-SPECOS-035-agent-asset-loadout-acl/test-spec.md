---
id: BUGRAIL-SPECOS-035-TEST
version: "0.1"
status: draft
sourceSpecId: BUGRAIL-SPECOS-035
sourceSpecVersion: "0.1"
sourceSpecHash: "0c2c2e143ff2dd5bbeabd2f8aeb39ff9227e605b2b71c15ef690dcd7bcd00486"
independentFromImplementation: true
---

# Test Spec: Agent Asset Loadouts And ACL

## 1. Strategy

Resolve loadouts for deterministic projects, users, Teams, Agents and WorkTask
generations. Backend Adapter call capture and immutable Context Packages, not
visible UI controls, prove authorization.

## 2. Test Cases

| ID | Requirements | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-035.T01` | R01,R05 | Launch two Agents on one model with different presets/inheritance and revise config mid-run. | Each generation retains its exact effective plugin policy, budgets, revisions and exclusions. |
| `BUGRAIL-SPECOS-035.T02` | R02,R03 | Execute the project/user/team/agent/task/private scope matrix for all four plugin kinds. | Unauthorized requests are rejected before network/index access and no cross-project/private candidate enters Context. |
| `BUGRAIL-SPECOS-035.T03` | R03,R04 | Handoff planner to implementer to reviewer with task evidence, private Memory and unconfirmed hypotheses. | Attributable task assets transfer; private/unconfirmed assets remain excluded with reasons. |
| `BUGRAIL-SPECOS-035.T04` | R04 | Run A/B variants that produce conflicting Memory and Skill candidates, then select a winner. | Variants stay isolated until explicit evidence-backed resolution; only the winner may promote shared assets. |
| `BUGRAIL-SPECOS-035.T05` | R05 | Restart and revise profiles while old/new generations are inspected. | Historical package evidence is immutable and new generations use the new revision only. |
| `BUGRAIL-SPECOS-035.T06` | R06 | Edit/inspect presets and denials through Tauri/Axum, including tampered frontend requests. | Equivalent backend-enforced results and safe denial reasons survive restart. |

## 3. Required Evidence

- Resolver property/scope-matrix tests for all plugin kinds.
- Adapter preflight tests proving rejection before external access.
- Team handoff/A-B/Context generation integration tests.
- Loadout UI, tamper, transport parity and localization evidence.

## 4. Exclusions

Hidden buttons, frontend filters, same-model equality assumptions or mutable
historical loadouts cannot satisfy this Test Spec.
