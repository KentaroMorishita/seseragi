import assert from "node:assert/strict"
import { createHash } from "node:crypto"
import { readFileSync, writeFileSync } from "node:fs"
import { join, resolve } from "node:path"
import ts from "typescript"
import { createBrowserEnvironment } from "../../../../../runtime/ts/src/browser/host.ts"
import {
  createEffectExecution,
  run,
} from "../../../../../runtime/ts/src/effect.ts"
import { executeGeneratedModule } from "../../../../playground/src/runtime/browser-execution.ts"
import { runtimeModules } from "../../../../playground/src/runtime/runtime-modules.ts"
import init, {
  compile_single_file,
  toolchain_version_json,
} from "../../../../playground/src/wasm/pkg/seseragi_wasm.js"
import {
  collectionTransformCases,
  collectionTransformExamples,
} from "../../../scripts/collection-transform.ts"

const destination = process.argv[2]
assert(destination, "Expected evidence destination")
const root = resolve(import.meta.dir, "../../../../..")
const sha256 = (value: string | Buffer) =>
  createHash("sha256").update(value).digest("hex")
const wasm = readFileSync(
  join(root, "apps/playground/src/wasm/pkg/seseragi_wasm_bg.wasm")
)
await init({ module_or_path: wasm })
const examples = collectionTransformExamples("https://seseragi.vercel.app/")
const results = []
for (const item of collectionTransformCases) {
  const example = examples.find(
    (value) => value.id === `collection-transform-${item.slug}`
  )!
  const source = new URL(example.playgroundUrl).searchParams.get("source")!
  assert.equal(source, readFileSync(join(root, example.sourcePath), "utf8"))
  assert.equal(sha256(source), example.sha256)
  const compiled = JSON.parse(
    compile_single_file("main.ssrg", "playground/main", source)
  )
  assert.equal(compiled.status, "success", JSON.stringify(compiled.diagnostics))
  assert.deepEqual(compiled.entry.environment, [
    { field: "console", service: "console" },
  ])
  assert.deepEqual(compiled.entry.providers ?? [], [])
  const displayed = await executeGeneratedModule(
    compiled.generated.typescript,
    compiled.entry,
    ""
  )
  assert.deepEqual(displayed, { stdout: item.output.trimEnd(), debug: "()" })
  // The production display trims final whitespace; verify raw bytes separately.
  const javascript = ts.transpileModule(compiled.generated.typescript, {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS,
    },
  }).outputText
  const module = { exports: {} as Record<string, unknown> }
  new Function("require", "module", "exports", javascript)(
    (specifier: string) => {
      const resolved = (runtimeModules as Record<string, unknown>)[specifier]
      assert.notEqual(
        resolved,
        undefined,
        `Unsupported runtime import: ${specifier}`
      )
      return resolved
    },
    module,
    module.exports
  )
  const chunks: string[] = []
  const execution = createEffectExecution()
  const environment = createBrowserEnvironment(
    compiled.entry.environment,
    "",
    (text) => chunks.push(text),
    undefined,
    execution.context
  )
  const main = module.exports.main as (
    unit: undefined
  ) => Parameters<typeof run>[0]
  try {
    const status = await run(main(undefined), environment, execution.context)
    assert.equal(status.kind, "success")
    assert.equal(chunks.join(""), item.output)
  } finally {
    await execution.close()
  }
  results.push({
    slug: item.slug,
    sourceSha256: sha256(source),
    generatedTypeScriptSha256: sha256(compiled.generated.typescript),
    entry: compiled.entry,
    displayed,
    rawStdout: chunks.join(""),
    rawStdoutSha256: sha256(chunks.join("")),
    playgroundUrl: example.playgroundUrl,
  })
}
const sourcePaths = [
  "apps/playground/src/wasm/pkg/seseragi_wasm.js",
  "apps/playground/src/runtime/browser-execution.ts",
  "apps/playground/src/runtime/runtime-modules.ts",
  "runtime/ts/src/browser/host.ts",
  "runtime/ts/src/array.ts",
  "runtime/ts/src/list.ts",
]
writeFileSync(
  join(destination, "wasm-seeds.json"),
  `${JSON.stringify(
    {
      scope:
        "Exact decoded final source seeds through committed WASM and the unmodified browser execution/host implementation under Bun; this is not an actual browser UI or public Playground observation",
      version: JSON.parse(toolchain_version_json()),
      wasmSha256: sha256(wasm),
      sourceHashes: sourcePaths.map((path) => ({
        path,
        sha256: sha256(readFileSync(join(root, path))),
      })),
      results,
    },
    null,
    2
  )}\n`
)
