---
id: issue-022
title: "Build bounded repository impact analysis and language adapters"
status: draft
kind: implementation
type: backend
priority: medium
sourceSpecId: BUGRAIL-SPECOS-006
sourceSpecVersion: "0.1"
sourceSpecHash: "ad1ac57268e3fed82c35e8c7f3b57eda1e29fca753eb6b2ad747e328ffe223f5"
requirements: [BUGRAIL-SPECOS-006.R01, BUGRAIL-SPECOS-006.R02, BUGRAIL-SPECOS-006.R03, BUGRAIL-SPECOS-006.R04]
dependsOn: [issue-018]
---

# Build Bounded Repository Impact Analysis And Language Adapters

## Outcome

Explicit seed paths produce a deterministic, explained, cancellable snapshot of
related repository files without claiming a universal semantic graph.

## Scope

- Implement internal `RepositoryImpact.analyze` traversal, scoring, deduplication,
  limits, omissions, exact/heuristic confidence, and snapshot hashing.
- Add TypeScript/JavaScript import extraction and Rust module/use/Cargo adapters.
- Add language-neutral package/test/manifest/instructions/Git-cochange relations.
- Apply Context Pack secret/path/binary/generated/vendor rules before reads.
- Enforce 20 seeds, 2,000 inspected, 200 selected, depth 3, time, and byte limits.

## Acceptance Criteria

- Same revision/seeds/policy produces the same ordered nodes/edges/hash.
- Unsupported languages yield explicit partial snapshots, not guessed semantics.
- Cancellation/truncation/tool absence records stable omissions and returns promptly.
- Exact and heuristic relations never collapse into one confidence value.
- Outside-root, symlink, secret, binary, generated, and vendor content cannot leak.

## Verification

Rust/TS/language-neutral fixture repositories, limit boundaries, cancellation,
security attacks, deterministic ordering, and benchmark tests pass.
