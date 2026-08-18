import assert from "node:assert/strict"
import { mkdtempSync, writeFileSync, readFileSync, mkdirSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import path from "node:path"
import test from "node:test"

import {
  bumpCargoTomlVersion,
  bumpJsonVersion,
  fixCargoLock,
  resolveBlock,
  updateReadmeBaseline,
  versionFromTag,
} from "./sync-upstream.mjs"

function tempRepo() {
  const root = mkdtempSync(path.join(tmpdir(), "bugrail-sync-"))
  mkdirSync(path.join(root, "src-tauri"), { recursive: true })
  return root
}

test("versionFromTag strips a leading v", () => {
  assert.equal(versionFromTag("v0.23.3"), "0.23.3")
  assert.equal(versionFromTag("0.24.0"), "0.24.0")
})

test("bumpJsonVersion rewrites the version field", () => {
  const root = tempRepo()
  writeFileSync(
    path.join(root, "package.json"),
    '{\n  "name": "bugrail",\n  "version": "0.23.2"\n}\n'
  )
  const previous = bumpJsonVersion("package.json", "0.23.3", root)
  assert.equal(previous, "0.23.2")
  const pkg = JSON.parse(readFileSync(path.join(root, "package.json"), "utf8"))
  assert.equal(pkg.name, "bugrail")
  assert.equal(pkg.version, "0.23.3")
  rmSync(root, { recursive: true, force: true })
})

test("bumpCargoTomlVersion rewrites only the [package] version", () => {
  const root = tempRepo()
  const manifest = `[package]
name = "bugrail"
version = "0.23.2"
description = "Spec-driven AI coding workspace based on CodeG"

[lib]
name = "codeg_lib"
`
  writeFileSync(path.join(root, "src-tauri/Cargo.toml"), manifest)
  const previous = bumpCargoTomlVersion("0.23.3", root)
  assert.equal(previous, "0.23.2")
  const out = readFileSync(path.join(root, "src-tauri/Cargo.toml"), "utf8")
  assert.match(out, /name = "bugrail"/)
  assert.match(out, /version = "0\.23\.3"/)
  assert.match(out, /\[lib\]/)
  rmSync(root, { recursive: true, force: true })
})

test("fixCargoLock drops a codeg entry and bumps the bugrail version", () => {
  const root = tempRepo()
  const lock = `[[package]]
name = "bugrail"
version = "0.23.2"
dependencies = ["serde"]

[[package]]
name = "codeg"
version = "0.23.3"
dependencies = ["serde"]

[[package]]
name = "colorchoice"
version = "1.0.4"
`
  writeFileSync(path.join(root, "src-tauri/Cargo.lock"), lock)
  fixCargoLock("0.23.3", root)
  const out = readFileSync(path.join(root, "src-tauri/Cargo.lock"), "utf8")
  assert.match(out, /name = "bugrail"\nversion = "0\.23\.3"/)
  assert.doesNotMatch(out, /name = "codeg"/)
  assert.match(out, /name = "colorchoice"/)
  rmSync(root, { recursive: true, force: true })
})

test("updateReadmeBaseline rewrites the tracked CodeG release", () => {
  const root = tempRepo()
  writeFileSync(
    path.join(root, "README.md"),
    "The current bootstrap tracks CodeG release `v0.23.2`. Upstream docs.\n"
  )
  updateReadmeBaseline("v0.23.3", root)
  const out = readFileSync(path.join(root, "README.md"), "utf8")
  assert.match(out, /CodeG release `v0\.23\.3`/)
  rmSync(root, { recursive: true, force: true })
})

test("resolves the Cargo.toml package conflict keeping Bugrail identity", () => {
  const resolved = resolveBlock(
    "src-tauri/Cargo.toml",
    'name = "bugrail"\nversion = "0.23.2"\ndescription = "Spec-driven AI coding workspace based on CodeG"\nauthors = ["liquiid727", "feitao"]',
    'name = "codeg"\nversion = "0.23.3"\ndescription = "Agent Code Generation App"\nauthors = ["feitao"]'
  )
  assert.equal(
    resolved,
    'name = "bugrail"\nversion = "0.23.3"\ndescription = "Spec-driven AI coding workspace based on CodeG"\nauthors = ["liquiid727", "feitao"]'
  )
})

test("resolves the tauri.conf.json conflict keeping Bugrail identity", () => {
  const head = `  "productName": "Bugrail",
  "version": "0.23.2",
  "identifier": "io.liquiid.bugrail",
  "mainBinaryName": "codeg",`
  const merge = `  "productName": "codeg",
  "version": "0.23.3",
  "identifier": "app.codeg",`
  const resolved = resolveBlock("src-tauri/tauri.conf.json", head, merge)
  assert.equal(
    resolved,
    `  "productName": "Bugrail",
  "version": "0.23.3",
  "identifier": "io.liquiid.bugrail",
  "mainBinaryName": "codeg",`
  )
})

test("resolves the package.json conflict keeping the bugrail name", () => {
  const resolved = resolveBlock(
    "package.json",
    '  "name": "bugrail",\n  "version": "0.23.2",',
    '  "name": "codeg",\n  "version": "0.23.3",'
  )
  assert.equal(resolved, '  "name": "bugrail",\n  "version": "0.23.3",')
})

test("resolves the Cargo.lock conflict by dropping the upstream codeg entry", () => {
  const resolved = resolveBlock(
    "src-tauri/Cargo.lock",
    "",
    'name = "codeg"\nversion = "0.23.3"\ndependencies = ["serde"]\n'
  )
  assert.equal(resolved, "")
})

test("resolves a Cargo.lock conflict keeping the bugrail entry", () => {
  const head =
    'name = "bugrail"\nversion = "0.23.2"\ndependencies = ["serde", "smol-toml"]'
  const merge =
    'name = "codeg"\nversion = "0.23.3"\ndependencies = ["serde", "smol-toml"]'
  const resolved = resolveBlock("src-tauri/Cargo.lock", head, merge)
  assert.equal(resolved, head)
})

test("returns null for unknown files so they stay for manual review", () => {
  assert.equal(resolveBlock("src/lib/foo.ts", "a", "b"), null)
})

test("returns null for an unexpected conflict shape", () => {
  // merge side no longer declares the codeg identity: not the known shape
  const resolved = resolveBlock(
    "src-tauri/Cargo.toml",
    'name = "bugrail"\nversion = "0.23.2"',
    'name = "bugrail"\nversion = "0.23.3"'
  )
  assert.equal(resolved, null)
})
