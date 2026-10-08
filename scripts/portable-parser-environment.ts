import { readFileSync } from "node:fs"
import { resolve } from "node:path"

export function portableParserEnvironment(root: string) {
  const sdk = process.env.WASI_SDK_PATH
  if (
    !sdk ||
    readFileSync(resolve(sdk, "VERSION"), "utf8").split("\n")[0] !== "25.0"
  ) {
    throw new Error(
      "WASI SDK 25.0 is required; set WASI_SDK_PATH to its extracted directory (see docs/SCOPED_CHECKS.md)"
    )
  }
  const compiler = resolve(
    sdk,
    "bin",
    process.platform === "win32" ? "clang.exe" : "clang"
  )
  const version = Bun.spawnSync([compiler, "--version"], {
    stdout: "pipe",
    stderr: "pipe",
  })
  if (
    version.exitCode !== 0 ||
    !version.stdout.toString().includes("19.1.5-wasi-sdk")
  ) {
    throw new Error(
      "WASI SDK 25.0 clang 19.1.5 is required for reproducible parser artifacts"
    )
  }
  const metadataProcess = Bun.spawnSync(
    ["cargo", "metadata", "--locked", "--format-version", "1"],
    { cwd: root, stdout: "pipe", stderr: "inherit" }
  )
  if (metadataProcess.exitCode !== 0) throw new Error("cargo metadata failed")
  const metadata = JSON.parse(metadataProcess.stdout.toString()) as {
    target_directory: string
    packages: { name: string; version: string; manifest_path: string }[]
  }
  const language = metadata.packages.find(
    (dependency) => dependency.name === "tree-sitter-language"
  )
  if (!language)
    throw new Error("tree-sitter-language is missing from Cargo.lock")
  // The TypeScript grammar's upstream build script predates the Rust core's
  // wasm32 support. Use the same portable C headers as tree-sitter itself.
  const headers = resolve(language.manifest_path, "..", "wasm", "include")
  const environment = {
    ...process.env,
    CC_SHELL_ESCAPED_FLAGS: "1",
    "CC_wasm32-unknown-unknown": compiler,
    "CFLAGS_wasm32-unknown-unknown": [
      process.env["CFLAGS_wasm32-unknown-unknown"] ??
        process.env.CFLAGS_wasm32_unknown_unknown,
      `-I${JSON.stringify(headers)}`,
    ]
      .filter(Boolean)
      .join(" "),
  }

  return { environment, metadata }
}
