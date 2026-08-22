import { describe, expect, it } from "vitest"

import { PRODUCT_MANIFEST, formatProductTitle } from "./product-manifest"

describe("Bugrail product manifest", () => {
  it("owns the user-facing product identity", () => {
    expect(PRODUCT_MANIFEST.displayName).toBe("Bugrail")
    expect(PRODUCT_MANIFEST.version).toMatch(/^\d+\.\d+\.\d+/)
    expect(PRODUCT_MANIFEST.repositoryUrl).toBe(
      "https://github.com/liquiid727/bugrail"
    )
    expect(PRODUCT_MANIFEST.releasesUrl).toBe(
      "https://github.com/liquiid727/bugrail/releases"
    )
  })

  it("formats bare and contextual document titles consistently", () => {
    expect(formatProductTitle()).toBe("Bugrail")
    expect(formatProductTitle("specops")).toBe("specops - Bugrail")
  })
})
