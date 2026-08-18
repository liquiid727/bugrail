interface DevOriginEnv {
  NODE_ENV?: string
  TAURI_DEV_HOST?: string
}

export function resolveDevAssetPrefix(env: DevOriginEnv): string | undefined {
  if (env.NODE_ENV === "production" || !env.TAURI_DEV_HOST) return undefined
  return `http://${env.TAURI_DEV_HOST}:3000`
}
