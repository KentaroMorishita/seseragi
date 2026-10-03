import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { join, resolve } from "node:path"
import { pathToFileURL } from "node:url"
import type {
  BindingConversionRequest,
  BindingConversionResponse,
} from "../apps/playground/src/compiler/interop-types"

// Build with scripts/build-playground-wasm.sh <temporary-package-directory>
// first. This probe also works against the canonical committed package.
const root = resolve(import.meta.dir, "..")
const directory = resolve(
  root,
  process.argv[2] ?? "apps/playground/src/wasm/pkg"
)
const wasm = await import(
  pathToFileURL(join(directory, "seseragi_wasm.js")).href
)
const bytes = await readFile(join(directory, "seseragi_wasm_bg.wasm"))
const module = await WebAssembly.compile(bytes)
assert(
  !WebAssembly.Module.imports(module).some((item) =>
    /wasi|filesystem/i.test(`${item.module}/${item.name}`)
  ),
  "the browser adapter must not require filesystem imports"
)
await wasm.default({ module_or_path: bytes })
const fixture = join(
  root,
  "examples/spec/fixtures/projects/dts-basic-conversion"
)
const seed = JSON.parse(
  await readFile(
    join(root, "examples/spec/fixtures/playground/binding-workspace.json"),
    "utf8"
  )
) as { files: { path: string; source: string }[] }
const seedManifest = seed.files.find((file) => file.path === "seseragi.toml")
assert(seedManifest)
const request: BindingConversionRequest = {
  schema: 1,
  revision: "opaque workspace revision: 日本語/1",
  manifest: seedManifest.source,
  files: structuredClone(seed.files),
}
const convert = (
  request: BindingConversionRequest
): BindingConversionResponse =>
  JSON.parse(wasm.convert_bindings(JSON.stringify(request)))
const converted = convert(request)
assert.equal(converted.status, "success", JSON.stringify(converted))
assert.equal(converted.revision, request.revision)
assert.equal(converted.generated.length, 1)
assert.equal(
  converted.generated[0].source,
  await readFile(join(fixture, "expected/fixture-api.ssrg"), "utf8")
)
const generated = converted.generated[0]
const project = {
  schema: 1,
  manifest: request.manifest,
  files: [
    {
      path: "main.ssrg",
      source:
        'import { Config } from "gen/fixture-api"\npub fn accepts config: Config -> Unit = ()\npub effect fn main -> Unit = succeed ()\n',
    },
    {
      path: "gen/fixture-api.ssrg",
      root: "generated",
      source: generated.source,
    },
  ],
}
const seedProject = {
  ...project,
  files: [
    ...seed.files.filter((file) => file.path.endsWith(".ssrg")),
    project.files[1],
  ],
}
for (const operation of [wasm.compile_project, wasm.analyze_project]) {
  const seedResult = JSON.parse(operation(JSON.stringify(seedProject)))
  assert.equal(seedResult.status, "success", JSON.stringify(seedResult))
  const result = JSON.parse(operation(JSON.stringify(project)))
  assert.equal(result.status, "success", JSON.stringify(result))
}
request.files.push({
  path: "gen/fixture-api.binding.json",
  source: generated.metadata,
})
const repeated = convert(request)
assert.equal(repeated.status, "success")
const report = JSON.parse(repeated.generated[0].report)
for (const change of ["added", "changed", "removed"])
  assert.deepEqual(report[change], [])

request.revision = "changed"
const declaration = request.files.find(
  (file) => file.path === "host/index.d.ts"
)
assert(declaration)
declaration.source = "export declare function unsafe(value: any): unknown;"
const rejected = convert(request)
assert.equal(rejected.status, "failure")
assert.equal(rejected.revision, "changed")
assert.deepEqual(rejected.generated, [])
assert.equal(rejected.diagnostics[0].entry, "api")
assert.equal(rejected.diagnostics[0].path, "host/index.d.ts")

project.files[1].source = "pub let replacement = 1\n"
for (const operation of [wasm.compile_project, wasm.analyze_project]) {
  const result = JSON.parse(operation(JSON.stringify(project)))
  assert.equal(result.status, "failure", JSON.stringify(result))
}
console.log(
  "Production WASM: workspace declaration conversion, generated imports, analysis, report reuse, diagnostics and regeneration passed."
)
