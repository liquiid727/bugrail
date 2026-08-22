---
id: issue-080
title: "Memory Provider operations and Context UI"
status: implemented_pending_verification
kind: implementation
sourceSpecId: BUGRAIL-SPECOS-017
sourceSpecVersion: "0.2"
sourceSpecHash: "f62824d31787ff774d962d942ffba0423fe78f28c77fa7b0e2e000d71dacfd58"
requirements: [BUGRAIL-SPECOS-017.R06, BUGRAIL-SPECOS-017.R07]
dependsOn: [issue-078, issue-079]
---

# Memory Provider operations and Context UI

## Outcome

Expose provider test, delivery list/retry, recall preview and package provenance
through shared command-core, Tauri/Axum and the existing BugRail Context page.

## Scope

Retain last-good data and all required error states. Do not embed or reproduce
the full upstream MemoryPanel asset/Team/ACL administration surface.

## Verification

Cover `T07`, safe rendering from `T06`, all ten locales, responsive/keyboard
behavior and desktop/server transport parity.


### 2026-08-23 status

- Implementation landed in `feat(memory): provider commands and Context
  UI`: shared command-core, Tauri/Axum registration, Memory tab on the
  Context page and all ten locale catalogs, with package detail moved to
  on-demand `specos_context_package_get`.
- T07 transport parity and UI state coverage recorded in
  `tests/results/2026-08-22-specos-017-memory-verification.md`.
