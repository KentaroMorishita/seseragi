import assert from "node:assert/strict"
import path from "node:path"
import { pathToFileURL } from "node:url"
import ts from "typescript"

const input = process.argv[2]
assert(input, "expected an emitted TypeScript Char fixture")
const program = ts.createProgram([input], {
  noEmit: true,
  strict: true,
  skipLibCheck: true,
  target: ts.ScriptTarget.ES2022,
  module: ts.ModuleKind.ESNext,
  moduleResolution: ts.ModuleResolutionKind.Bundler,
})
const diagnostics = ts.getPreEmitDiagnostics(program)
assert.equal(
  diagnostics.length,
  0,
  ts.formatDiagnosticsWithColorAndContext(diagnostics, {
    getCanonicalFileName: (name) => name,
    getCurrentDirectory: () => process.cwd(),
    getNewLine: () => "\n",
  })
)
const built = await Bun.build({
  entrypoints: [input],
  target: "node",
  format: "esm",
})
assert(built.success, built.logs.map(String).join("\n"))
assert.equal(built.outputs.length, 1)
const output = path.join(path.dirname(input), "char-module.mjs")
await Bun.write(output, built.outputs[0])
const harness = path.resolve("scripts/probes/char-probe.mjs")
let expected: unknown
for (const runtime of [process.execPath, "node"]) {
  const result = Bun.spawnSync([runtime, harness, pathToFileURL(output).href], {
    stdout: "pipe",
    stderr: "pipe",
  })
  assert(result.success, result.stderr.toString())
  const report = JSON.parse(result.stdout.toString())
  if (expected !== undefined) assert.deepEqual(report, expected)
  expected = report
  console.log(`${runtime}: ${JSON.stringify(report)}`)
}
