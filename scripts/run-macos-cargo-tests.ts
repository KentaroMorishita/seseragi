import { spawnSync } from "node:child_process"
import {
  chmodSync,
  copyFileSync,
  mkdtempSync,
  readFileSync,
  rmSync,
} from "node:fs"
import { tmpdir } from "node:os"
import { basename, join } from "node:path"

const manifestPath = process.argv[2]
if (!manifestPath) {
  throw new Error("cargo artifact manifest path is required")
}

const executables = new Set<string>()
for (const line of readFileSync(manifestPath, "utf8").split("\n")) {
  if (line === "") continue

  const message = JSON.parse(line) as {
    executable?: string | null
    profile?: { test?: boolean }
    reason?: string
  }
  if (
    message.reason === "compiler-artifact" &&
    message.profile?.test === true &&
    message.executable
  ) {
    executables.add(message.executable)
  }
}

const run = (command: string, args: string[]): void => {
  const result = spawnSync(command, args, { stdio: "inherit" })
  if (result.error) throw result.error
  if (result.status !== 0) {
    throw new Error(
      `${command} exited with status ${result.status ?? "unknown"}`
    )
  }
}

const sign = (executable: string): void => {
  const result = spawnSync(
    "codesign",
    ["--force", "--sign", "-", "--timestamp=none", executable],
    { encoding: "utf8" }
  )
  if (result.error) throw result.error
  if (result.status !== 0) {
    throw new Error(
      `failed to sign ${executable}: ${result.stderr ?? "unknown error"}`
    )
  }
}

const directory = mkdtempSync(join(tmpdir(), "seseragi-cargo-tests-"))
try {
  let index = 0
  for (const executable of executables) {
    const copy = join(directory, `${index}-${basename(executable)}`)
    index += 1
    copyFileSync(executable, copy)
    chmodSync(copy, 0o755)
    sign(copy)
    run(copy, [])
  }
} finally {
  rmSync(directory, { force: true, recursive: true })
}

console.log(`Ran ${executables.size} signed macOS Cargo test artifacts.`)
