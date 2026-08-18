---
id: BUGRAIL-SPECOS-014
version: "0.1"
title: "Skill Experience Lifecycle"
status: draft
changeType: governed-learning
prd: ".prd/prd-specos-agent-team-context-system.md"
design: "design/specos-control-plane-design.md"
clientDesign: "design/specos-client-interaction-design.md"
codeBaseline: "55545d43"
dependsOn: [BUGRAIL-SPECOS-012, BUGRAIL-SPECOS-013]
---

# BUGRAIL-SPECOS-014: Skill Experience Lifecycle

## 1. Summary

Add a governed candidate lifecycle in front of BugRail's existing ACP Skill
read/save/delete capabilities. Repeated evidence can propose a Skill draft, but
validation and explicit user approval are required before it becomes available
to an Agent. Activation and rollback reuse existing Skill storage and refresh
behavior rather than creating a parallel Skill runtime.

### Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-014.R01` | A Skill candidate requires repeated independent eligible runs sharing a normalized task pattern and successful behavior. |
| `BUGRAIL-SPECOS-014.R02` | Candidate draft, sources, supported Agents, scope, risks, validation plan, version, and conflicts are inspectable before validation. |
| `BUGRAIL-SPECOS-014.R03` | Validation compares the candidate against a recorded baseline on fixed fixtures or approved replay inputs and stores evidence. |
| `BUGRAIL-SPECOS-014.R04` | Only a human-approved, passing candidate can be written through existing ACP Skill save behavior and activated. |
| `BUGRAIL-SPECOS-014.R05` | Activation is versioned and reversible; failed post-activation evidence can mark a version degraded but cannot silently rewrite it. |

PRD coverage: `P-DC-15`, `P-DC-12`, `P-DC-16`, `P-DC-18`.

## 2. Existing Module Placement

| Existing module | Role |
|---|---|
| `acp_list/read/save/delete_agent_skill` | Authoritative Skill file operations and Agent-specific path conventions. |
| Agent/custom registry Skill declarations | Supported Agents and shared/private Skill behavior. |
| WorkTask run/evaluation facts | Candidate and validation evidence. |
| Project memory candidates | Pattern/failure inputs; no automatic conversion. |
| Agent settings/refresh path | Make an approved version visible using current runtime behavior. |

`SkillCandidate` is governance around this existing module. It is not a new
Skill execution adapter.

## 3. Candidate And Version Contract

`skill_candidate` stores UUID, project/folder, normalized pattern key, title,
draft content, target Agent registry IDs, scope, source run refs, evidence
revision, risk flags, status, current candidate version, actor/timestamps, and
active Skill path/version when promoted.

`skill_candidate_validation` is append-only and stores candidate/version,
baseline and candidate fixture refs, commands/policy, outcomes, regressions,
review actor, and evidence hash.

Lifecycle:

```text
proposed -> validating -> validated | validation_failed
validated -> approved -> active
active -> degraded | deprecated
degraded -> active (new validation) | deprecated
```

Default proposal threshold is three successful, Spec-bound runs from at least
two distinct WorkTasks. Threshold satisfaction proposes only; it never validates
or approves.

## 4. Validation, Activation, And Rollback

1. Candidate grouping uses exact normalized pattern/scope keys. Similarity
   models do not merge candidates in the first implementation.
2. Draft content is rendered and diffed as untrusted text. It cannot include
   secrets, absolute local paths, hidden credential files, or instructions that
   bypass permissions/gates.
3. Validation runs in a disposable Worktree/fixture with declared commands,
   time/resource limits, no release/push, and the same permission policy as a
   normal task. Network is denied unless the validation plan explicitly allows
   it and the user approves.
4. Passing requires no baseline regression plus every declared validation
   criterion. The validating Agent cannot approve its own result.
5. Activation previews the exact Skill path/content, revalidates candidate and
   target hashes, calls existing Skill save behavior, and records the previous
   version for rollback.
6. Rollback restores the previous recorded version through the same save path.
   Deleting an active Skill is not used as rollback when a previous version exists.
7. Runtime quality changes may mark a version degraded and notify the user;
   automatic rewrite, revalidation, rollback, or deletion is forbidden.

## 5. Commands, Errors, And UI

```text
skill_candidate_list(project, filters, cursor) -> CandidatePage
skill_candidate_get(id) -> CandidateDetail
skill_candidate_validate(id, version, plan) -> ValidationRun
skill_candidate_approve(id, version, expected_hash) -> Candidate
skill_candidate_activate(id, version, expected_hash) -> ActiveSkill
skill_candidate_rollback(id, target_version) -> ActiveSkill
```

| Error key | Condition |
|---|---|
| `skillCandidate.insufficientEvidence` | Proposal/validation lacks independent evidence. |
| `skillCandidate.unsafeContent` | Draft violates path, secret, permission, or gate policy. |
| `skillCandidate.validationFailed` | Required validation or baseline comparison fails. |
| `skillCandidate.approvalRequired` | Activation attempted without current human approval. |
| `skillCandidate.sourceChanged` | Candidate, evidence, or target Skill changed after review. |

UI covers empty, proposed, insufficient evidence, validating, failed, validated,
approval required, conflict, active, degraded, rollback preview, deprecated, and
transport failure.

## 6. Client Interaction Contract

This Feature implements Tasks `Insights > Skill Candidates` as a governed
lifecycle, not a one-click generator.

- List filters by lifecycle, target Agent, scope, risk, validation state, and
  degradation. Rows show evidence threshold, independent task/run counts,
  candidate version, risk flags, validation outcome, and active version.
- Detail shows draft content as untrusted text, exact source runs, supported
  Agents, scope, risk findings, conflicts, validation plan, append-only
  validation history, approval actor/time, and active/prior versions.
- `Validate` opens a plan review dialog covering fixtures/replay refs, commands,
  limits, permissions, network choice, baseline, and criteria. Progress is
  refetched from durable validation facts; closing the dialog does not cancel.
- `Approve` is enabled only for the exact passing candidate version and is
  separate from `Activate`. Activation previews exact Skill path/content/hash
  and the preserved previous version before confirmation.
- Degraded state never auto-rolls back. `Rollback` previews the target prior
  version and requires explicit confirmation; deprecated remains inspectable.
- Stale/source-changed responses retain the user's context and require
  refetch/re-review rather than replaying the mutation.

`src/lib/api.ts` exposes list/get/validate/approve/activate/rollback methods;
DTOs live in `src/lib/types.ts`. `skill-candidate-list`,
`skill-candidate-detail`, `skill-validation-dialog`,
`skill-activation-dialog`, and `skill-rollback-dialog` live under the SpecOS
task component directory.

Required states are empty, proposed, insufficient evidence, validating,
validation failed, validated, approval required, conflict, active, degraded,
rollback preview, deprecated, source changed, and transport failure. Long draft
and validation content uses capped scroll regions, safe text rendering, and
complete keyboard/focus management.

## 7. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-014.AC01` | One run or repeated runs from one WorkTask cannot meet the default proposal threshold. |
| `BUGRAIL-SPECOS-014.AC02` | Eligible independent runs create an idempotent candidate with complete source/risk/scope facts. |
| `BUGRAIL-SPECOS-014.AC03` | Validation is isolated, bounded, baseline-compared, version-bound, and independently reviewed. |
| `BUGRAIL-SPECOS-014.AC04` | Unsafe, failing, stale, unapproved, or conflicting candidates cannot activate through any transport. |
| `BUGRAIL-SPECOS-014.AC05` | Approved activation uses existing Skill storage/refresh and preserves the previous version. |
| `BUGRAIL-SPECOS-014.AC06` | Rollback restores the selected prior version and records an auditable event. |
| `BUGRAIL-SPECOS-014.AC07` | Degradation never silently changes active Skill content or project policy. |
| `BUGRAIL-SPECOS-014.AC08` | Desktop/server and all lifecycle UI states are equivalent. |

## 8. Testing And Implementation Order

1. Candidate threshold/grouping/lifecycle and unsafe-content tests.
2. Candidate/validation migration and version-binding tests.
3. Disposable Worktree validation harness and baseline comparison fixtures.
4. Existing ACP Skill save/refresh/rollback integration tests.
5. Transport and governed-lifecycle UI tests covering separated approval and
   activation, validation progress, stale version, rollback preview, keyboard
   flow, and every lifecycle state.
6. Security review of permission, network, path, secret, and self-approval bypasses.
