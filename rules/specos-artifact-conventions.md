# SpecOS Artifact Conventions

## Purpose

Keep BugRail product intent, design decisions, delivery specs, Issues, and
verification evidence in one self-contained chain. These rules apply only to
the `bugrail/` repository.

## Source Of Truth

`.specos/manifest.yaml` is the only declaration of artifact directories.

```text
docs/specos/                    product vision, indexes, CodeG module mapping
design/                         stable system design
design/adr/                     one decision per ADR
.prd/                           product requirement drafts
.features/roadmap.md            feature order and dependencies
.features/<FEATURE-ID>-<slug>/  Feature Spec and exact-version Test Spec
.issues/                        implementation and verification Issues
current/                        active handoff and evidence posture
tests/                          executable verification assets and results
```

`docs/specos/rfcs/` contains design exploration only. An RFC cannot authorize
implementation and cannot satisfy a review or release gate.

## Naming

| Artifact | Pattern | Example |
|---|---|---|
| Feature | `BUGRAIL-SPECOS-NNN` | `BUGRAIL-SPECOS-001` |
| Feature directory | `<FEATURE-ID>-<slug>` | `BUGRAIL-SPECOS-001-work-task-quality` |
| Requirement | `<FEATURE-ID>.RNN` | `BUGRAIL-SPECOS-001.R01` |
| Acceptance criterion | `<FEATURE-ID>.ACNN` | `BUGRAIL-SPECOS-001.AC01` |
| Test case | `<FEATURE-ID>.TNN` | `BUGRAIL-SPECOS-001.T01` |
| Local Issue file | `issue-NNN-<slug>.md` | `issue-001-spec-contract-schema.md` |
| ADR | `ADR-NNN-<slug>.md` | `ADR-001-embed-specos-in-work-task.md` |

Identifiers are namespaced by artifact kind. Numeric suffixes do not imply a
single global counter and are never inferred from examples.

Inside one Feature/Test Spec, dense scenario tables may show `R01`, `AC01`, or
`T01` as local shorthand. Front matter, Issues, normalized evidence, review
records, and cross-document references always use the full identifier.

## Version Binding

- Every Feature Spec declares `id`, `version`, `status`, design source, and
  requirement identifiers.
- Every Test Spec records the exact source Spec ID, version, and SHA-256.
- Every Issue records the same source Spec ID, version, and SHA-256.
- Changing behavior creates a new Spec version. Existing Test Specs and Issues
  become stale until they are regenerated or explicitly reviewed.
- Markdown is the human and machine-reviewed source for delivery specs. Do not
  maintain a second YAML rendering of the same normative content.

## Relationship To Existing CodeG Code

- A Feature Spec must list existing modules it reuses, modules it extends, and
  any behavior it replaces.
- Existing CodeG wire identifiers such as command names, URI schemes, database
  filenames, and `CODEG_*` variables are compatibility contracts. They change
  only under a compatibility Feature Spec with rollout and rollback behavior.
- New modules are justified only when the behavior cannot be placed behind an
  existing deep module interface.
- A proposed external seam needs at least two justified adapters. Test-only
  substitution may remain an internal seam.

## Delivery Gate

The canonical chain is:

```text
product vision -> design/ADR -> PRD -> approved Feature Spec
  -> implementation Issues
  -> exact-version Test Spec -> verification Issues
  -> results -> review -> release
```

Draft documents and local command output are not release evidence.
