# Bugrail

[![Release](https://img.shields.io/github/v/release/liquiid727/bugrail)](https://github.com/liquiid727/bugrail/releases)
[![License](https://img.shields.io/github/license/liquiid727/bugrail)](./LICENSE)

Bugrail is a spec-driven workspace for AI-assisted software development. It
combines a desktop application, a standalone server runtime, agent sessions,
worktrees, and SpecOS delivery artifacts in one repository.

## Documentation

- [SpecOS documentation](./docs/specos/README.md)
- [Product vision](./docs/specos/product-vision.md)
- [Privacy notes](./docs/CLIENT-PRIVACY.md)
- [Repository rules](./rules/README.md)

The repository keeps its existing runtime compatibility contracts, including
the `codeg://` URI scheme, `codeg-events`, `codeg` binaries, and `CODEG_*`
environment variables. These identifiers are implementation compatibility
details and are not project branding.

## Development

Requirements: Node.js, pnpm, and Rust.

```bash
make init
make dev
```

Useful commands:

```bash
make desktop       # Tauri desktop development mode
make test          # Frontend tests
make build         # Static frontend build
pnpm eslint .      # Lint
```

The server runtime can be built or started with:

```bash
pnpm server:build
pnpm server:dev
```

## Releases

Release metadata and all public distribution addresses live in
[`release/manifest.json`](./release/manifest.json). Validate the current state
with `pnpm release:check`; update every application version with
`pnpm release:set -- 0.27.0`.

Production releases are tag-driven through GitHub Actions. The desktop updater
reads the Bugrail `latest.json` manifest, and server images are published at
`ghcr.io/liquiid727/bugrail`.

The reusable release procedure is bundled as the
`bugrail-release-management` Skill under
[`src-tauri/experts/skills/bugrail-release-management/SKILL.md`](./src-tauri/experts/skills/bugrail-release-management/SKILL.md).

See `AGENTS.md` for the repository architecture, compatibility constraints,
and backend verification commands.

## License

Apache-2.0. See [LICENSE](./LICENSE).
