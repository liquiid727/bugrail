---
id: BUGRAIL-SPECOS-012
version: "0.1"
title: "Run Evaluation Projection"
status: draft
changeType: evidence-projection
prd: ".prd/prd-specos-agent-team-context-system.md"
design: "design/specos-control-plane-design.md"
clientDesign: "design/specos-client-interaction-design.md"
codeBaseline: "55545d43"
dependsOn: [BUGRAIL-SPECOS-003, BUGRAIL-SPECOS-011]
---

# BUGRAIL-SPECOS-012: Run Evaluation Projection

## 1. Summary

Project version-bound WorkTask run, gate, route, token, review, and Git evidence
into comparable evaluation facts. Evaluation is read-oriented and evidence
quality is explicit. This Feature does not copy transcripts, add a second event
bus, or let sparse historical scores silently steer routing.

### Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-012.R01` | An eligible settled run produces one idempotent evaluation fact bound to task/run, Spec hash, route policy, and evidence revision. |
| `BUGRAIL-SPECOS-012.R02` | Facts normalize outcome, first-pass, gate/review results, rework, latency, tokens, intervention, and failure category while preserving unknown values. |
| `BUGRAIL-SPECOS-012.R03` | Queries aggregate by Agent, model, task kind, risk, Context policy, route policy, and time range with sample/evidence counts. |
| `BUGRAIL-SPECOS-012.R04` | Stale Spec, missing blocking evidence, pending token sync, waived gates, and legacy-unscoped runs are visibly qualified and excluded from strict cohorts by default. |
| `BUGRAIL-SPECOS-012.R05` | Evaluation views never change WorkTask state, gates, route decisions, memory, or Skills. |

PRD coverage: `P-DC-13`, `P-DC-16`, `P-DC-18`.

## 2. Sources And Deep Module

The `RunEvaluation` module has one interface:

```text
project(run_trace, evidence_policy) -> EvaluationFact
query(filters, cohort_policy, pagination) -> EvaluationReport
```

Sources are existing `work_task_run`, Spec contract/gates, route decision,
Context Pack, WorkTask events, Conversation/token facts, diff/merge facts, and
review evidence. The projection owns normalization so UI and routing callers do
not reproduce failure or cohort rules.

## 3. Evaluation Contract

`work_task_evaluation` uses `(task_id, run_seq)` as primary key and stores source
revision hash, Spec/route/context IDs, task kind, risk, Agent/model/provider,
outcome, first-pass boolean/unknown, blocking-gate counts, waived count, rework
rounds, human interventions, duration, token counters/unknown, diff size,
failure category, evidence quality, and projected time.

Failure categories are stable:

```text
setup | context | routing | agent | permission | tool | build | test | review
| requirement | architecture | integration_conflict | merge | timeout
| provider | canceled_by_user | interrupted | unknown
```

Evidence quality is `strict`, `qualified`, or `insufficient`. Missing values are
`null`; they are never converted to zero or success.

## 4. Projection And Query Rules

1. A projector runs after settlement and may rerun after token/review sync.
   Upsert is allowed only when the source revision hash changes.
2. Strict facts require current Spec hash, complete required gates, terminal
   outcome, and resolved source attribution. Waivers make a fact qualified.
3. Percentages always return numerator, denominator, excluded count, and cohort
   policy. Minimum display sample is 5; comparative recommendation requires 20.
4. User cancellation and infrastructure failure are not scored as equivalent
   to requirement/test failure.
5. Reports are cursor-paginated and time-bounded; default 30 days, maximum one
   year per request.
6. No transcript text, prompt text, file content, provider key, or personal
   identifier is stored in evaluation facts.

## 5. Commands, Errors, And UI

```text
work_task_evaluation_get(task_id, run_seq) -> EvaluationFact
evaluation_report(query) -> EvaluationReport
evaluation_reproject(task_id, run_seq) -> EvaluationFact
```

Reprojection is an explicit user/maintenance action and is idempotent.

| Error key | Condition |
|---|---|
| `evaluation.sourceIncomplete` | Run cannot yet form an evidence fact. |
| `evaluation.invalidCohort` | Filters/range/policy are invalid or too broad. |
| `evaluation.sourceChanged` | Source changed during projection; retryable. |

Views cover empty sample, loading, strict report, qualified warning, insufficient
evidence, pending sync, source changed, and transport failure.

## 6. Client Interaction Contract

This Feature implements Run Inspector `Evaluation` and Tasks `Insights >
Evaluation`.

- Run Inspector shows one fact: outcome, first-pass, evidence quality, failure
  category, gate/review/rework/intervention facts, duration, usage, diff, source
  revision, and explicit unknown/pending fields.
- Insights provides time range, Agent, model, task kind, risk, Context policy,
  route policy, and evidence-quality filters. Default range is 30 days and
  strict cohort.
- Every metric card/table row shows numerator, denominator, excluded count,
  sample size, cohort policy, and range. Unknown and excluded are inspectable,
  never folded into zero.
- Samples below five show facts without a rate; comparisons below twenty show
  `Insufficient sample for recommendation` and no winner styling.
- `Reproject` is available on a run detail only, requires confirmation, and
  refetches by source revision. It cannot modify execution or learning state.
- Reports use cursor pagination and retain filter state during a detail drill-in.

`src/lib/api.ts` exposes `workTaskEvaluationGet`, `evaluationReport`, and
`evaluationReproject`; DTOs live in `src/lib/types.ts`. UI modules are
`evaluation-tab`, `evaluation-insights`, `evaluation-filters`,
`evaluation-metric-table`, and `evidence-quality-summary`.

Required states are empty sample, loading, strict, qualified warning,
insufficient evidence/sample, pending sync, source changed, invalid filters,
and transport failure. Charts are optional; whenever used they must have an
equivalent data table and must not imply ranking below the threshold.

## 7. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-012.AC01` | A complete settled run projects the same idempotent fact after restart/reprojection. |
| `BUGRAIL-SPECOS-012.AC02` | Unknown, waived, stale, legacy, canceled, and infrastructure-failed inputs retain distinct evidence/failure semantics. |
| `BUGRAIL-SPECOS-012.AC03` | Aggregates expose numerator, denominator, exclusions, sample size, time range, and cohort policy. |
| `BUGRAIL-SPECOS-012.AC04` | Token/review sync updates the fact only through a new source revision and never duplicates a run. |
| `BUGRAIL-SPECOS-012.AC05` | Evaluation commands and UI cannot mutate execution, gate, route, memory, or Skill facts. |
| `BUGRAIL-SPECOS-012.AC06` | Sensitive raw content is absent and report queries remain bounded/indexed. |
| `BUGRAIL-SPECOS-012.AC07` | Desktop/server and all report UI states are equivalent. |

## 8. Testing And Implementation Order

1. Pure taxonomy/evidence/cohort projection fixtures.
2. Migration/idempotent revision and token/review resync tests.
3. Aggregation accuracy, unknown/exclusion, bounds, and performance tests.
4. Transport/report UI tests for filter persistence, cohort math display,
   threshold language, table accessibility, pagination, and every state.
5. Full WorkTask run, token usage, route, and gate regression suites.
