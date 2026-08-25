---
id: issue-095
title: "Task summaries, canvas and generation-safe resume"
status: planned
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-031
sourceSpecVersion: "0.1"
sourceSpecHash: "537c3e75a4432ce8b56a87656198a990ba7b98eed55eb1ec9f1b8fb0e317ddf4"
requirements: [BUGRAIL-SPECOS-031.R02, BUGRAIL-SPECOS-031.R03, BUGRAIL-SPECOS-031.R04]
dependsOn: [issue-094]
---

# Task summaries, canvas and generation-safe resume

## Outcome

Build attributable step summaries/checkpoints/task canvas and resume them into
a new WorkTask generation and immutable Context Package.

## Scope

WorkTask remains the state machine; summarization jobs cannot mutate a settled
generation or replay the full transcript into the prompt.

## Verification

Cover `BUGRAIL-SPECOS-031.T02-T04` with process restart and source changes.
