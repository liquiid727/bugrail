import releaseManifest from "../../release/manifest.json"

export interface ProductManifest {
  displayName: string
  bundleName: string
  version: string
  description: string
  repositoryUrl: string
  releasesUrl: string
  latestReleaseUrl: string
  updaterManifestUrl: string
}

export const PRODUCT_MANIFEST: Readonly<ProductManifest> = Object.freeze({
  displayName: releaseManifest.product.name,
  bundleName: releaseManifest.product.name,
  version: releaseManifest.version,
  description: "Spec-driven AI coding workspace",
  repositoryUrl: releaseManifest.release.repositoryUrl,
  releasesUrl: releaseManifest.release.releasesUrl,
  latestReleaseUrl: releaseManifest.release.latestReleaseUrl,
  updaterManifestUrl: releaseManifest.release.updaterManifestUrl,
})

export function formatProductTitle(context?: string | null): string {
  const normalizedContext = context?.trim()
  return normalizedContext
    ? `${normalizedContext} - ${PRODUCT_MANIFEST.displayName}`
    : PRODUCT_MANIFEST.displayName
}
