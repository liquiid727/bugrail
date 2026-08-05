export interface ProductManifest {
  displayName: string
  bundleName: string
  description: string
  repositoryUrl: string
  releasesUrl: string
  latestReleaseUrl: string
  updaterManifestUrl: string
}

export const PRODUCT_MANIFEST: Readonly<ProductManifest> = Object.freeze({
  displayName: "Code: Bugrail",
  bundleName: "Bugrail",
  description: "Spec-driven AI coding workspace",
  repositoryUrl: "https://github.com/liquiid727/bugrail",
  releasesUrl: "https://github.com/liquiid727/bugrail/releases",
  latestReleaseUrl: "https://github.com/liquiid727/bugrail/releases/latest",
  updaterManifestUrl:
    "https://github.com/liquiid727/bugrail/releases/latest/download/latest.json",
})

export function formatProductTitle(context?: string | null): string {
  const normalizedContext = context?.trim()
  return normalizedContext
    ? `${normalizedContext} - ${PRODUCT_MANIFEST.displayName}`
    : PRODUCT_MANIFEST.displayName
}
