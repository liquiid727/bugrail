# Normalized Results

Store normalized independent verification results here. Raw build/test output
may live in CI or another declared artifact store, but each normalized record
must retain a stable reference to it.

| Record | Features | Decision |
|---|---|---|
| `2026-08-13-specos-002-009-verification.md` | `002-009` issues `045/047/049/051/053/055/058/060` | not verified / QA blocked |
| `2026-08-21-specos-001-approval-review.md` | `001` approval review | not approved; independent verification still pending |
| `2026-08-21-specos-005-verification.md` | `005` T01-T06 core evidence | command/SQLite/Git/UI slice passes; independent verification pending |
| `2026-08-23-specos-001-verification.md` | `001` issues `001-005` | not verified / blocked (T17/T20, transport/UI evidence, repository gates) |
| `2026-08-23-specos-001-gap-evidence/README.md` | `001` issues `001-005` | T17/T20/T27 fixtures added, matrix green with logs; still not verified (T28-T31, migration fixture, independent run) |
