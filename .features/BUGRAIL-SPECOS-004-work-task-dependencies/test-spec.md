---
id: BUGRAIL-SPECOS-004-TEST
version: "0.2"
feature: BUGRAIL-SPECOS-004
featureVersion: "0.2"
status: approved
independentFromImplementation: true
---

# Test Spec: WorkTask Dependencies

## 1. Purpose And Oracle

Verify the externally observable requirements of BUGRAIL-SPECOS-004 without
using implementation completion as evidence. Authoritative oracles are validated
Git configuration, SQLite projections after restart, command-core responses,
existing WorkTask/ACP/Git facts, and rendered UI states.

## 2. Fixtures

- a temporary Git project with normal and symlink/path-escape files;
- migrated SQLite plus a pre-feature legacy fixture;
- deterministic ACP/runtime and HTTP Provider substitutes;
- sequential and concurrent command clients for Tauri/Axum core parity;
- React transport mocks for no-workspace, loading, empty, success, degraded,
  blocked, stale, and error states.

## 3. Scenario Matrix

| ID | Scenario | Branch | Evidence |
|---|---|---|---|
| T01 | acyclic-edge-validation | happy/error/edge as applicable | persisted facts + command/UI assertion |
| T02 | blocked-child-not-claimed | happy/error/edge as applicable | persisted facts + command/UI assertion |
| T03 | parallel-ready-claims | happy/error/edge as applicable | persisted facts + command/UI assertion |
| T04 | parent-failure-reason | happy/error/edge as applicable | persisted facts + command/UI assertion |
| T05 | concurrency-race | happy/error/edge as applicable | persisted facts + command/UI assertion |
| T06 | legacy-task-readiness | happy/error/edge as applicable | persisted facts + command/UI assertion |

## 4. Cross-Cutting Assertions

1. Invalid input commits no partial definition, run, dependency, package,
   activity, handoff, or state transition.
2. Process restart yields the same durable decision and exact generation
   attribution; live events are never the only oracle.
3. Secrets, full environment values, uncapped output, and paths outside the
   project do not appear in DTOs, SQLite, logs, or rendered UI.
4. Existing unprofiled WorkTasks and current ACP/Session/Worktree behavior pass
   regression fixtures.
5. Desktop and standalone-server transports return equivalent typed results and
   errors from the same command core.
6. Frontend interactions are keyboard reachable, responsive, localized, and do
   not derive authoritative readiness/eligibility locally.

## 5. Commands And Evidence Record

Record the exact commit, database migration set, fixture revision, commands,
exit codes, and report paths. Minimum commands are Rust focused tests, cargo
check, frontend unit tests, TypeScript check, and production build. A scenario
is not passed by a screenshot alone; it requires the command/database oracle
specified in the matrix.

