import { join, resolve } from "node:path"
import { portableParserEnvironment } from "./portable-parser-environment"

const root = resolve(import.meta.dir, "..")
const { environment, metadata } = portableParserEnvironment(root)
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
