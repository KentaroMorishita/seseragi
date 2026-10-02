import assert from "node:assert/strict"
import { readFileSync, writeFileSync } from "node:fs"
import ts from "typescript"

const configPath = process.argv[2]
assert(configPath, "Expected a compiler API check configuration")
const config = JSON.parse(readFileSync(configPath, "utf8")) as {
  paths: string[]
  typeRoots: string[]
  result: string
  runtimeProbe: boolean
  expectedCode?: number
}
const options: ts.CompilerOptions = {
  target: ts.ScriptTarget.ES2022,
  module: config.runtimeProbe ? ts.ModuleKind.ESNext : ts.ModuleKind.NodeNext,
  moduleResolution: config.runtimeProbe
    ? ts.ModuleResolutionKind.Bundler
    : ts.ModuleResolutionKind.NodeNext,
  strict: true,
  noUncheckedIndexedAccess: true,
  exactOptionalPropertyTypes: true,
  skipLibCheck: false,
  noEmit: true,
  allowImportingTsExtensions: true,
  types: ["node"],
  typeRoots: config.typeRoots,
}
const program = ts.createProgram(config.paths, options)
const diagnostics = ts.getPreEmitDiagnostics(program).map((diagnostic) => ({
  code: diagnostic.code,
  file: diagnostic.file?.fileName,
  start: diagnostic.start,
  length: diagnostic.length,
  message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
}))
writeFileSync(
  config.result,
  `${JSON.stringify({ version: ts.version, options, paths: config.paths, diagnostics }, null, 2)}\n`
)
if (config.expectedCode === undefined) assert.deepEqual(diagnostics, [])
else {
  assert.equal(diagnostics.length, 1)
  assert.equal(diagnostics[0]?.code, config.expectedCode)
  assert.equal(diagnostics[0]?.file, config.paths[0])
}
