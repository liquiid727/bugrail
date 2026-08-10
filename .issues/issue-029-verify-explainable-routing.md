---
id: issue-029
title: "Derive and execute verification for explainable routing"
status: draft
kind: verification
type: fullstack
priority: high
sourceSpecId: BUGRAIL-SPECOS-007
sourceSpecVersion: "0.1"
sourceSpecHash: "13da86ab9e3289a51d07a15aad6555e80d02c1300ff0951e64929c8fb18d518b"
requirements: [BUGRAIL-SPECOS-007.R01, BUGRAIL-SPECOS-007.R02, BUGRAIL-SPECOS-007.R03, BUGRAIL-SPECOS-007.R04, BUGRAIL-SPECOS-007.R05, BUGRAIL-SPECOS-007.R06]
dependsOn: [issue-026, issue-027, issue-028]
---

# Derive And Execute Verification For Explainable Routing

## Scope

- Derive exact-version Test Spec coverage for AC01–AC07.
- Test determinism, precedence, invalid policies, all disqualifications, decision
  races, allowed/forbidden fallback, invariant preservation, redaction, and UI.
- Run existing Agent settings, custom registry, ACP spawn, provider, and WorkTask
  regression suites.
- Store normalized route hashes, attempts, and source-bound evidence.

## Acceptance Criteria

- Repeated fixtures prove stable candidate ordering and route hash.
- No post-prompt, denied, stale, or client-forged path crosses Agents silently.
- Spec, Context, Worktree, permissions, and gates remain byte/fact equivalent
  across allowed fallback.
- Every AC has independent passing evidence.
