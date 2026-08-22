import assert from "node:assert/strict"
import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises"
import os from "node:os"
import path from "node:path"
import test from "node:test"

import {
  normalizeVersion,
  tagForVersion,
  updateVersionFiles,
  validateReleaseData,
} from "./release.mjs"

const manifest = {
  product: {
    name: "Bugrail",
    slug: "bugrail",
    repository: "liquiid727/bugrail",
    bundleIdentifier: "io.liquiid.bugrail",
  },
  version: "0.26.1",
  release: {
    tagPrefix: "v",
    releaseName: "Bugrail",
    repositoryUrl: "https://github.com/liquiid727/bugrail",
    releasesUrl: "https://github.com/liquiid727/bugrail/releases",
    latestReleaseUrl: "https://github.com/liquiid727/bugrail/releases/latest",
    updaterManifestUrl:
      "https://github.com/liquiid727/bugrail/releases/latest/download/latest.json",
    downloadBase:
      "https://github.com/liquiid727/bugrail/releases/latest/download",
  },
  container: {
    registry: "ghcr.io",
    image: "ghcr.io/liquiid727/bugrail",
  },
  compatibilityArtifacts: { server: "codeg-server", mcp: "codeg-mcp" },
}

const versions = {
  manifest: "0.26.1",
  packageJson: "0.26.1",
  cargoToml: "0.26.1",
  cargoLock: "0.26.1",
  tauri: "0.26.1",
}

const runtime = {
  rustRepository: "liquiid727/bugrail",
  rustUpdaterManifestUrl:
    "https://github.com/liquiid727/bugrail/releases/latest/download/latest.json",
  rustDownloadBase:
    "https://github.com/liquiid727/bugrail/releases/latest/download",
  tauriProductName: "Bugrail",
  tauriIdentifier: "io.liquiid.bugrail",
  tauriUpdaterManifestUrl:
    "https://github.com/liquiid727/bugrail/releases/latest/download/latest.json",
}

test("accepts the canonical release metadata", () => {
  assert.deepEqual(validateReleaseData(manifest, versions, runtime), [])
})

test("rejects stale upstream or mismatched release addresses", () => {
  const broken = structuredClone(manifest)
  broken.release.updaterManifestUrl =
    "https://github.com/xintaofei/codeg/releases/latest/download/latest.json"
  broken.container.image = "xintaofei/codeg"
  const errors = validateReleaseData(broken, versions, runtime)
  assert.match(errors.join("\n"), /release\.updaterManifestUrl/)
  assert.match(errors.join("\n"), /container\.image/)
})

test("normalizes versions and tags", () => {
  assert.equal(normalizeVersion("v1.2.3-rc.1"), "1.2.3-rc.1")
  assert.equal(tagForVersion("1.2.3"), "v1.2.3")
  assert.throws(() => normalizeVersion("latest"), /invalid semver/)
})

test("updates every application version source", async () => {
  const base = await mkdtemp(path.join(os.tmpdir(), "bugrail-release-"))
  await mkdir(path.join(base, "release"))
  await mkdir(path.join(base, "src-tauri"))
  await writeFile(
    path.join(base, "release/manifest.json"),
    `${JSON.stringify(manifest, null, 2)}\n`
  )
  await writeFile(
    path.join(base, "package.json"),
    '{\n  "version": "0.26.1"\n}\n'
  )
  await writeFile(
    path.join(base, "src-tauri/Cargo.toml"),
    '[package]\nname = "bugrail"\nversion = "0.26.1"\n\n[dependencies]\n'
  )
  await writeFile(
    path.join(base, "src-tauri/Cargo.lock"),
    '[[package]]\nname = "bugrail"\nversion = "0.26.1"\n\n[[package]]\nname = "other"\nversion = "1.0.0"\n'
  )
  await writeFile(
    path.join(base, "src-tauri/tauri.conf.json"),
    '{\n  "version": "0.26.1"\n}\n'
  )

  updateVersionFiles("0.27.0", base)

  assert.equal(
    JSON.parse(await readFile(path.join(base, "release/manifest.json")))
      .version,
    "0.27.0"
  )
  assert.match(
    await readFile(path.join(base, "package.json"), "utf8"),
    /0\.27\.0/
  )
  assert.match(
    await readFile(path.join(base, "src-tauri/Cargo.toml"), "utf8"),
    /version = "0\.27\.0"/
  )
  assert.match(
    await readFile(path.join(base, "src-tauri/Cargo.lock"), "utf8"),
    /name = "bugrail"\nversion = "0\.27\.0"/
  )
  assert.match(
    await readFile(path.join(base, "src-tauri/tauri.conf.json"), "utf8"),
    /0\.27\.0/
  )
})
