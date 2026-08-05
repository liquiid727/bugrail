import { spawn } from "node:child_process"

const nodeOptions = [process.env.NODE_OPTIONS, "--no-experimental-webstorage"]
  .filter(Boolean)
  .join(" ")

const child = spawn(
  process.execPath,
  ["./node_modules/vitest/vitest.mjs", ...process.argv.slice(2)],
  {
    env: { ...process.env, NODE_OPTIONS: nodeOptions },
    stdio: "inherit",
  }
)

child.on("error", (error) => {
  console.error(error)
  process.exitCode = 1
})

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal)
    return
  }
  process.exitCode = code ?? 1
})
