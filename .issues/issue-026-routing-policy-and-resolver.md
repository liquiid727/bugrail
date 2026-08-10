---
id: issue-026
title: "Implement deterministic Agent and model route resolution"
status: draft
kind: implementation
type: backend
priority: high
sourceSpecId: BUGRAIL-SPECOS-007
sourceSpecVersion: "0.1"
sourceSpecHash: "13da86ab9e3289a51d07a15aad6555e80d02c1300ff0951e64929c8fb18d518b"
requirements: [BUGRAIL-SPECOS-007.R01, BUGRAIL-SPECOS-007.R02, BUGRAIL-SPECOS-007.R03, BUGRAIL-SPECOS-007.R04]
dependsOn: [issue-019]
---

# Implement Deterministic Agent And Model Route Resolution

## Outcome

One deep resolver turns existing registries/settings/providers and optional
project policy into an explained, hashed, deterministic route decision.

## Scope

- Load/validate shipped capability data and optional `.specos/routing.yaml`.
- Build candidate catalog from built-in/custom Agents, settings, model options,
  provider health, task needs/kind/risk, and Context estimate.
- Implement explicit-task, folder, then automatic precedence and stable scoring.
- Validate registry IDs, capabilities, limits, and acyclic fallback graph.
- Emit stable qualifications/disqualifications/reason codes without secrets.

## Acceptance Criteria

- Identical request/catalog/policy yields identical candidates/order/reasons/hash.
- Explicit selections are never silently replaced outside their declared fallback.
- Disabled, missing, incompatible, context-oversized, provider-unavailable, and
  denied candidates are explicitly disqualified.
- Stable tie-break order ends with registry ID.
- Invalid/oversized/cyclic policy fails before any Agent spawn.

## Verification

Golden resolver fixtures, invalid policies, precedence, ties, catalog changes,
secret redaction, and property tests pass.
