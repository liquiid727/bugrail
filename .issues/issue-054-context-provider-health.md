---
id: issue-054
title: "Context Provider bootstrap and health boundary"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-007
sourceSpecVersion: "0.2"
sourceSpecHash: "fc10e29a5c2be849a1573875aee7b3fb73f0292dcbfb5e23623c88318f6d669f"
requirements: [BUGRAIL-SPECOS-007.R01]
dependsOn: [issue-052]
---

# Context Provider bootstrap and health boundary

## Outcome

Add project Provider definitions and normalize local/Tencent-compatible health without coupling Agents to provider APIs.

## Scope

Use credential references, bounded timeout, required fail-closed and optional degraded activity; remote retrieval remains out of scope.

## Acceptance And Verification

- Preserve the existing WorkTask, ACP, Session, Worktree, Git and transport
  invariants named by the source Feature Spec.
- Cover happy, error, edge, restart/concurrency, security and legacy behavior.
- For frontend scope, cover no-workspace, loading, empty, success, degraded or
  blocked, stale and transport-error states without deriving backend authority.
- Record exact commands and durable evidence before changing this Issue to
  verified; implementation status alone does not satisfy the source Test Spec.

