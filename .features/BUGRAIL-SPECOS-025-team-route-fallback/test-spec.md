---
id: BUGRAIL-SPECOS-025-TEST
version: "0.1"
status: draft
sourceSpecId: BUGRAIL-SPECOS-025
sourceSpecVersion: "0.1"
sourceSpecHash: "ba7bdbb3dca53a0e86acc5d09a01e98b391d98e68c04f5bc6af93ea31391c65c"
independentFromImplementation: true
---

# Test Spec: Team Provider And Model Route Fallback

## Test Cases

| ID | Covers | Scenario | Durable oracle |
|---|---|---|---|
| `BUGRAIL-SPECOS-025.T01` | R01-R03 | Provider unavailable with one allowed fallback. | One new WorkTask generation with old/new route and trigger. |
| `BUGRAIL-SPECOS-025.T02` | R02 | Produce semantic, test, review, permission and budget failures. | No silent fallback unless exact policy allows the class. |
| `BUGRAIL-SPECOS-025.T03` | R04 | Candidate route exceeds permission or budget. | Blocked before ACP dispatch with both decisions recorded. |
| `BUGRAIL-SPECOS-025.T04` | R05 | Exhaust, disable or invalidate all candidates. | Original error retained and actionable final disposition. |
| `BUGRAIL-SPECOS-025.T05` | R03,R06 | Feature disabled with legacy fallback IDs configured. | IDs remain inert; legacy route unchanged. |
| `BUGRAIL-SPECOS-025.T06` | R01-R06 | Restart during route transition. | No duplicate candidate use; route history reconstructs exactly. |

## Evidence

Use fake provider/ACP adapters, classified error fixtures, WorkTask generation
assertions and permission/budget integration tests. Paid providers are excluded
from deterministic CI.

