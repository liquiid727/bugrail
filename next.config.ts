import { fileURLToPath } from "node:url"
import type { NextConfig } from "next"
import createNextIntlPlugin from "next-intl/plugin"
import { resolveDevAssetPrefix } from "./src/lib/dev-origin"

const projectRoot = fileURLToPath(new URL(".", import.meta.url))

const withNextIntl = createNextIntlPlugin({
  requestConfig: "./src/i18n/request.ts",
  experimental: {
    messages: {
      path: "./src/i18n/messages",
      format: "json",
      locales: [
        "en",
        "zh-CN",
        "zh-TW",
        "ja",
        "ko",
        "es",
        "de",
        "fr",
        "pt",
        "ar",
      ],
      precompile: true,
    },
  },
})

const nextConfig: NextConfig = {
  output: "export",
  turbopack: {
    root: projectRoot,
  },
  images: {
    unoptimized: true,
  },
  assetPrefix: resolveDevAssetPrefix(process.env),
}

export default withNextIntl(nextConfig)
