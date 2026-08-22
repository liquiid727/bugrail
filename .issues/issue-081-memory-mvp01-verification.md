---
id: issue-081
title: "Memory Plugin MVP01 independent verification"
status: verified
kind: verification
sourceSpecId: BUGRAIL-SPECOS-017
sourceSpecVersion: "0.2"
sourceSpecHash: "f62824d31787ff774d962d942ffba0423fe78f28c77fa7b0e2e000d71dacfd58"
requirements: []
dependsOn: [issue-077, issue-078, issue-079, issue-080]
---

# Memory Plugin MVP01 independent verification

## Outcome

Execute the exact Test Spec against local deterministic adapters and a pinned
TencentDB Agent Memory `v2.0.0` fixture, then retain durable evidence for the
capture-to-later-recall path.

## Scope

Verify restart, idempotency, identity isolation, required/optional failure,
privacy, budgets, provenance, UI and Tauri/Axum compatibility. A moving upstream
branch, Proxy injection or frontend-only evidence cannot close this Issue.

This Issue also aggregates the final verification evidence for the pending
baseline verification Issues `issue-047`, `issue-053`, `issue-055` and
`issue-060`: the pinned `v2.0.0+bugrail.1` integration fixture supplies the
real remote health evidence that `issue-055` T02 lacks, and the shared
package/inspector evidence covers the remaining inspector oracles.

### 2026-08-22 execution

- Executed the exact Test Spec T01-T08 against deterministic adapters, the
  fake Gateway and the pinned TencentDB `v2.0.0` + `bugrail.1` fixture
  (image digest recorded). Evidence:
  `tests/results/2026-08-22-specos-017-memory-verification.md`.
- Two defects found and fixed with tests: `parse_hits` now accepts the
  authoritative `data.items` shape; legacy projects without a Memory
  provider are no longer auto-injected one (AC08).
- Aggregated confirmations for `issue-047`, `issue-053`, `issue-055` and
  `issue-060` are recorded in the same evidence document.
- Status: `pending_verification` until the evidence record is accepted.
