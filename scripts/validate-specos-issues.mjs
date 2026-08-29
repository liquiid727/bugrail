import { createHash } from "node:crypto"
import { readdir, readFile } from "node:fs/promises"
import path from "node:path"
import { fileURLToPath } from "node:url"

const ACTIVE_STATUSES = new Set([
  "planned",
  "implemented_pending_verification",
  "pending_verification",
  "verified",
  "reopened",
])
const ALLOWED_STATUSES = new Set([...ACTIVE_STATUSES, "superseded"])
const ALLOWED_KINDS = new Set(["implementation", "verification"])

export function parseFrontmatter(raw, source = "artifact") {
  const match = /^---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/.exec(raw)
  if (!match) throw new Error(`${source}: missing YAML frontmatter`)

  const values = {}
  for (const line of match[1].split(/\r?\n/)) {
    if (!line || /^\s/.test(line) || line.trimStart().startsWith("#")) continue
    const separator = line.indexOf(":")
    if (separator < 1) continue
    const key = line.slice(0, separator).trim()
    const value = line.slice(separator + 1).trim()
    values[key] = parseValue(value)
  }
  return values
}

function parseValue(value) {
  if (value.startsWith("[") && value.endsWith("]")) {
    const body = value.slice(1, -1).trim()
    return body ? body.split(",").map((item) => unquote(item.trim())) : []
  }
  return unquote(value)
}

function unquote(value) {
  if (
    value.length >= 2 &&
    ((value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'")))
  ) {
    return value.slice(1, -1)
  }
  return value
}

function sha256(raw) {
  return createHash("sha256").update(raw).digest("hex")
}

async function loadArtifacts(root) {
  const issuesDir = path.join(root, ".issues")
  const featuresDir = path.join(root, ".features")
  const issueNames = (await readdir(issuesDir))
    .filter((name) => name.startsWith("issue-") && name.endsWith(".md"))
    .sort()
  const featureNames = (await readdir(featuresDir, { withFileTypes: true }))
    .filter((entry) => entry.isDirectory() && entry.name.startsWith("BUGRAIL-"))
    .map((entry) => entry.name)

  const issues = []
  for (const name of issueNames) {
    const file = path.join(issuesDir, name)
    const raw = await readFile(file, "utf8")
    issues.push({
      ...parseFrontmatter(raw, path.relative(root, file)),
      file: path.relative(root, file),
      filename: name,
    })
  }

  const specs = new Map()
  for (const name of featureNames) {
    const directory = path.join(featuresDir, name)
    const specFile = path.join(directory, "spec.md")
    const testFile = path.join(directory, "test-spec.md")
    let specRaw
    let testRaw
    try {
      ;[specRaw, testRaw] = await Promise.all([
        readFile(specFile, "utf8"),
        readFile(testFile, "utf8"),
      ])
    } catch (error) {
      if (error?.code === "ENOENT") continue
      throw error
    }
    const spec = parseFrontmatter(specRaw, path.relative(root, specFile))
    const testSpec = parseFrontmatter(testRaw, path.relative(root, testFile))
    specs.set(spec.id, {
      ...spec,
      hash: sha256(specRaw),
      file: path.relative(root, specFile),
      testSpec,
      testFile: path.relative(root, testFile),
    })
  }

  const docsNames = await readdir(path.join(root, "docs"))
  return {
    issues,
    specs,
    implementationNotes: docsNames.filter((name) =>
      /^issue#\d+\.html$/.test(name)
    ),
  }
}

export function validateLedger({ issues, specs, implementationNotes = [] }) {
  const errors = []
  const byId = new Map()
  const numberToId = new Map()

  for (const issue of issues) {
    const fileMatch = /^issue-(\d{3})-(.+)\.md$/.exec(issue.filename ?? "")
    if (!fileMatch) {
      errors.push(`${issue.file}: invalid Issue filename`)
      continue
    }
    const expectedId = `issue-${fileMatch[1]}`
    if (issue.id !== expectedId) {
      errors.push(`${issue.file}: frontmatter id must be ${expectedId}`)
    }
    if (byId.has(issue.id))
      errors.push(`${issue.file}: duplicate id ${issue.id}`)
    byId.set(issue.id, issue)
    const number = Number(fileMatch[1])
    if (numberToId.has(number))
      errors.push(`${issue.file}: duplicate number ${fileMatch[1]}`)
    numberToId.set(number, issue.id)

    if (!ALLOWED_STATUSES.has(issue.status)) {
      errors.push(`${issue.file}: invalid status ${issue.status}`)
    }
    if (!ALLOWED_KINDS.has(issue.kind)) {
      errors.push(`${issue.file}: invalid kind ${issue.kind}`)
    }
    for (const field of [
      "sourceSpecId",
      "sourceSpecVersion",
      "sourceSpecHash",
    ]) {
      if (!issue[field]) errors.push(`${issue.file}: missing ${field}`)
    }
    if (!Array.isArray(issue.dependsOn)) {
      errors.push(`${issue.file}: dependsOn must be an inline list`)
    }
  }

  const maxNumber = Math.max(0, ...numberToId.keys())
  for (let number = 1; number <= maxNumber; number += 1) {
    if (!numberToId.has(number)) {
      errors.push(
        `Issue ledger has an undeclared gap at issue-${String(number).padStart(3, "0")}`
      )
    }
  }

  for (const issue of issues) {
    for (const field of ["dependsOn", "supersededBy"]) {
      for (const reference of Array.isArray(issue[field]) ? issue[field] : []) {
        if (reference === issue.id)
          errors.push(`${issue.file}: ${field} contains itself`)
        if (!byId.has(reference))
          errors.push(`${issue.file}: ${field} references missing ${reference}`)
      }
    }

    if (issue.status === "superseded") continue
    const spec = specs.get(issue.sourceSpecId)
    if (!spec) {
      errors.push(
        `${issue.file}: source Spec ${issue.sourceSpecId} does not exist`
      )
      continue
    }
    if (spec.version !== issue.sourceSpecVersion) {
      errors.push(
        `${issue.file}: source Spec version ${spec.version} does not match ${issue.sourceSpecVersion}`
      )
    }
    if (spec.hash !== issue.sourceSpecHash) {
      errors.push(`${issue.file}: source Spec hash does not match ${spec.file}`)
    }
    if (
      issue.status !== "planned" &&
      (spec.status !== "approved" || spec.testSpec.status !== "approved")
    ) {
      errors.push(
        `${issue.file}: executable Issue requires approved Spec and Test Spec`
      )
    }
  }

  const activeSpecIds = new Set(
    issues
      .filter((issue) => issue.status !== "superseded")
      .map((issue) => issue.sourceSpecId)
  )
  for (const id of activeSpecIds) {
    const spec = specs.get(id)
    if (!spec) continue
    const testSourceId = spec.testSpec.sourceSpecId ?? spec.testSpec.feature
    const testSourceVersion =
      spec.testSpec.sourceSpecVersion ?? spec.testSpec.featureVersion
    if (testSourceId !== spec.id) {
      errors.push(
        `${spec.testFile}: Test Spec source does not match ${spec.id}`
      )
    }
    if (testSourceVersion !== spec.version) {
      errors.push(
        `${spec.testFile}: Test Spec version does not match ${spec.version}`
      )
    }
    if (
      spec.testSpec.sourceSpecHash &&
      spec.testSpec.sourceSpecHash !== spec.hash
    ) {
      errors.push(
        `${spec.testFile}: sourceSpecHash does not match ${spec.file}`
      )
    }
  }

  detectActiveCycles(issues, byId, errors)
  validateVerifiedDependencies(issues, byId, errors)

  for (const note of implementationNotes) {
    errors.push(
      `docs/${note}: implementation notes belong in the canonical Issue Completion Record`
    )
  }

  return errors
}

function detectActiveCycles(issues, byId, errors) {
  const visiting = new Set()
  const visited = new Set()

  function visit(issue, trail) {
    if (visited.has(issue.id) || issue.status === "superseded") return
    if (visiting.has(issue.id)) {
      const start = trail.indexOf(issue.id)
      errors.push(
        `active dependency cycle: ${[...trail.slice(start), issue.id].join(" -> ")}`
      )
      return
    }
    visiting.add(issue.id)
    for (const id of issue.dependsOn ?? []) {
      const dependency = byId.get(id)
      if (dependency && dependency.status !== "superseded")
        visit(dependency, [...trail, issue.id])
    }
    visiting.delete(issue.id)
    visited.add(issue.id)
  }

  for (const issue of issues) visit(issue, [])
}

function validateVerifiedDependencies(issues, byId, errors) {
  for (const issue of issues) {
    if (issue.status !== "verified") continue
    for (const id of issue.dependsOn ?? []) {
      const dependency = byId.get(id)
      if (!dependency || dependency.status === "superseded") continue
      if (issue.kind === "verification" && dependency.status !== "verified") {
        errors.push(
          `${issue.file}: verified verification Issue depends on non-verified ${id}`
        )
      }
      if (
        issue.kind === "implementation" &&
        !["implemented_pending_verification", "verified"].includes(
          dependency.status
        )
      ) {
        errors.push(
          `${issue.file}: verified implementation Issue depends on unimplemented ${id}`
        )
      }
    }
  }
}

export async function validateRepository(root = process.cwd()) {
  const artifacts = await loadArtifacts(root)
  return validateLedger(artifacts)
}

async function main() {
  const root = path.resolve(process.argv[2] ?? process.cwd())
  const errors = await validateRepository(root)
  if (errors.length > 0) {
    for (const error of errors) console.error(`- ${error}`)
    process.exitCode = 1
    return
  }
  console.log("SpecOS Issue ledger is valid")
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
