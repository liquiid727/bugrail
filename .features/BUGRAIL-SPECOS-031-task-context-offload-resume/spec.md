---
id: BUGRAIL-SPECOS-031
version: "0.1"
title: "Task Context Offload And Resume"
status: draft
changeType: work-task-context-offload
prd: ".prd/prd-memory-operating-layer-roadmap.md"
design: ".features/bugrail-specoos-memory/03-MEMORY-记忆模型召回与上下文.md"
codeBaseline: "2ab6d5cf"
dependsOn: [BUGRAIL-SPECOS-003, BUGRAIL-SPECOS-006, BUGRAIL-SPECOS-009, BUGRAIL-SPECOS-028, BUGRAIL-SPECOS-029]
---

# BUGRAIL-SPECOS-031: Task Context Offload And Resume

## 1. Outcome

Keep long WorkTasks usable by replacing oversized tool/terminal results with
durable bounded references, summaries and a resumable task canvas before they
consume the active Agent context window.

## 2. Requirements

| ID | Requirement |
|---|---|
| `BUGRAIL-SPECOS-031.R01` | Tool, terminal and large-file outputs cross explicit size/token thresholds before offload; original evidence is stored under the task/run scope with content hashes. |
| `BUGRAIL-SPECOS-031.R02` | Agent-facing output receives a stable bounded reference and safe summary, never a silent truncation or untrusted instruction. |
| `BUGRAIL-SPECOS-031.R03` | Step summaries, checkpoints and a versioned task canvas are derived from durable WorkTask/run evidence and remain attributable to source refs. |
| `BUGRAIL-SPECOS-031.R04` | Resume creates a new WorkTask generation and Context Package containing the effective canvas/summaries plus only selected refs. |
| `BUGRAIL-SPECOS-031.R05` | Cleanup, retention and quota policy are scoped, restart-safe and cannot remove evidence referenced by an active or reviewable generation. |
| `BUGRAIL-SPECOS-031.R06` | Task UI shows offload volume, canvas, checkpoints, ref drill-down and resume/degradation states in desktop and server mode. |

## 3. Existing Modules

- Extend WorkTask run evidence, transcript/event capture and Context Package
  preparation; WorkTask remains the resume state machine.
- Reuse provider jobs for summarization/canvas work and existing file access
  confinement for stored refs.
- The Memory Engine may store/query summaries, but it cannot own task status.

## 4. Acceptance Criteria

| ID | Criterion |
|---|---|
| `BUGRAIL-SPECOS-031.AC01` | A 10 MiB tool result stays outside the active prompt while a bounded ref and summary remain usable. |
| `BUGRAIL-SPECOS-031.AC02` | Restart during offload/summarization recovers exactly one artifact/job and preserves its source hash. |
| `BUGRAIL-SPECOS-031.AC03` | Resume after process restart creates a new generation whose package restores task state without replaying the full transcript. |
| `BUGRAIL-SPECOS-031.AC04` | Malicious output remains untrusted data; secrets and excluded paths do not enter summaries or Memory capture. |
| `BUGRAIL-SPECOS-031.AC05` | Retention and quota tests protect active evidence and reclaim only eligible artifacts. |

## 5. Non-Goals

No second task scheduler, editable general-purpose knowledge base, Wiki index or
Skill promotion pipeline is introduced.
