# SpecOS Product Vision For Code: BugRail

> Status: product direction
> Product: Code: BugRail
> Scope: long-term intent; interfaces and delivery scope are defined elsewhere

## Problem

CodeG already gives BugRail a mature workbench for projects, conversations,
Agent sessions, Git, Worktrees, ACP agents, delegation, automations, and desktop
or server operation. Complex engineering work still requires the developer to
coordinate several concerns manually:

- connect a request to a stable specification and acceptance criteria;
- decide which tasks can run in parallel and which must wait;
- provide each Agent with the relevant project context;
- distinguish an Agent finishing a turn from a change being safe to merge;
- retain review, test, decision, cost, and failure evidence across sessions;
- reuse repeated successful behavior without treating one run as a new Skill.

SpecOS adds these engineering-control capabilities inside BugRail while keeping
the inherited CodeG workbench and runtime contracts intact.

## Product Principles

1. **Extend the current workbench.** WorkTask, ACP, Worktree, event, database,
   transport, and Tasks UI modules are the starting point.
2. **Artifact references are explicit.** A task records the exact Spec ID,
   version, content hash, acceptance criteria, and evidence it uses.
3. **Agent completion is an input.** Merge and completion decisions are made by
   declared quality gates and recorded evidence.
4. **Automation remains controllable.** Users can inspect, override, retry,
   cancel, or take over a running Session.
5. **Context is selected per run.** Every selected item records its source and
   reason; later Features may optimize the selection.
6. **Existing compatibility identifiers are preserved.** Product-facing BugRail
   identity and inherited CodeG wire identity are managed separately.
7. **Learning is evidence-based.** Memory and Skill candidates require repeated
   observations, validation, and explicit lifecycle state.

## Target User Flow

```text
request or approved Feature Spec
  -> one or more WorkTasks
  -> exact Spec and acceptance snapshot
  -> existing Worktree + ACP Session execution
  -> structured gate results and independent review
  -> merge/complete decision
  -> timeline and evidence retained for later evaluation
```

The first delivery slice covers the middle of this flow: binding a WorkTask to
a Spec and preventing merge or completion until its required gates pass.

## Capability Sequence

1. Spec-linked WorkTask and quality gates.
2. Run trace and Artifact/Context inspection using existing task timelines.
3. WorkTask dependency graph and integration coordination.
4. Context selection and code intelligence.
5. explainable Agent/model routing.
6. evaluation, project memory, and controlled Skill evolution.

Each step is delivered as a small Feature/Test Spec pair. Later steps may deepen
existing modules; they do not require all future interfaces to be frozen now.

## Non-Goals

- Reimplementing the editor, terminal, Git client, Worktree manager, ACP runtime,
  session UI, or existing delegation flow.
- Adding a second workflow state machine beside WorkTask.
- Adding an independent Event Bus without a delivery requirement the existing
  typed bus and transport emitters cannot satisfy.
- Treating every internal policy as a plugin.
- Allowing a task to mark its own evidence as independently verified.

