#!/usr/bin/env node

import { readFileSync, writeFileSync } from "node:fs"
import path from "node:path"
import { fileURLToPath } from "node:url"

const REPO_ROOT = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  ".."
)
const MANIFEST_REL = "release/manifest.json"
const VERSION_RE =
  /^\d+\.\d+\.\d+(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/

function absolute(base, rel) {
  return path.join(base, rel)
}

function readJson(rel, base = REPO_ROOT) {
  return JSON.parse(readFileSync(absolute(base, rel), "utf8"))
}

function writeJson(rel, value, base = REPO_ROOT) {
  writeFileSync(absolute(base, rel), `${JSON.stringify(value, null, 2)}\n`)
}

function readCargoPackageVersion(base = REPO_ROOT) {
  const content = readFileSync(absolute(base, "src-tauri/Cargo.toml"), "utf8")
  const packageStart = content.search(/^\[package\]$/m)
  const nextSection = content.indexOf("\n[", packageStart + 1)
  const section = content.slice(
    packageStart,
    nextSection < 0 ? undefined : nextSection
  )
  return section.match(/^version = "([^"]+)"$/m)?.[1] ?? null
}

function readCargoLockVersion(base = REPO_ROOT) {
  const content = readFileSync(absolute(base, "src-tauri/Cargo.lock"), "utf8")
  const packageBlock = content.match(
    /\[\[package\]\]\nname = "bugrail"\nversion = "([^"]+)"[\s\S]*?(?=\n\[\[package\]\]|\s*$)/
  )
  return packageBlock?.[1] ?? null
}

function replaceFirstJsonVersion(rel, version, base = REPO_ROOT) {
  const file = absolute(base, rel)
  const content = readFileSync(file, "utf8")
  const next = content.replace(
    /(^\s*"version"\s*:\s*")[^"]+("\s*,?\s*$)/m,
    `$1${version}$2`
  )
  if (next === content) throw new Error(`${rel} has no root version field`)
  writeFileSync(file, next)
}

function replaceCargoVersion(version, base = REPO_ROOT) {
  const file = absolute(base, "src-tauri/Cargo.toml")
  const content = readFileSync(file, "utf8")
  const packageStart = content.search(/^\[package\]$/m)
  if (packageStart < 0) throw new Error("Cargo.toml has no [package] section")
  const nextSection = content.indexOf("\n[", packageStart + 1)
  const sectionEnd = nextSection < 0 ? content.length : nextSection
  const section = content.slice(packageStart, sectionEnd)
  const nextSectionText = section.replace(
    /^version = "[^"]+"$/m,
    `version = "${version}"`
  )
  if (nextSectionText === section) {
    throw new Error("Cargo.toml [package] has no version field")
  }
  writeFileSync(
    file,
    content.slice(0, packageStart) + nextSectionText + content.slice(sectionEnd)
  )
}

function replaceCargoLockVersion(version, base = REPO_ROOT) {
  const file = absolute(base, "src-tauri/Cargo.lock")
  const content = readFileSync(file, "utf8")
  const next = content.replace(
    /(\[\[package\]\]\nname = "bugrail"\nversion = ")[^"]+(")/,
    `$1${version}$2`
  )
  if (next === content) {
    throw new Error('Cargo.lock has no package entry for "bugrail"')
  }
  writeFileSync(file, next)
}

export function normalizeVersion(value) {
  const version = String(value ?? "")
    .trim()
    .replace(/^v/, "")
  if (!VERSION_RE.test(version)) {
    throw new Error(`invalid semver version: ${value}`)
  }
  return version
}

export function tagForVersion(value) {
  return `v${normalizeVersion(value)}`
}

export function readVersionSources(base = REPO_ROOT) {
  return {
    manifest: readJson(MANIFEST_REL, base).version,
    packageJson: readJson("package.json", base).version,
    cargoToml: readCargoPackageVersion(base),
    cargoLock: readCargoLockVersion(base),
    tauri: readJson("src-tauri/tauri.conf.json", base).version,
  }
}

export function readRuntimeSources(base = REPO_ROOT) {
  const productRust = readFileSync(
    absolute(base, "src-tauri/src/product/mod.rs"),
    "utf8"
  )
  const tauri = readJson("src-tauri/tauri.conf.json", base)
  const stringValue = (name) =>
    productRust.match(new RegExp(`${name}:\\s*"([^"]+)"`))?.[1] ?? null

  return {
    rustRepository: stringValue("repository_slug"),
    rustUpdaterManifestUrl: stringValue("update_manifest_url"),
    rustDownloadBase: stringValue("release_download_base"),
    tauriProductName: tauri.productName,
    tauriIdentifier: tauri.identifier,
    tauriUpdaterManifestUrl: tauri.plugins?.updater?.endpoints?.[0] ?? null,
  }
}

export function validateReleaseData(manifest, versions, runtime = {}) {
  const errors = []
  const product = manifest?.product
  const release = manifest?.release
  const container = manifest?.container
  const expectedRepository = "liquiid727/bugrail"
  const expectedBase = `https://github.com/${expectedRepository}`

  if (!VERSION_RE.test(manifest?.version ?? "")) {
    errors.push(
      `release/manifest.json has invalid version: ${manifest?.version}`
    )
  }

  const versionsSeen = Object.entries(versions).filter(([, value]) => value)
  for (const [name, value] of versionsSeen) {
    if (value !== manifest.version) {
      errors.push(`${name} version ${value} does not match ${manifest.version}`)
    }
  }
  for (const name of [
    "manifest",
    "packageJson",
    "cargoToml",
    "cargoLock",
    "tauri",
  ]) {
    if (!versions[name]) errors.push(`missing version source: ${name}`)
  }

  if (product?.name !== "Bugrail") errors.push("product.name must be Bugrail")
  if (product?.slug !== "bugrail") errors.push("product.slug must be bugrail")
  if (product?.repository !== expectedRepository) {
    errors.push(`product.repository must be ${expectedRepository}`)
  }
  if (product?.bundleIdentifier !== "io.liquiid.bugrail") {
    errors.push("product.bundleIdentifier must be io.liquiid.bugrail")
  }

  const expectedUrls = {
    repositoryUrl: expectedBase,
    releasesUrl: `${expectedBase}/releases`,
    latestReleaseUrl: `${expectedBase}/releases/latest`,
    updaterManifestUrl: `${expectedBase}/releases/latest/download/latest.json`,
    downloadBase: `${expectedBase}/releases/latest/download`,
  }
  for (const [key, expected] of Object.entries(expectedUrls)) {
    if (release?.[key] !== expected) {
      errors.push(`release.${key} must be ${expected}`)
    }
  }
  if (release?.tagPrefix !== "v") errors.push('release.tagPrefix must be "v"')
  if (release?.releaseName !== product?.name) {
    errors.push("release.releaseName must match product.name")
  }

  if (container?.registry !== "ghcr.io") {
    errors.push('container.registry must be "ghcr.io"')
  }
  if (container?.image !== `ghcr.io/${expectedRepository}`) {
    errors.push(`container.image must be ghcr.io/${expectedRepository}`)
  }
  if (manifest?.compatibilityArtifacts?.server !== "codeg-server") {
    errors.push('compatibilityArtifacts.server must remain "codeg-server"')
  }
  if (manifest?.compatibilityArtifacts?.mcp !== "codeg-mcp") {
    errors.push('compatibilityArtifacts.mcp must remain "codeg-mcp"')
  }

  const runtimeExpectations = {
    rustRepository: expectedRepository,
    rustUpdaterManifestUrl: expectedUrls.updaterManifestUrl,
    rustDownloadBase: expectedUrls.downloadBase,
    tauriProductName: product?.name,
    tauriIdentifier: product?.bundleIdentifier,
    tauriUpdaterManifestUrl: expectedUrls.updaterManifestUrl,
  }
  for (const [key, expected] of Object.entries(runtimeExpectations)) {
    if (runtime[key] !== expected) {
      errors.push(`runtime.${key} must be ${expected}`)
    }
  }

  return errors
}

export function validateRepository(base = REPO_ROOT) {
  const manifest = readJson(MANIFEST_REL, base)
  const versions = readVersionSources(base)
  const runtime = readRuntimeSources(base)
  const errors = validateReleaseData(manifest, versions, runtime)
  if (errors.length) {
    throw new Error(
      `release metadata validation failed:\n- ${errors.join("\n- ")}`
    )
  }
  return { manifest, versions, runtime, tag: tagForVersion(manifest.version) }
}

export function updateVersionFiles(value, base = REPO_ROOT) {
  const version = normalizeVersion(value)
  const manifest = readJson(MANIFEST_REL, base)
  manifest.version = version
  writeJson(MANIFEST_REL, manifest, base)
  replaceFirstJsonVersion("package.json", version, base)
  replaceCargoVersion(version, base)
  replaceCargoLockVersion(version, base)
  replaceFirstJsonVersion("src-tauri/tauri.conf.json", version, base)
  return version
}

function usage() {
  console.log(`Usage:
  node scripts/release.mjs check
  node scripts/release.mjs show
  node scripts/release.mjs version
  node scripts/release.mjs tag [version]
  node scripts/release.mjs name
  node scripts/release.mjs image
  node scripts/release.mjs set <version>`)
}

function main(args) {
  const [command, value] = args
  if (command === "check") {
    const result = validateRepository()
    console.log(
      JSON.stringify(
        {
          product: result.manifest.product.name,
          version: result.manifest.version,
          tag: result.tag,
          repository: result.manifest.product.repository,
          updaterManifestUrl: result.manifest.release.updaterManifestUrl,
          containerImage: result.manifest.container.image,
        },
        null,
        2
      )
    )
    return
  }
  if (command === "show") {
    const result = validateRepository()
    console.log(JSON.stringify(result, null, 2))
    return
  }
  if (command === "version") {
    console.log(validateRepository().manifest.version)
    return
  }
  if (command === "tag") {
    console.log(tagForVersion(value ?? validateRepository().manifest.version))
    return
  }
  if (command === "name") {
    console.log(validateRepository().manifest.release.releaseName)
    return
  }
  if (command === "image") {
    console.log(validateRepository().manifest.container.image)
    return
  }
  if (command === "set" && value) {
    const version = updateVersionFiles(value)
    console.log(
      `Updated Bugrail version to ${version} (${tagForVersion(version)})`
    )
    return
  }
  usage()
  process.exitCode = 1
}

if (
  process.argv[1] &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  try {
    main(process.argv.slice(2))
  } catch (error) {
    console.error(error instanceof Error ? error.message : error)
    process.exitCode = 1
  }
}
