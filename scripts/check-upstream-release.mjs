import { readFile } from "node:fs/promises"
import path from "node:path"
import { fileURLToPath } from "node:url"

function parseReleaseTag(tag) {
  const match = /^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?$/.exec(tag)
  if (!match) throw new Error(`unsupported release tag: ${tag}`)
  return {
    major: Number(match[1]),
    minor: Number(match[2]),
    patch: Number(match[3]),
    prerelease: match[4] ?? null,
  }
}

export function compareReleaseTags(leftTag, rightTag) {
  const left = parseReleaseTag(leftTag)
  const right = parseReleaseTag(rightTag)
  for (const key of ["major", "minor", "patch"]) {
    if (left[key] !== right[key]) return left[key] > right[key] ? 1 : -1
  }
  if (left.prerelease === right.prerelease) return 0
  if (left.prerelease == null) return 1
  if (right.prerelease == null) return -1
  return left.prerelease.localeCompare(right.prerelease)
}

export async function peelGitHubTagObject(object, loadTag) {
  let current = object
  for (let depth = 0; depth < 8; depth += 1) {
    if (current.type === "commit") return current.sha
    if (current.type !== "tag") {
      throw new Error(
        `release ref resolved to unsupported object: ${current.type}`
      )
    }
    const tag = await loadTag(current.sha)
    current = tag.object
  }
  throw new Error("release tag nesting exceeds the supported depth")
}

export function evaluateUpstreamRelease({
  baselineTag,
  baselineCommit,
  latestTag,
  latestCommit,
}) {
  const comparison = compareReleaseTags(latestTag, baselineTag)
  if (comparison === 0 && latestCommit !== baselineCommit) {
    return { status: "tag-moved", latestTag, latestCommit }
  }
  if (comparison > 0) {
    return { status: "update-available", latestTag, latestCommit }
  }
  if (comparison < 0) {
    return { status: "baseline-ahead", latestTag, latestCommit }
  }
  return { status: "up-to-date", latestTag, latestCommit }
}

export async function fetchLatestReleaseTag(repository) {
  const payload = await fetchGitHubJson(
    `https://api.github.com/repos/${repository}/releases/latest`
  )
  if (typeof payload.tag_name !== "string" || payload.tag_name.length === 0) {
    throw new Error("GitHub latest release has no tag_name")
  }
  return payload.tag_name
}

async function fetchGitHubJson(url) {
  const response = await fetch(url, {
    headers: {
      Accept: "application/vnd.github+json",
      "User-Agent": "bugrail-upstream-check",
      ...(process.env.GITHUB_TOKEN
        ? { Authorization: `Bearer ${process.env.GITHUB_TOKEN}` }
        : {}),
    },
  })
  if (!response.ok) {
    throw new Error(`GitHub API request failed: HTTP ${response.status}`)
  }
  return response.json()
}

export async function resolveTagCommit(repository, tag) {
  const ref = await fetchGitHubJson(
    `https://api.github.com/repos/${repository}/git/ref/tags/${encodeURIComponent(tag)}`
  )
  return peelGitHubTagObject(ref.object, (sha) =>
    fetchGitHubJson(
      `https://api.github.com/repos/${repository}/git/tags/${sha}`
    )
  )
}

async function appendGitHubOutput(result, config) {
  if (!process.env.GITHUB_OUTPUT) return
  const { appendFile } = await import("node:fs/promises")
  await appendFile(
    process.env.GITHUB_OUTPUT,
    [
      `status=${result.status}`,
      `latest_tag=${result.latestTag}`,
      `latest_commit=${result.latestCommit}`,
      `baseline_tag=${config.baselineTag}`,
      `baseline_commit=${config.baselineCommit}`,
      "",
    ].join("\n")
  )
}

async function main() {
  const configIndex = process.argv.indexOf("--config")
  const configPath = path.resolve(
    configIndex >= 0 ? process.argv[configIndex + 1] : ".bugrail-upstream.json"
  )
  const config = JSON.parse(await readFile(configPath, "utf8"))
  const latestTag = await fetchLatestReleaseTag(config.upstreamRepository)
  const latestCommit = await resolveTagCommit(
    config.upstreamRepository,
    latestTag
  )
  const result = evaluateUpstreamRelease({ ...config, latestTag, latestCommit })

  await appendGitHubOutput(result, config)
  console.log(JSON.stringify({ ...config, ...result }, null, 2))

  if (result.status === "tag-moved") process.exitCode = 1
}

if (
  process.argv[1] &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : error)
    process.exitCode = 1
  })
}
