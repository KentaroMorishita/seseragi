import { copyFile, mkdir } from "node:fs/promises"
import { dirname, join, resolve } from "node:path"

const target = process.argv[2]
const output = process.argv[3]
if (!target || !output)
  throw new Error("Expected target and evidence directories")
const patterns = [
  "wasm32-unknown-unknown/release/seseragi_wasm.wasm",
  "wasm32-unknown-unknown/release/.fingerprint/*/*.json",
  "release/.fingerprint/*/*.json",
  "wasm32-unknown-unknown/release/build/*/output",
  "wasm32-unknown-unknown/release/build/tree-sitter-*/out/*.{o,a}",
  "wasm32-unknown-unknown/release/build/tree-sitter-typescript-*/out/*.{o,a}",
]
for (const pattern of patterns) {
  for await (const path of new Bun.Glob(pattern).scan({
    cwd: target,
    dot: true,
  })) {
    const destination = join(resolve(output), "compiler", path)
    await mkdir(dirname(destination), { recursive: true })
    await copyFile(join(target, path), destination)
  }
}
