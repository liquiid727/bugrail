# SpecOS Client Interaction Design For Code: BugRail

## Meta

- Product: `Code: BugRail`
- Status: proposed
- Date: `2026-08-12`
- Product requirements: `.prd/prd-specos-agent-team-context-system.md`
- System design: `design/specos-control-plane-design.md`
- Existing UI baseline: `src/components/tasks/`, `src/lib/api.ts`,
  `src/lib/types.ts`, `src/components/workbench/workbench-content.tsx`
- Applies to: `BUGRAIL-SPECOS-001` through `BUGRAIL-SPECOS-016`

## 1. Purpose

Define the durable client information architecture, interaction model, visual
language, state handling, accessibility, transport usage, and verification
standards for SpecOS capabilities inside BugRail.

The client must let a person answer, without reading SQLite or logs:

1. What Spec and acceptance criteria does this task implement?
2. Why is the task ready, blocked, stale, or eligible to merge?
3. What happened in each run, with which Agent/model/context and evidence?
4. How do tasks depend on and integrate with one another?
5. What evaluation, memory, or Skill proposal was derived, and who approved it?
6. Which expert profile and model were resolved for each Team node?
7. Which Context Package was injected, why, and what degraded or blocked it?

## 2. Existing Surface Is The Product Shell

SpecOS extends the existing Tasks workbench route. It does not create a second
application, replace the four-column board, or add a parallel navigation tree.

Existing interaction contracts remain:

- `TasksPage` owns folder filtering, board actions, dialogs, and selected task.
- `TaskCard` is a compact operational summary, not a full inspector.
- `TaskDetailSheet` is the primary task-level inspection and action surface.
- modal dialogs are used for focused compare/confirm operations.
- `src/lib/api.ts` is the typed client interface over Tauri/Web transports.
- `task://changed` is a refetch nudge; persisted command results are the truth.
- UI text is provided through `next-intl` for all ten existing locales.

## 3. Design Direction

### Subject And Audience

The subject is evidence-backed software delivery. The audience is a developer
or technical lead who needs to inspect a task quickly, then drill into proof
without leaving the coding workbench.

### Signature: The Delivery Rail

The characteristic SpecOS element is a compact vertical rail connecting:

```text
Contract -> Execution -> Evidence -> Decision
```

Each stop has a textual state, evidence count, and direct inspection action.
The rail appears in Task Detail Overview and Run Inspector Summary. It reuses
the visual grammar of the existing WorkTask timeline, but represents delivery
readiness rather than chronological events.

This is the one distinctive addition. The rest of the interface remains quiet,
dense, and consistent with BugRail's current shadcn/Radix workbench.

### Visual Rules

- Reuse current theme variables (`background`, `card`, `muted`, `primary`,
  `destructive`, `border`, `ring`) and all 12 existing theme presets.
- Do not introduce a SpecOS-only palette or hard-coded light/dark surfaces.
- Semantic states use icon + label + tone; color is never the only signal:
  `eligible` emerald, `attention/stale` amber, `failed/blocked` destructive,
  `running` primary, `unknown/legacy` muted.
- Continue the configured UI font. Use monospace only for IDs, hashes, paths,
  commands, commit SHAs, and numeric evidence.
- Preserve the existing `rounded-xl` cards, compact `text-xs` metadata, pill
  filters, and 1px border hierarchy.
- Motion is limited to existing loading spinners, sheet/dialog transitions, and
  a single rail-state transition. Respect `prefers-reduced-motion`.

## 4. Information Architecture

```text
Workbench sidebar
  -> Tasks
       -> Board
       -> Graph
       -> Insights

Board/Graph task selection
  -> Task Detail Sheet
       -> Overview
       -> Contract
       -> Plan
       -> Runs

Runs -> select generation
  -> Run Inspector Dialog
       -> Summary
       -> Timeline
       -> Context
       -> Impact
       -> Route
       -> Evaluation

Insights
  -> Evaluation
  -> Memory Candidates
  -> Skill Candidates
```

Capabilities appear progressively. A tab is hidden until its Feature is
available for the selected project; it must not show fabricated placeholder
data. Missing facts inside an available Feature use explicit empty/legacy
states.

### 4.2 Team And Context Routes

`Teams` and `Context` are first-level workbench routes beside `Tasks`.

- Teams combines project Agent/Model configuration, static Team/Workflow
  editing, an accessible DAG list, and current Team run controls. It covers no
  workspace, loading, empty/starter, invalid definition, running, paused,
  partially failed, canceled, completed, stale data, and transport failure.
- Context combines Provider health, loadout/source budgets, recent immutable
  packages, provenance, and activity. It covers empty, healthy, degraded,
  required-provider blocked, stale, loading, and transport failure.
- Task Detail provides the local join: Contract/Gates, Runs/Dependencies,
  Context Package, and Handoff. Backend decisions remain authoritative.
- Last good persisted content may remain on screen during a refresh error, but
  the stale/degraded banner and retry action must be explicit.

### 4.1 Reviewable Screen Blueprints

These are layout contracts, not final pixel art. A frontend implementation may
adjust spacing but must preserve the information priority and action placement.

Tasks Board:

```text
+ Folder ----------------------+  [Board|Graph|Insights]  [New task]
| TODO          RUNNING        |  REVIEW             DONE
| +-----------+ +-----------+  |  +----------------+ +-----------+
| | Add cache | | API tests |  |  | Auth cleanup   | | Docs      |
| | SPEC-003  | | 1 blocker |  |  | SPEC-001 · 2/3 | | eligible  |
| | Agent ... | | running   |  |  | stale          | |           |
| +-----------+ +-----------+  |  +----------------+ +-----------+
```

Task Detail:

```text
+ Add repository cache -------------------------------------- [x]
| review · Codex                                              |
| [Overview] [Contract] [Plan] [Runs]                         |
|                                                             |
| Contract ---- Execution ---- Evidence ---- Decision          |
| bound        settled        2/3 gates     BLOCKED            |
|                                                             |
| Source Spec     BUGRAIL-SPECOS-001 v0.3   [open] [rebind]   |
| Acceptance      AC01 ... / AC04 ...                          |
| Required gates  preflight passed · human approval pending   |
| Block reason    human-approval has no applicable attempt     |
|                                      [Approve] [Waive...]    |
+ [View session] [Edit]                         [Merge blocked] +
```

Run Inspector:

```text
+ Run #3 · implementation -------------------------------- [x]
| [Summary] [Timeline] [Context] [Impact] [Route] [Evaluation] |
| Contract ---- Execution ---- Evidence ---- Decision          |
| SPEC v0.3     review         qualified     not eligible      |
|                                                             |
| Agent/model  Codex / model-x     Duration  12m 09s           |
| Worktree     task/42              Diff      +128 -31          |
| Failure      test                  Tokens    pending sync      |
|                                                             |
| Evidence references and last durable event                  |
+-------------------------------------------------------------+
```

Dependency Graph and accessible fallback:

```text
Graph:  [ready] [waiting] [blocked] [integration]

[Task A done] ----completion----> [Task B review]
        \----integration source--> [Integration C waiting]

List fallback:
Task B | waits for Task A | completion | satisfied
Task C | waits for Task A | integration_source | handoff missing
```

Insights:

```text
[Evaluation] [Memory Candidates] [Skill Candidates]
Filters: 30 days · strict · all agents · all models

First pass   7 / 12   excluded 3   sample 12
Failures     test 4 · context 2 · provider 1

Candidate rows -> evidence/detail -> preview diff/validation -> explicit apply
```

## 5. Tasks Route

### 5.1 View Switch

The existing Tasks toolbar gains a compact segmented switch after the folder
filter:

```text
[ Board ] [ Graph ] [ Insights ]
```

- `Board` remains the default and preserves current behavior.
- `Graph` shows dependency/integration topology for the current folder.
- `Insights` shows project evaluation and governed learning proposals.
- Selection persists locally per workspace; it is a view preference, not a
  backend fact.
- `Start all` and `New task` remain Board actions and are hidden on Insights.

### 5.2 Task Card Contract

Task cards add no more than two SpecOS metadata chips below the title:

- Spec chip: short Spec ID/version; clicking opens Task Detail `Contract`.
- Delivery chip: `2/2 gates`, `1 blocker`, `stale`, or `integration 2/3`;
  clicking opens the relevant Detail tab.

Priority order when space is limited:

1. blocker/stale;
2. required gate summary;
3. Spec identity;
4. secondary run metadata.

The card never renders AC text, evidence output, context items, route candidate
lists, or handoff bodies.

### 5.3 Graph View

Desktop Graph uses a left-to-right dependency layout inside the current content
area. Each node shows title, current WorkTask status, readiness, Agent, and
gate/integration summary. Edge labels distinguish `completion` and
`integration_source`.

Interactions:

- select node -> open Task Detail;
- keyboard arrow navigation between adjacent nodes;
- filter by ready/waiting/blocked/integration;
- cycle/edit errors appear inline above the graph;
- no drag-to-change dependency until an explicit Edit Dependencies mode is
  entered;
- editing opens one compare-and-save dialog using graph revision CAS.

For more than 100 nodes, render a virtualized list/topology summary rather than
an unbounded DOM graph.

### 5.4 Insights View

Insights is folder/project-scoped and uses three tabs:

- Evaluation: cohort filters, sample quality, outcome/failure summaries.
- Memory Candidates: proposal lifecycle and file preview/apply.
- Skill Candidates: evidence, validation, approval, activation, rollback.

Insights never hides excluded/unknown sample counts and never turns a statistical
comparison into an automatic action.

## 6. Task Detail Sheet

Keep the existing right Sheet (`w-full`, desktop maximum `44rem`) and header,
status chip, Agent identity, and footer actions. Split its growing body into:

```text
[ Overview ] [ Contract ] [ Plan ] [ Runs ]
```

### Overview

- Delivery Rail with Contract, Execution, Evidence, Decision stops.
- existing result summary, current preflight compatibility, changed files, and
  timeline remain available;
- primary next action is contextual: bind Spec, resolve blocker, approve,
  inspect failure, merge, or complete;
- View Session, Edit, cleanup, and Delete remain in the existing footer.

### Contract

- exact Spec ID/version/path/hash;
- selected AC list with stable IDs and full text;
- Gate policy and latest applicable attempts;
- actor, reason, evidence summary, verified head, and timestamps;
- stale comparison and explicit Rebind action;
- human approve/waive controls and resulting decision;
- merge eligibility reason list.

Contract binding is a two-step flow:

1. Choose a repository-local Feature Spec and request backend preview.
2. Review parsed identity/hash/AC, select AC and gates, then bind using the
   preview hash as an optimistic-concurrency token.

### Plan

- readiness summary and unmet/terminal dependencies;
- editable dependency list in allowed WorkTask states;
- integration source eligibility, captured heads, deterministic merge order;
- handoff summaries, conflicts, verification, and unresolved items;
- open source task/session/diff actions.

### Runs

- newest-first run generation list;
- each row: round kind, outcome, Agent/model, duration, token state, gate count,
  changed files, and evidence quality;
- running row updates via refetch nudges;
- select row -> Run Inspector;
- legacy events are labeled `Unscoped history`, never assigned to a fake run.

## 7. Run Inspector

Use a focused Dialog, desktop maximum `56rem`, maximum height `85vh`; on small
screens it becomes a full-screen dialog. It is read-only except for retrying a
failed fetch and following evidence links.

### Summary

- Delivery Rail for the selected `run_seq`;
- Spec, Agent/model, Session/Worktree, outcome, failure category, diff, token
  state, gates, and evidence quality;
- unknown values display `Not recorded`, never `0` or `passed`.

### Timeline

- ordered by durable event ID;
- event-kind filter and evidence links;
- live events are reconciled with a refetch after reconnect.

### Context

- budget totals and pack hash;
- included and excluded items, each with source, path, hash/revision, reason,
  size, and priority;
- content preview only for included, safe, capped text;
- required-missing/over-budget errors include corrective guidance.

### Impact

- seed paths, selected related paths, relation/reason, exact vs heuristic,
  omissions, truncation, duration, and repository revision;
- table/list is authoritative; a small graph is optional and cannot be the only
  representation.

### Route

- explicit/folder/automatic source;
- candidate table with eligible/disqualified, score, and stable reason codes;
- chosen Agent/mode/model/provider and fallback attempts;
- secrets and raw environment diagnostics never render.

### Evaluation

- run evaluation fact, evidence quality, normalized failure, first-pass/rework,
  gates, interventions, duration/tokens, and cohort inclusion/exclusion reason;
- evaluation is visibly read-only.

## 8. Client Interface And State Ownership

### 8.1 Wire Types And Client Functions

- Rust DTO serialization is the wire source of truth.
- Exact TypeScript mirrors live in `src/lib/types.ts` or focused files exported
  from it; snake_case wire fields are preserved.
- All command calls are named functions in `src/lib/api.ts`; React components do
  not call `getTransport()` directly.
- Tauri and Web use the same function signature and error projection.
- New list operations use cursor/page DTOs, not unbounded arrays.

### 8.2 Feature Data Modules

Do not keep adding unrelated `useState` and fetch effects to
`task-detail-sheet.tsx`. Extract focused modules:

```text
src/components/tasks/specos/
  delivery-rail.tsx
  task-contract-panel.tsx
  contract-bind-dialog.tsx
  task-plan-panel.tsx
  task-runs-panel.tsx
  run-inspector-dialog.tsx
  context-inspector.tsx
  impact-inspector.tsx
  route-inspector.tsx
  evaluation-inspector.tsx
  task-graph-view.tsx
  delivery-insights-view.tsx

src/hooks/specos/
  use-task-contract.ts
  use-task-plan.ts
  use-task-runs.ts
  use-run-inspector.ts
  use-delivery-insights.ts
```

Hooks own request IDs, stale-response rejection, refresh, mutation busy state,
and error normalization. Presentational modules receive typed view data and
callbacks. No new client-state dependency is introduced for these Features.

### 8.3 Refresh Rules

- first open: show local skeleton only for the panel being fetched;
- refresh: keep stale data visible with a subtle updating indicator;
- mutation: disable only conflicting actions, not the entire Sheet;
- `task://changed`: refetch task summary and the active Detail panel;
- reconnect: refetch active task/panel/run because WebSocket events may be lost;
- closing Sheet/Dialog invalidates pending response writes through request ID or
  abort signal;
- backend errors retain the last good data and provide an inline Retry action.

## 9. Interaction And Feedback Standards

- Use inline validation for paths, AC selection, Gate policy, dependency edges,
  reasons, and stale revisions. Toasts confirm successful mutations or report
  failures; they do not carry the only explanation.
- Destructive or contract-changing operations use a Dialog with consequences:
  Rebind, remove dependency with effects, waive gate, activate/rollback Skill.
- Buttons use outcome language: `Bind Spec`, `Save dependencies`, `Approve
  gate`, `Waive gate`, `Refresh integration plan`, `Activate Skill`, `Rollback`.
- A blocked action remains discoverable. Prefer disabled-with-visible-reason or
  an enabled inspect action over silently hiding it.
- Backend decisions are shown verbatim as structured reasons; the client does
  not recalculate merge eligibility, readiness, route, or evidence quality.
- File, Spec, Session, Worktree, commit, test, and review references open through
  existing BugRail workspace actions when available.

## 10. Required Async States

Every feature panel defines these states:

| State | Required behavior |
|---|---|
| Empty | Explain what is absent and provide the valid next action. |
| Loading | Skeleton/spinner within the affected region; existing task content stays visible. |
| Success | Show authoritative facts, last update/revision when relevant, and valid next actions. |
| Partial/legacy | Name missing attribution explicitly; never infer success. |
| Blocked/stale | Show every reason and the action that can resolve it. |
| Failure | Preserve last good content, render inline error and Retry. |
| Mutating | Prevent duplicate/conflicting submission and retain cancel/close behavior where safe. |

## 11. Responsive Behavior

- `>= 1280px`: current four-column Board; `44rem` Detail Sheet; `56rem` Run
  Inspector; Graph uses full content width.
- `768-1279px`: Board retains horizontal scroll; Sheet may use up to 80vw;
  Inspector uses up to 92vw; metadata grids collapse from 3 to 2 columns.
- `< 768px`: Detail and Run Inspector are full-screen; tabs horizontally
  scroll with visible focus; Graph becomes a dependency list grouped by
  ready/waiting/blocked; Insights becomes one column; sticky action footer
  respects safe-area insets.
- No information or action is available only on hover.

## 12. Accessibility And Localization

- Radix Dialog/Sheet/Tabs provide focus management; new custom graph/list
  interactions must expose keyboard navigation and visible focus.
- Status uses readable text and icon with accessible name; decorative icons set
  `aria-hidden`.
- Evidence lists, candidate comparisons, and metrics use semantic tables/lists;
  graphs have equivalent text representations.
- Changes announced after mutation use the existing toast/live-region behavior.
- All strings live under a stable `Tasks.specos.*` or `DeliveryInsights.*`
  namespace in all ten locale files.
- Layout tolerates 30% text expansion and Arabic RTL. IDs/hashes remain LTR with
  isolated monospace spans.
- Dates/numbers use locale formatting; raw backend timestamps are not displayed.

## 13. Frontend Verification Standard

Each Feature's frontend Issue must include:

1. TypeScript wire-contract tests for nullable/legacy/pagination shapes.
2. API client tests asserting exact command name and request payload.
3. Testing Library interaction tests for every Section 10 state.
4. Keyboard/focus tests for Dialog, Sheet, Tabs, graph/list, and confirmations.
5. Reconnect/stale-response/duplicate-submit tests where data mutates.
6. All-locale key consistency; no hard-coded user-facing strings.
7. Light/dark and narrow/wide manual visual verification screenshots for new
   primary surfaces.
8. `pnpm lint`, focused `pnpm test`, full `pnpm test`, and `pnpm build` before
   Feature verification closes.

Snapshot-only tests are insufficient. Tests assert actions, visible state,
backend reason rendering, and accessibility semantics.

## 14. Feature-To-Surface Matrix

| Feature | Board | Detail Contract | Detail Plan | Detail Runs | Run Inspector | Graph | Insights |
|---|---|---|---|---|---|---|---|
| `001` Contract/Gates | Spec + gate chip | bind/AC/gates/decision | — | — | — | — | — |
| `002` Run Evidence | current run hint | — | — | run list | summary/timeline | — | — |
| `003` Dependencies | blocker chip | — | readiness/editor | — | — | dependency graph | — |
| `004` Integration | source progress | — | sources/handoffs/conflicts | integration runs | integration evidence | integration edges | — |
| `005` Context Pack | — | — | — | context hash | Context tab | — | — |
| `006` Impact | — | — | — | impact state | Impact tab | optional relations | — |
| `007` Routing | Agent/model hint | — | — | route hint | Route tab | — | — |
| `008` Evaluation | evidence quality | — | — | evaluation state | Evaluation tab | — | Evaluation |
| `009` Memory | — | — | — | source refs | memory source links | — | Memory Candidates |
| `010` Skill | — | — | — | source refs | Skill source links | — | Skill Candidates |

## 15. Manual Review Script

A reviewer must be able to verify the client without database or log access:

1. Open Board and confirm existing four-column behavior remains intact.
2. Open an unbound task, preview a Spec, bind selected AC/gates, and see the
   Contract stop change from empty to bound.
3. Open a blocked task from its card chip, inspect every blocker, and confirm a
   stale/forged UI cannot bypass the backend decision.
4. Open Runs, select a generation, then traverse Timeline, Context, Impact,
   Route, and Evaluation with missing/legacy values labeled honestly.
5. Switch to Graph, inspect and keyboard-navigate dependency/integration facts;
   verify the list fallback contains equivalent information.
6. Switch to Insights and inspect sample/exclusion math, a Memory diff preview,
   and the separate Validate -> Approve -> Activate Skill lifecycle.
7. Repeat the primary flow in light/dark themes, at desktop and phone widths,
   using keyboard only; retain screenshots as Issue evidence.

An implementation is not review-ready if a reviewer must query SQLite, inspect
developer tools, or read backend logs to determine any displayed decision.

## 16. Non-Goals

- Replacing the existing Tasks board or Session viewer.
- Showing full raw transcripts, prompts, secrets, or uncapped command output in
  an Inspector.
- Rendering large graphs without an accessible list/table alternative.
- Client-side authority for gates, readiness, routing, evaluation, approval,
  activation, or rollback.
- Adding a new design system, chart library, data-fetching dependency, or font
  solely for SpecOS.
