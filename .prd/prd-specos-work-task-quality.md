# PRD: Spec-Linked WorkTask Quality

## Meta

- Status: draft
- Parent PRD: `.prd/prd-specos-delivery-control.md`
- Product vision: `docs/specos/product-vision.md`
- Design: `design/specos-control-plane-design.md`
- Target Feature: `BUGRAIL-SPECOS-001`

## Problem

A current WorkTask can run an Agent in a Worktree, collect a verdict, run one
preflight command, show a timeline, and let the user merge or complete the task.
It does not record which approved Spec version and acceptance criteria the task
implements, and the merge decision cannot express several independently
required checks.

## User Outcome

The developer can bind a WorkTask to one exact Feature Spec version, see its
acceptance criteria and required gates in Task Detail, and receive a concrete
explanation when merge or completion is blocked.

## Requirements

| ID | Requirement |
|---|---|
| `P-WTQ-01` | A WorkTask may bind to one repository-local Feature Spec by ID, version, path, and content hash. |
| `P-WTQ-02` | The binding captures the selected acceptance criteria and required gate policy. |
| `P-WTQ-03` | Existing WorkTasks without a binding retain current behavior. |
| `P-WTQ-04` | Existing WorkTask preflight and explicit human approval outcomes can be recorded as structured gate evidence through trusted producer paths. |
| `P-WTQ-05` | Agent completion alone cannot pass a required independent gate. |
| `P-WTQ-06` | Merge and no-change completion are rejected while a required gate is unmet. |
| `P-WTQ-07` | Task Detail shows the source Spec, acceptance criteria, gate states, evidence, and block reason. |
| `P-WTQ-08` | Retry and rebind behavior does not silently reuse stale Spec or gate evidence. |

## Non-Goals

- Building a generic Artifact database.
- Adding a new task state machine or event bus.
- Automatic Issue DAG creation, context ranking, model routing, or memory.
- Test/review/security gate producers; later Features add them only with a
  trusted execution or review source.
- Renaming inherited CodeG commands or protocols.
- Claiming release readiness without an approved Test Spec and normalized
  evidence.

## Success Criteria

- A bound task cannot merge when any required gate is pending, failed, blocked,
  or invalidly waived.
- The user can identify the unmet gate and its evidence from Task Detail.
- Legacy tasks pass existing regression tests without data migration by users.
- Desktop and standalone-server command behavior remains equivalent.
