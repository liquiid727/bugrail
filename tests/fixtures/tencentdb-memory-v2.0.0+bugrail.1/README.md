# Pinned TencentDB Agent Memory fixture: `v2.0.0+bugrail.1`

BUGRAIL-SPECOS-017 T08 integration fixture. Upstream base:
`TencentCloud/TencentDB-Agent-Memory@v2.0.0` — commit
`0aff21a2d9f2b8a0354aaa80a2e586aab4054562` (tag `v2.0.0` points at this
commit, verified via the GitHub git refs API).

Contents:

- `bugrail.1.patch` — minimal patch (sha256
  `4a2d4b514aa9807cb87e83718bb34e7279ac52a7f1fef92a74639c8b63fd2d16`):
  `conversation/add` honors caller message `id` (bounded `[\w:.-]{1,64}`)
  and upserts by it — replay of a known id set adds no L0 rows and returns
  the identical accepted ids — plus `/health` reports `v2.0.0+bugrail.1`.
  Full contract: `docs/specos/upstream/tencentdb-agent-memory-patch-contract.md`.
- `tdai-gateway.yaml` — standalone loopback config (sqlite backend, BM25,
  no external services; the LLM points at the local mock below).
- `mock-llm.mjs` — offline OpenAI-compatible mock returning a deterministic
  L1 extraction whose content embeds the `T8TOK-*` token from the captured
  conversation so per-team isolation stays observable.

Evidence run (see `tests/results/2026-08-22-specos-017-memory-verification.md`):

```bash
# upstream source: codeload tarball of the pinned commit (github.com clone
# unavailable from the build network; tarball + GitHub API tag->commit
# verification used instead), then:
git apply is not needed — the Docker build below uses a tree with the
patch applied; reproduce with:
  curl -L -o v2.0.0.tar.gz \
    https://codeload.github.com/TencentCloud/TencentDB-Agent-Memory/tar.gz/refs/tags/v2.0.0
  tar xzf v2.0.0.tar.gz && cd TencentDB-Agent-Memory-*
  git init . && git add -A && git commit -m baseline
  git am < .../bugrail.1.patch

# image (the pinned `# syntax=docker/dockerfile:1` line is dropped for the
# offline build; no other Dockerfile change):
docker build -f Dockerfile.bugrail -t bugrail-t08-memory:v2.0.0-bugrail.1 .

# fixture:
node .../mock-llm.mjs &                       # 127.0.0.1:18100
docker run -d --name bugrail-t08-gw -p 127.0.0.1:18420:8420 \
  -v .../tdai-gateway.yaml:/data/config/tdai-gateway.yaml:ro \
  -v bugrail-t08-data:/data \
  -e TDAI_GATEWAY_API_KEY=t08-secret -e TDAI_LLM_API_KEY=dummy \
  bugrail-t08-memory:v2.0.0-bugrail.1

# T08:
cd src-tauri && BUGRAIL_T08_URL=http://127.0.0.1:18420 \
  BUGRAIL_T08_SECRET=t08-secret BUGRAIL_T08_SERVICE_ID=t08-service \
  BUGRAIL_T08_USER_ID=t08-user \
  cargo test --features test-utils --test memory_pinned_gateway -- --ignored
```

Image digest (local build, arm64):
`sha256:969a78b925495d62498ac4e3b9780b7861b110d640adf2d03e6e7ff52977f784`.
The mock LLM and the bearer secret are fixture-only values on loopback
interfaces; they are not credentials.
