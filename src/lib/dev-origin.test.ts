import { describe, expect, it } from "vitest"

import { resolveDevAssetPrefix } from "./dev-origin"

describe("resolveDevAssetPrefix", () => {
  it("leaves ordinary web previews on their requested origin", () => {
    expect(resolveDevAssetPrefix({ NODE_ENV: "development" })).toBeUndefined()
  })

  it("keeps Tauri development assets on the configured internal host", () => {
    expect(
      resolveDevAssetPrefix({
        NODE_ENV: "development",
        TAURI_DEV_HOST: "192.168.1.12",
      })
    ).toBe("http://192.168.1.12:3000")
  })

  it("never prefixes production exports", () => {
    expect(
      resolveDevAssetPrefix({
        NODE_ENV: "production",
        TAURI_DEV_HOST: "192.168.1.12",
      })
    ).toBeUndefined()
  })
})
