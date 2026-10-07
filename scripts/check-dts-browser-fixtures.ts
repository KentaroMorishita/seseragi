import { deepStrictEqual } from "node:assert"
import { readdir } from "node:fs/promises"
import { join, resolve } from "node:path"
import init, {
  convert_workspace_bindings,
} from "../apps/playground/src/wasm/pkg/seseragi_wasm.js"

const root = resolve(import.meta.dir, "..")
const output = process.argv[2]
if (!output) throw new Error("Expected a conversion-results output path")
await init({
  module_or_path: await Bun.file(
    join(root, "apps/playground/src/wasm/pkg/seseragi_wasm_bg.wasm")
  ).arrayBuffer(),
})

const results: Record<string, unknown> = {}
for (const name of [
  "dts-basic-conversion",
  "dts-callback-during-call",
  "dts-declaration-merge",
  "dts-generated-name",
  "dts-namespace-runtime",
  "dts-overload-selection",
  "dts-callback-missing-release",
  "dts-unsupported-any",
]) {
  const directory = join(root, "examples/spec/fixtures/projects", name)
  const files: { path: string; source: string }[] = []
  async function collect(relative = "") {
    const entries = await readdir(join(directory, relative), {
      withFileTypes: true,
    })
    entries.sort((left, right) =>
      left.name < right.name ? -1 : left.name > right.name ? 1 : 0
    )
    for (const entry of entries) {
      const path = relative ? `${relative}/${entry.name}` : entry.name
      if (entry.isDirectory()) await collect(path)
      else
        files.push({
          path,
          source: await Bun.file(join(directory, path)).text(),
        })
    }
  }
  await collect()
  const response = JSON.parse(
    convert_workspace_bindings(
      JSON.stringify({ schema: 1, revision: "cross-host-fixture", files })
    )
  )
  const expectedStatus = [
    "dts-callback-missing-release",
    "dts-unsupported-any",
  ].includes(name)
    ? "failure"
    : "success"
  deepStrictEqual(response.status, expectedStatus, name)
  results[name] = response
}
await Bun.write(output, `${JSON.stringify(results, null, 2)}\n`)
console.log("Recorded all eight browser converter fixture results")
