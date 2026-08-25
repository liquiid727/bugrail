---
id: BUGRAIL-SPECOS-034
version: "0.1"
title: "Independent Skill Evolution Plugin"
status: draft
changeType: skill-lifecycle-plugin
prd: ".prd/prd-memory-operating-layer-roadmap.md"
design: ".features/bugrail-specoos-memory/04-KNOWLEDGE-SKILL-WIKI-CODEGRAPH.md"
codeBaseline: "2ab6d5cf"
dependsOn: [BUGRAIL-SPECOS-003, BUGRAIL-SPECOS-009, BUGRAIL-SPECOS-028]
---

# BUGRAIL-SPECOS-034: Independent Skill Evolution Plugin

## 1. Outcome

Turn repeated, evidence-backed successful WorkTask traces into reviewable,
versioned Skills without treating Memory atoms as executable instructions or
allowing an Agent to self-publish unverified behavior.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-034.R01` | Extend existing custom Skills with a versioned schema for trigger, procedure, resources, validation, recovery, scope and provenance. |
| `BUGRAIL-SPECOS-034.R02` | Candidate discovery requires repeated attributable traces and records scoring, similar-Skill matches and source evidence. |
| `BUGRAIL-SPECOS-034.R03` | Candidate states are draft, validating, review, published, disabled or rejected with generation-safe transitions and durable jobs. |
| `BUGRAIL-SPECOS-034.R04` | Validation runs in a constrained environment and cannot publish; high-risk or unverifiable candidates require human approval. |
| `BUGRAIL-SPECOS-034.R05` | Skill routing selects bounded Top-K published versions and records why each version entered the immutable Context Package. |
| `BUGRAIL-SPECOS-034.R06` | Skills reference current Wiki/CodeGraph assets by `AssetRef`; source snapshots are not copied into the Skill body. |
| `BUGRAIL-SPECOS-034.R07` | UI supports candidate inbox, diff, evidence, validation, publish, disable and rollback with transport parity. |

## 3. Existing Modules

- Extend `commands/custom_skills`, existing Skill files/settings, WorkTask run
  evidence, Context Packages and provider jobs.
- Reuse existing Agent/permission execution paths for validation.
- Skill Evolution remains independent from MemoryProvider.

## 4. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-034.AC01` | One successful trace cannot auto-publish; repeated matching traces create one deduplicated candidate with source links. |
| `BUGRAIL-SPECOS-034.AC02` | Validation failure preserves evidence and leaves the published version unchanged. |
| `BUGRAIL-SPECOS-034.AC03` | Publish/disable/rollback is audited, generation-safe and reflected in later Context Packages. |
| `BUGRAIL-SPECOS-034.AC04` | Routing explains trigger/version/scope and never selects rejected, disabled or unauthorized Skills. |
| `BUGRAIL-SPECOS-034.AC05` | Prompt-injection, secret and malicious-resource fixtures cannot become published procedure content. |

## 5. Non-Goals

No autonomous high-risk publication, model fine-tuning, Memory conflict policy
or Wiki/CodeGraph ownership is included.
