---
id: issue-018
title: "Build the deterministic and secure Context Pack compiler"
status: draft
kind: implementation
type: backend
priority: high
sourceSpecId: BUGRAIL-SPECOS-005
sourceSpecVersion: "0.1"
sourceSpecHash: "1491551bf38d5e1a56a932986c45604f97a5325ff41dbf05634a4000876993c2"
requirements: [BUGRAIL-SPECOS-005.R01, BUGRAIL-SPECOS-005.R02, BUGRAIL-SPECOS-005.R03]
dependsOn: [issue-002, issue-014]
---

# Build The Deterministic And Secure Context Pack Compiler

## Outcome

Identical task/repository facts and policy compile byte-identical ordered context,
with every inclusion/exclusion explained and required items never silently dropped.

## Scope

- Implement the private `compile(ContextRequest, FileReader)` module.
- Collect Spec/AC, prompt, explicit refs, project instructions, handoffs, and
  retry facts in the declared priority order.
- Enforce stable item/byte/token-estimate budgets and pack hashing.
- Canonicalize paths and reject symlink escape, secrets, Git internals, binary,
  invalid encoding, device files, size limits, and read/hash races.
- Provide local filesystem fixtures through the internal `FileReader` seam.

## Acceptance Criteria

- Same input produces identical item order, decisions, bytes, and SHA-256.
- Required missing/over-budget/stale items return typed blocking errors.
- Optional overflow is excluded in stable priority/path order with reasons.
- Excluded items store no content; included content stays within caps.
- No repository-wide scan, embedding, network access, or impact inference occurs.

## Verification

Golden determinism, budget boundaries, filesystem races, symlink/path attacks,
secret/binary exclusions, cancellation, and fuzz/property fixtures pass.
