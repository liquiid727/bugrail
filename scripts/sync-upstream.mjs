#!/usr/bin/env node
// Sync a newer immutable CodeG release tag into the Bugrail fork without
// rewriting published history. The script does the mechanical parts (check,
// fetch, verify, branch, merge, version/baseline bumps) and stops for human
// review of anything it cannot resolve deterministically.
import { execFileSync } from "node:child_process"
import { readFileSync, writeFileSync } from "node:fs"
import path from "node:path"
import { fileURLToPath } from "node:url"

import {
  evaluateUpstreamRelease,
  fetchLatestReleaseTag,
  resolveTagCommit,
} from "./check-upstream-release.mjs"

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..")
const CONFIG_PATH = path.join(REPO_ROOT, ".bugrail-upstream.json")
const CONFIG_REL = ".bugrail-upstream.json"
const UPSTREAM_BASE = "https://github.com"

const IDENTITY_FILES = [
  "package.json",
  "README.md",
  "src-tauri/Cargo.toml",
  "src-tauri/Cargo.lock",
  "src-tauri/tauri.conf.json",
]

function git(...args) {
  return execFileSync("git", args, { cwd: REPO_ROOT, encoding: "utf8" }).trim()
}

function gitOk(...args) {
  try {
    git(...args)
    return true
  } catch {
    return false
  }
}

function readJson(rel) {
  return JSON.parse(readFileSync(path.join(REPO_ROOT, rel), "utf8"))
}

function writeJson(rel, value) {
  writeFileSync(path.join(REPO_ROOT, rel), `${JSON.stringify(value, null, 2)}\n`)
}

function isDirtyWorktree() {
  return git("status", "--porcelain").length > 0
}

export function versionFromTag(tag) {
  return tag.replace(/^v/, "")
}

function peelTag(tag) {
  return git("rev-parse", `${tag}^{commit}`)
}

function ensureUpstreamRemote() {
  const url = `${UPSTREAM_BASE}/${readJson(CONFIG_REL).upstreamRepository}.git`
  if (!git("remote").split("\n").includes("upstream")) {
    git("remote", "add", "upstream", url)
    console.log(`ℹ  added upstream remote ${url}`)
  }
}

async function resolveLatestRelease(config) {
  const latestTag = await fetchLatestReleaseTag(config.upstreamRepository)
  const latestCommit = await resolveTagCommit(config.upstreamRepository, latestTag)
  return evaluateUpstreamRelease({ ...config, latestTag, latestCommit })
}

// ---- version / baseline file edits ---------------------------------------

export function bumpJsonVersion(rel, version, base = REPO_ROOT) {
  const file = path.join(base, rel)
  const pkg = JSON.parse(readFileSync(file, "utf8"))
  const previous = pkg.version
  if (previous === version) return previous
  pkg.version = version
  writeFileSync(file, `${JSON.stringify(pkg, null, 2)}\n`)
  return previous
}

export function bumpCargoTomlVersion(version, base = REPO_ROOT) {
  const file = path.join(base, "src-tauri/Cargo.toml")
  let content = readFileSync(file, "utf8")
  const pkgStart = content.search(/^\[package\]/m)
  if (pkgStart < 0) throw new Error("Cargo.toml has no [package] section")
  const pkgEnd = content.indexOf("\n[", pkgStart + 1)
  const section = content.slice(pkgStart, pkgEnd < 0 ? undefined : pkgEnd)
  const previous = (section.match(/^version = "([^"]+)"/m) || [])[1]
  if (previous === version) return previous
  const next = section.replace(/^version = "[^"]*"$/m, `version = "${version}"`)
  if (next === section) throw new Error("Cargo.toml [package] has no version line")
  content = content.slice(0, pkgStart) + next + (pkgEnd < 0 ? "" : content.slice(pkgEnd))
  writeFileSync(file, content)
  return previous
}

export function fixCargoLock(version, base = REPO_ROOT) {
  const file = path.join(base, "src-tauri/Cargo.lock")
  let content = readFileSync(file, "utf8")
  // Drop any lingering top-level codeg package entry (renamed to bugrail).
  const blocks = content.split(/\n(?=\[\[package\]\])/)
  const kept = blocks.filter((block) => {
    const name = (block.match(/^name = "([^"]+)"/m) || [])[1]
    return name !== "codeg"
  })
  content = kept.join("\n")
  content = content.replace(
    /^name = "bugrail"\nversion = "[^"]*"/m,
    `name = "bugrail"\nversion = "${version}"`
  )
  writeFileSync(file, content)
}

export function updateReadmeBaseline(tag, base = REPO_ROOT) {
  const file = path.join(base, "README.md")
  let content = readFileSync(file, "utf8")
  const next = content.replace(
    /CodeG release `v[0-9]+\.[0-9]+\.[0-9]+`/,
    `CodeG release \`${tag}\``
  )
  if (next !== content) writeFileSync(file, next)
}

// ---- deterministic conflict resolution ------------------------------------
// The fork and upstream only ever clash on the product-identity seams. Each
// known seam has a single correct resolution: keep the Bugrail identity fields
// (name, identifier, data roots, endpoints) and adopt the upstream version.
// Anything that does not match the expected shape is left for manual review.

const CONFLICT_BLOCK_RE =
  /^<<<<<<< (?:HEAD|(?!v)[^\n]*)\n([\s\S]*?)^=======\n([\s\S]*?)^>>>>>>> [^\n]*$/gm

export function resolveBlock(relPath, headSide, mergeSide) {
  switch (relPath) {
    case "src-tauri/Cargo.toml":
      return resolveCargoTomlBlock(headSide, mergeSide)
    case "src-tauri/tauri.conf.json":
      return resolveTauriConfBlock(headSide, mergeSide)
    case "package.json":
      return resolvePackageJsonBlock(headSide, mergeSide)
    case "src-tauri/Cargo.lock":
      return resolveCargoLockBlock(headSide, mergeSide)
    default:
      return null
  }
}

function resolveCargoTomlBlock(head, merge) {
  const headOwns = /^name = "bugrail"$/m.test(head)
  const mergeFromUpstream = /^name = "codeg"$/m.test(merge)
  const version = (merge.match(/^version = "([^"]+)"/m) || [])[1]
  if (!(headOwns && mergeFromUpstream && version)) return null
  return head.replace(/^version = "[^"]*"$/m, `version = "${version}"`)
}

function resolveTauriConfBlock(head, merge) {
  const headOwns = /"identifier"\s*:\s*"io\.liquiid\.bugrail"/.test(head)
  const mergeFromUpstream = /"identifier"\s*:\s*"app\.codeg"/.test(merge)
  const m = merge.match(/^(\s*)"version"\s*:\s*"([^"]+)"(,?)\s*$/m)
  if (!(headOwns && mergeFromUpstream && m)) return null
  const [, indent, version, comma] = m
  return head.replace(
    /^(\s*)"version"\s*:\s*"[^"]*"(,?)\s*$/m,
    `${indent}"version": "${version}"${comma}`
  )
}

function resolvePackageJsonBlock(head, merge) {
  const headOwns = /"name"\s*:\s*"bugrail"/.test(head)
  const m = merge.match(/^(\s*)"version"\s*:\s*"([^"]+)"(,?)\s*$/m)
  if (!(headOwns && m)) return null
  const [, indent, version, comma] = m
  return head.replace(
    /^(\s*)"version"\s*:\s*"[^"]*"(,?)\s*$/m,
    `${indent}"version": "${version}"${comma}`
  )
}

function resolveCargoLockBlock(head, merge) {
  // Upstream's renamed package entry: keep our bugrail block (or drop the
  // merge side entirely if head is empty), never adopt the codeg name.
  if (!/^name = "codeg"$/m.test(merge)) return null
  return /^name = "bugrail"$/m.test(head) ? head : ""
}

export function resolveConflicts() {
  const conflicted = git("diff", "--name-only", "--diff-filter=U")
    .split("\n")
    .filter(Boolean)
  const resolved = []
  const remaining = []
  for (const rel of conflicted) {
    const file = path.join(REPO_ROOT, rel)
    const content = readFileSync(file, "utf8")
    const blocks = [...content.matchAll(CONFLICT_BLOCK_RE)]
    if (blocks.length === 0) {
      remaining.push(rel)
      continue
    }
    const resolutions = blocks.map((b) => ({
      start: b.index,
      end: b.index + b[0].length,
      text: resolveBlock(rel, b[1], b[2]),
    }))
    if (resolutions.some((r) => r.text === null)) {
      remaining.push(rel)
      continue
    }
    let out = content
    for (let i = resolutions.length - 1; i >= 0; i -= 1) {
      const { start, end, text } = resolutions[i]
      out = out.slice(0, start) + text + out.slice(end)
    }
    writeFileSync(file, out)
    git("add", rel)
    resolved.push(rel)
  }
  return { resolved, remaining }
}

// ---- subcommands ----------------------------------------------------------

async function cmdStatus() {
  const config = readJson(CONFIG_REL)
  const result = await resolveLatestRelease(config)
  console.log(JSON.stringify({ ...config, ...result }, null, 2))
}

async function cmdPrepare({ tag, dryRun }) {
  if (isDirtyWorktree()) {
    console.error("✗ worktree is not clean; commit or stash changes first.")
    process.exit(1)
  }
  const config = readJson(CONFIG_REL)
  ensureUpstreamRemote()

  let targetTag = tag
  let targetCommit = null
  if (tag) {
    if (!/^v?\d+\.\d+\.\d+$/.test(tag)) {
      console.error(`✗ invalid release tag: ${tag}`)
      process.exit(1)
    }
    targetTag = tag.startsWith("v") ? tag : `v${tag}`
    targetCommit = await resolveTagCommit(config.upstreamRepository, targetTag)
  } else {
    const result = await resolveLatestRelease(config)
    if (result.status !== "update-available") {
      console.log(JSON.stringify({ ...config, ...result }, null, 2))
      console.log("No upstream update available.")
      return
    }
    targetTag = result.latestTag
    targetCommit = result.latestCommit
  }

  console.log(`ℹ  target: ${targetTag} @ ${targetCommit}`)
  if (dryRun) {
    console.log("ℹ  dry-run: no branch or merge performed.")
    return
  }

  git("fetch", "upstream", "--tags", "--force")
  const peeled = peelTag(targetTag)
  if (peeled !== targetCommit) {
    console.error(
      `✗ tag ${targetTag} peeled to ${peeled}, expected ${targetCommit}; refusing to sync a moved tag`
    )
    process.exit(1)
  }

  const branch = `sync/upstream-${targetTag}`
  if (gitOk("rev-parse", "--verify", `refs/heads/${branch}`)) {
    git("checkout", branch)
  } else {
    git("checkout", "-b", branch)
  }
  console.log(`ℹ  on branch ${branch}`)

  const merged = gitOk("merge", "--no-commit", targetTag)
  if (!merged && !gitOk("rev-parse", "--verify", "MERGE_HEAD")) {
    // A merge that fails outright (e.g. local changes) leaves no MERGE_HEAD.
    // A conflicted merge has started and must continue through resolution.
    console.error("✗ merge could not start; see git status")
    process.exit(1)
  }

  const { resolved, remaining } = resolveConflicts()
  if (resolved.length) {
    console.log(`✔ auto-resolved known identity conflicts: ${resolved.join(", ")}`)
  }
  if (remaining.length) {
    console.error(
      `✗ merge conflicts remain that need manual review: ${remaining.join(", ")}`
    )
    console.error("  Resolve them, then run: node scripts/sync-upstream.mjs finalize --tag <tag>")
    process.exit(1)
  }

  const stillConflicted = git("diff", "--name-only", "--diff-filter=U").trim()
  if (stillConflicted) {
    console.error(`✗ unexpected conflicts remain: ${stillConflicted}`)
    process.exit(1)
  }

  cmdFinalize({ tag: targetTag, commit: targetCommit })
}

function cmdFinalize({ tag }) {
  if (!tag) {
    console.error("✗ finalize requires --tag <vX.Y.Z>")
    process.exit(1)
  }
  const targetTag = tag.startsWith("v") ? tag : `v${tag}`
  const version = versionFromTag(targetTag)
  const config = readJson(CONFIG_REL)
  const targetCommit = peelTag(targetTag)

  console.log(`ℹ  finalizing baseline ${config.baselineTag} → ${targetTag}`)

  config.baselineTag = targetTag
  config.baselineCommit = targetCommit
  writeJson(CONFIG_REL, config)

  bumpJsonVersion("package.json", version)
  bumpCargoTomlVersion(version)
  bumpJsonVersion("src-tauri/tauri.conf.json", version)
  fixCargoLock(version)
  updateReadmeBaseline(targetTag)

  console.log("✔ updated .bugrail-upstream.json, versions, and README baseline")
  console.log("✔ running upstream-check tests…")
  execFileSync("node", ["--test", "scripts/check-upstream-release.test.mjs"], {
    cwd: REPO_ROOT,
    stdio: "inherit",
  })

  git("add", CONFIG_REL, ...IDENTITY_FILES)

  console.log("\nSync staged on the current branch. Next steps:")
  console.log("  1. Review the staged diff:  git diff --cached --stat")
  console.log("  2. Run gates:               cargo check && pnpm lint && pnpm build")
  console.log("  3. Commit the merge:        git commit -m \"merge: integrate upstream CodeG <tag>\"")
  console.log("  4. Push:                    git push origin <branch>")
  console.log("  5. Update parent gitlink in a separate parent-repository change.")
}

function usage() {
  console.log(
    `Usage:
  node scripts/sync-upstream.mjs status                     check for a newer upstream release
  node scripts/sync-upstream.mjs prepare [--tag <vX.Y.Z>] [--dry-run]
      fetch upstream, verify the tag, create sync/upstream-<tag>, and merge --no-commit.
      Auto-resolves the known Bugrail identity conflicts, then runs finalize.
  node scripts/sync-upstream.mjs finalize --tag <vX.Y.Z>
      after resolving any remaining conflicts: update baseline + versions, run
      upstream-check tests, and stage the identity files.`
  )
}

async function main() {
  const [cmd, ...rest] = process.argv.slice(2)
  const flags = {}
  for (let i = 0; i < rest.length; i += 1) {
    const a = rest[i]
    if (a === "--tag") flags.tag = rest[++i]
    else if (a === "--dry-run") flags.dryRun = true
  }

  if (cmd === "status") await cmdStatus()
  else if (cmd === "prepare") {
    await cmdPrepare({ tag: flags.tag, dryRun: flags.dryRun })
  } else if (cmd === "finalize") await cmdFinalize({ tag: flags.tag })
  else usage()
}

if (
  process.argv[1] &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : error)
    process.exit(1)
  })
}
