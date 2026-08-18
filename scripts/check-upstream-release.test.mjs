import assert from "node:assert/strict"
import test from "node:test"

import {
  compareReleaseTags,
  evaluateUpstreamRelease,
  peelGitHubTagObject,
} from "./check-upstream-release.mjs"

test("compares semantic release tags", () => {
  assert.equal(compareReleaseTags("v0.23.3", "v0.23.2"), 1)
  assert.equal(compareReleaseTags("v0.23.2", "v0.23.2"), 0)
  assert.equal(compareReleaseTags("v0.23.1", "v0.23.2"), -1)
})

test("peels an annotated GitHub tag to its immutable commit", async () => {
  const commit = await peelGitHubTagObject(
    { type: "tag", sha: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" },
    async () => ({
      object: {
        type: "commit",
        sha: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      },
    })
  )

  assert.equal(commit, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
})

test("reports an available immutable upstream release", () => {
  assert.deepEqual(
    evaluateUpstreamRelease({
      baselineTag: "v0.23.2",
      baselineCommit: "159f68e42e6b9d81d9135d47a3879033446b824d",
      latestTag: "v0.24.0",
      latestCommit: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    }),
    {
      status: "update-available",
      latestTag: "v0.24.0",
      latestCommit: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    }
  )
})
