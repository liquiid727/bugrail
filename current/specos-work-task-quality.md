# SpecOS WorkTask Quality Handoff

## Meta

- Date: `2026-08-09`
- Active design: `design/specos-control-plane-design.md`
- Client design: `design/specos-client-interaction-design.md`
- Decision: `design/adr/ADR-001-embed-specos-in-work-task.md` (`proposed`)
- Feature: `BUGRAIL-SPECOS-001` version `0.3` (`draft`)
- Source hash: `81b9aff1353243855173525f5a9111200f00a201674a338871f1b344084d657d`
- Release posture: `not started`

## Current Decision

SpecOS delivery control will deepen existing CodeG modules through the ten
vertical Feature Specs in `.features/roadmap.md`. The first implementation slice
binds one exact Feature Spec and trusted preflight/human gates to a WorkTask,
then enforces those gates in the current merge/complete commands.

The user-visible slice extends the existing Tasks shell with Board traceability
chips and a Task Detail Contract tab. Binding is Preview -> review parsed
identity/AC/gates -> Bind with the preview hash; gate decisions, stale reasons,
approval/waiver, and merge blockers remain inspectable without database access.

No implementation has started. No test, review, or release evidence is claimed.

## Existing Modules In Scope

- WorkTask engine, models, service, migrations, and Git helpers.
- ACP runtime and current Session/Worktree lifecycle.
- Tauri/Axum WorkTask commands and frontend transport.
- Tasks Board/Detail, preflight, merge, completion, and timeline UI.

## Preserved Behavior

- Legacy tasks without a Spec contract.
- Existing WorkTask status values and Git-truth merge behavior.
- Existing CodeG compatibility command/route/protocol/environment names.
- Existing live event buses and transport adapters.

## Next Gate

1. Review and accept/reject ADR-001.
2. Approve Feature Spec and independent Test Spec at the recorded hash.
3. Only then promote Issues `001-005` from draft.
