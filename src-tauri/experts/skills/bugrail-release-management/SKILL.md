---
name: bugrail-release-management
description: Use when changing Bugrail versions, preparing a release tag, diagnosing updater or CI/CD publication failures, publishing the GHCR image, deploying production, or rolling back a release.
---

# Bugrail Release Management

Use this skill for the complete Bugrail release path. The release metadata file
and the release CLI are authoritative; do not hand-edit only one version or URL
field.

## Source Of Truth

Read `release/manifest.json` first. It owns the product version, repository,
release URLs, updater manifest, download base, GHCR image, and compatibility
artifact names. Validate it before any mutation:

```bash
pnpm release:check
```

The runtime compatibility names remain `codeg`, `codeg-server`, `codeg-mcp`,
`codeg://`, `codeg-events`, and `CODEG_*`. Do not rename them as part of a
release operation.

## Version Changes

Use the CLI, which updates `release/manifest.json`, `package.json`,
`src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json`:

```bash
pnpm release:show
pnpm release:set -- 0.27.0
pnpm release:check
pnpm test:release
```

Use semver without the `v` prefix. Release tags are generated with:

```bash
pnpm release:tag
```

## Production Release

1. Confirm the worktree is clean and the version is intentional.
2. Run `pnpm release:check`, `pnpm eslint .`, `pnpm test`, `pnpm build`, and the applicable Rust checks.
3. Create and push the exact tag printed by `pnpm release:tag`; never move an existing tag.
4. Monitor `.github/workflows/release.yml`. It must create `Bugrail vX.Y.Z`, build desktop/server artifacts, publish `latest.json`, and push `ghcr.io/liquiid727/bugrail:<version>` plus `:latest`.
5. Verify the GitHub release assets, updater manifest, signatures/checksums, GHCR multi-architecture manifest, and server `/health`/`--version` before declaring success.

Do not publish manually around a failed gate. Fix the source or workflow and
rerun the tag workflow through the normal draft-release path.

## Deployment And Rollback

For Docker production, pull the versioned GHCR tag, recreate the service with
the persistent `/data` volume, then verify health and version. Keep the previous
image tag available for rollback. For server self-update, use the application's
rollback action only after confirming the new process is unhealthy; preserve
the update marker and logs until the rollback is verified.

## Hard Safety Boundaries

Require explicit approval before rotating signing keys, deleting releases,
force-moving tags, changing repository/registry ownership, changing updater
endpoints, or renaming compatibility identifiers. Never print signing private
keys or tokens, and never treat a successful upload as proof that clients can
install the release.
