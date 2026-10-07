import { join, resolve } from "node:path"

const root = resolve(import.meta.dir, "..")
const metadataProcess = Bun.spawnSync(
  ["cargo", "metadata", "--locked", "--format-version", "1"],
  { cwd: root, stdout: "pipe", stderr: "inherit" }
)
if (metadataProcess.exitCode !== 0) process.exit(metadataProcess.exitCode)
const metadata = JSON.parse(metadataProcess.stdout.toString()) as {
  target_directory: string
  packages: { name: string; manifest_path: string }[]
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
  "CFLAGS_wasm32-unknown-unknown": [
    process.env["CFLAGS_wasm32-unknown-unknown"] ??
      process.env.CFLAGS_wasm32_unknown_unknown,
    `-I${JSON.stringify(headers)}`,
  ]
    .filter(Boolean)
    .join(" "),
}
const build = Bun.spawnSync(
  [
    "cargo",
    "build",
    "--locked",
    "-p",
    "seseragi-dts",
    "--no-default-features",
    "--target",
    "wasm32-unknown-unknown",
    "--example",
    "portable-wasm",
  ],
  { cwd: root, env: environment, stdout: "inherit", stderr: "inherit" }
)
if (build.exitCode !== 0) process.exit(build.exitCode)
const path = join(
  metadata.target_directory,
  "wasm32-unknown-unknown/debug/examples/portable_wasm.wasm"
)
const module = await WebAssembly.compile(await Bun.file(path).arrayBuffer())
const imports = WebAssembly.Module.imports(module)
if (imports.length !== 0) {
  throw new Error(
    `Portable converter requires host imports: ${JSON.stringify(imports)}`
  )
}
const instance = await WebAssembly.instantiate(module, {})
const smoke = instance.exports.conversion_smoke as () => number
if (smoke() !== 1) throw new Error("Portable converter smoke failed")
console.log(
  "Portable .d.ts WASM: conversion, metadata, regeneration and diagnostics passed with no host imports"
)
