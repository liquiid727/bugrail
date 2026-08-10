# SpecOS Verification Assets

Feature-local Test Specs live beside their Feature Specs under `.features/`.
This directory contains executable cross-surface plans or fixtures that do not
belong beside Rust/TypeScript unit tests, plus normalized result references.

- Existing implementation tests remain colocated under `src/`, `src-tauri/src/`,
  and `src-tauri/tests/`.
- Normalized independent evidence is written under `tests/results/` and must
  reference an exact Feature/Test Spec version and hash.

