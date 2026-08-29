import assert from "node:assert/strict"
import { createHash } from "node:crypto"
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"
import test from "node:test"

import {
  parseFrontmatter,
  validateLedger,
  validateRepository,
} from "./validate-specos-issues.mjs"

function hash(raw) {
  return createHash("sha256").update(raw).digest("hex")
}

function fixture() {
  const specRaw = '---\nid: FEATURE-001\nversion: "1"\nstatus: approved\n---\n'
  const specHash = hash(specRaw)
  const spec = {
    id: "FEATURE-001",
    version: "1",
    status: "approved",
    hash: specHash,
    file: ".features/FEATURE-001/spec.md",
    testFile: ".features/FEATURE-001/test-spec.md",
    testSpec: {
      status: "approved",
      sourceSpecId: "FEATURE-001",
      sourceSpecVersion: "1",
      sourceSpecHash: specHash,
    },
  }
  const base = {
    sourceSpecId: "FEATURE-001",
    sourceSpecVersion: "1",
    sourceSpecHash: specHash,
    requirements: [],
    supersededBy: [],
  }
  return {
    specs: new Map([[spec.id, spec]]),
    issues: [
      {
        ...base,
        id: "issue-001",
        filename: "issue-001-implementation.md",
        file: ".issues/issue-001-implementation.md",
        status: "verified",
        kind: "implementation",
        dependsOn: [],
      },
      {
        ...base,
        id: "issue-002",
        filename: "issue-002-verification.md",
        file: ".issues/issue-002-verification.md",
        status: "verified",
        kind: "verification",
        dependsOn: ["issue-001"],
      },
    ],
    implementationNotes: [],
  }
}

test("parses scalar and inline-list frontmatter", () => {
  assert.deepEqual(
    parseFrontmatter(
      '---\nid: issue-001\ndependsOn: [issue-002, "issue-003"]\n---\n'
    ),
    { id: "issue-001", dependsOn: ["issue-002", "issue-003"] }
  )
})

test("accepts a dense, source-bound, terminal ledger", () => {
  assert.deepEqual(validateLedger(fixture()), [])
})

test("reports numbering, reference, cycle, hash and note errors", () => {
  const data = fixture()
  data.issues[0].sourceSpecHash = "stale"
  data.issues[0].dependsOn = ["issue-002", "issue-999"]
  data.issues[1].filename = "issue-003-verification.md"
  data.implementationNotes.push("issue#0001.html")

  const errors = validateLedger(data).join("\n")
  assert.match(errors, /frontmatter id must be issue-003/)
  assert.match(errors, /undeclared gap at issue-002/)
  assert.match(errors, /references missing issue-999/)
  assert.match(errors, /source Spec hash does not match/)
  assert.match(errors, /active dependency cycle/)
  assert.match(errors, /Completion Record/)
})

test("repository loading reports malformed Issue filenames", async (t) => {
  const root = await mkdtemp(path.join(tmpdir(), "bugrail-specos-"))
  t.after(() => rm(root, { recursive: true, force: true }))
  await Promise.all(
    [".issues", ".features", "docs"].map((directory) =>
      mkdir(path.join(root, directory))
    )
  )
  await writeFile(
    path.join(root, ".issues/issue-1-invalid.md"),
    "---\nid: issue-001\nstatus: planned\nkind: implementation\n" +
      'sourceSpecId: FEATURE-001\nsourceSpecVersion: "1"\n' +
      "sourceSpecHash: stale\ndependsOn: []\n---\n"
  )

  const errors = await validateRepository(root)
  assert.ok(errors.some((error) => error.includes("invalid Issue filename")))
})

test("verified verification Issues require verified direct dependencies", () => {
  const data = fixture()
  data.issues[0].status = "implemented_pending_verification"
  const errors = validateLedger(data)
  assert.ok(
    errors.some((error) => error.includes("depends on non-verified issue-001"))
  )
})
