---
id: BUGRAIL-SPECOS-007
version: "0.1"
title: "Explainable Agent And Model Routing"
status: draft
changeType: work-task-routing-deepening
prd: ".prd/prd-specos-delivery-control.md"
design: "design/specos-control-plane-design.md"
clientDesign: "design/specos-client-interaction-design.md"
codeBaseline: "55545d43"
dependsOn: [BUGRAIL-SPECOS-002, BUGRAIL-SPECOS-005]
---

# BUGRAIL-SPECOS-007: Explainable Agent And Model Routing

## 1. Summary

Replace WorkTask's implicit override/default choice with a deterministic routing
decision that still uses existing ACP registries, agent settings, model provider
associations, and `config_values`. The decision is immutable per run,
inspectable, and always subordinate to explicit user selection and gate policy.

### Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-007.R01` | Before launch, the WorkTask module resolves one installed/enabled Agent, mode, model, and ordered fallback candidates. |
| `BUGRAIL-SPECOS-007.R02` | Explicit task selection wins; folder selection is next; automatic candidates are used only when neither exists. |
| `BUGRAIL-SPECOS-007.R03` | Automatic scoring uses declared task needs, task kind, risk, context estimate, Agent capabilities, availability, and project preference. |
| `BUGRAIL-SPECOS-007.R04` | The run stores candidates, disqualifications, scores, chosen route, policy version, and reason codes without secrets. |
| `BUGRAIL-SPECOS-007.R05` | Pre-prompt spawn/config failure may try the recorded fallback chain; after a prompt begins, fallback requires a new run generation. |
| `BUGRAIL-SPECOS-007.R06` | Routing and fallback cannot remove gates, widen permissions, or change the bound Spec. |

PRD coverage: `P-DC-11`, `P-DC-12`, `P-DC-16`, `P-DC-18`.

## 2. Existing Modules And Policy

| Existing module | Role |
|---|---|
| `acp/registry.rs` + `custom_registry.rs` | Candidate identity and runtime metadata. |
| `agent_setting_service.rs` | Enabled/installed state, order, provider association. |
| `acp_describe_agent_options_core` | Supported modes/config/model options. |
| `model_provider_service` | Provider/model configuration and health inputs. |
| `WorkTaskConfig` / folder settings | Explicit overrides and current preference chain. |
| `work_task/engine.rs` | Consume one stored route; do not reimplement scoring. |

Internal interface:

```text
resolve(RouteRequest, CandidateCatalog) -> RouteDecision | RouteError
```

The first policy is deterministic. It does not consume learned Eval scores;
`BUGRAIL-SPECOS-008` may later add an explicitly versioned input.

## 3. Capability And Decision Contract

Project routing policy records named capabilities (`implementation`, `review`,
`integration`, `rust`, `typescript`, `large_context`) against stable Agent
registry IDs. Built-in defaults are shipped as product data; project overrides
are Git-trackable SpecOS configuration and cannot enable an unavailable Agent.

The optional project file is `.specos/routing.yaml`:

```yaml
version: 1
agents:
  - registryId: codex
    capabilities: [implementation, review, rust, typescript, large_context]
    preference: 100
    fallbacks: [claude-code]
taskKinds:
  integration:
    requiredCapabilities: [integration]
```

Registry IDs and fallback targets must exist, capabilities use the shipped
stable vocabulary, fallback graphs are acyclic, preferences are `0..100`, and
the file is limited to 64 agents / 128 KiB. Missing file means shipped defaults
plus existing task/folder selections.

`work_task_route_decision` primary key `(task_id, run_seq)` stores policy
version/hash, request summary, candidate facts, chosen Agent/mode/model/provider
IDs, fallback order, decision reason codes, and creation time. Provider keys,
environment, and raw diagnostics are excluded.

Stable reason codes include `explicit_task`, `folder_default`,
`capability_match`, `task_kind_match`, `context_supported`,
`project_preference`, `disabled`, `not_installed`, `missing_capability`,
`context_too_large`, `provider_unavailable`, and `permission_denied`.

## 4. Routing And Fallback Rules

1. Build the candidate catalog once, validate it, and persist the decision
   before spawning an Agent.
2. Explicit task/folder candidates fail clearly when unavailable; they are not
   silently replaced unless the same policy explicitly contains fallbacks.
3. Automatic score order is stable: disqualification, required-capability
   coverage, task-kind fit, context fit, project preference, user agent order,
   then registry ID as final tie-breaker.
4. Fallback is allowed only before any prompt/message ID is created and only
   for launch-unavailable, model-unavailable, or provider-unavailable failures.
5. Permission denial, invalid project config, context overflow, Agent refusal,
   or post-prompt failure does not automatically cross to another Agent.
6. Every fallback attempt is appended to WorkTask events and run trace.
7. Manual override creates a new run decision; historical decisions are never
   rewritten.

## 5. Commands, Errors, And UI

```text
work_task_route_preview(task_id, draft_override?) -> RouteDecision
work_task_route_get(task_id, run_seq) -> RouteDecision
```

| Error key | Condition |
|---|---|
| `workTask.route.noCandidate` | No candidate satisfies required capabilities/availability. |
| `workTask.route.explicitUnavailable` | Explicit Agent/model cannot launch. |
| `workTask.route.invalidPolicy` | Capability or fallback config is invalid/cyclic. |
| `workTask.route.changed` | Candidate facts changed between decision and spawn. |

The route view covers loading, explicit, automatic, fallback-used,
no-candidate, stale catalog, invalid policy, and transport failure. A preview is
advisory; the persisted launch decision is authoritative.

## 6. Client Interaction Contract

This Feature implements route preview before Start and Run Inspector `Route`.

- Task Detail Overview/Plan shows the current explicit override or `Automatic`.
  `Preview route` calls `work_task_route_preview` and is advisory.
- Preview shows chosen Agent/mode/model/provider, decision hash/policy version,
  explicit-vs-automatic origin, ordered fallbacks, and a table of every
  candidate with score, qualifications, disqualifications, and reason codes.
- Start confirmation repeats the preview summary, but launch may reject changed
  candidate facts. `workTask.route.changed` keeps the dialog open and offers a
  fresh preview; it never silently starts a different route.
- Run Inspector shows the immutable persisted decision and each fallback
  attempt. A persisted decision is visually distinguished from a preview.
- No-candidate and explicit-unavailable views show actionable causes without
  exposing provider keys or raw environment diagnostics.

`src/lib/api.ts` exposes `workTaskRoutePreview` and `workTaskRouteGet`; exact
DTOs live in `src/lib/types.ts`. `route-preview-dialog`, `route-summary`,
`route-candidate-table`, and `route-tab` live under
`src/components/tasks/specos/`.

Required states are loading, explicit, automatic, fallback-used, no candidate,
stale catalog, invalid policy, changed-before-spawn, and transport failure.
Candidate tables become stacked cards on narrow screens and preserve score and
reason semantics for screen readers.

## 7. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-007.AC01` | Explicit task and folder choices retain their current precedence and wire behavior. |
| `BUGRAIL-SPECOS-007.AC02` | Identical request/catalog/policy produces the same scored candidates, reasons, route, and hash. |
| `BUGRAIL-SPECOS-007.AC03` | Disabled, missing, incompatible, oversized-context, or denied candidates are disqualified with stable reasons. |
| `BUGRAIL-SPECOS-007.AC04` | Allowed pre-prompt failures follow only the recorded fallback chain; post-prompt and denied failures require a new run/user action. |
| `BUGRAIL-SPECOS-007.AC05` | Fallback preserves Spec, Context Pack, Worktree, permissions, and gate policy. |
| `BUGRAIL-SPECOS-007.AC06` | Route decisions survive restart and are visible in run trace without secret-bearing fields. |
| `BUGRAIL-SPECOS-007.AC07` | Tauri/Axum and all route UI states are equivalent. |

## 8. Testing And Implementation Order

1. Pure resolver/capability-policy determinism and invalid-config tests.
2. Candidate catalog integration with built-in/custom Agents and providers.
3. WorkTask decision persistence and pre/post-prompt fallback tests.
4. Preview/get transports and route-explain UI tests including stale-preview,
   candidate-table accessibility, and persisted-vs-advisory labeling.
5. Existing Agent settings, ACP spawn, WorkTask, and provider regression suites.
