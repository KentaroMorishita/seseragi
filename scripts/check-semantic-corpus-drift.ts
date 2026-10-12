/** Compare actual TS checker interpretation under both locked compiler versions. */
import { createHash } from "node:crypto"
import { existsSync, readFileSync, writeFileSync } from "node:fs"
import { createRequire } from "node:module"
import path from "node:path"
import { fileURLToPath } from "node:url"
import type * as TypeScript from "typescript"

const root = path.resolve(fileURLToPath(new URL("..", import.meta.url)))
const names = ["domains", "relations", "structures", "callables"]
const digest = (bytes: Uint8Array | string) =>
  createHash("sha256").update(bytes).digest("hex")
const require = createRequire(import.meta.url)
const snapshots: unknown[] = []
for (const location of [
  "node_modules/typescript",
  "apps/playground/node_modules/typescript",
]) {
  const compiler = path.join(root, location, "lib/typescript.js")
  if (!existsSync(compiler))
    throw new Error(
      `Bootstrap frozen dependencies before corpus drift check: ${location}`
    )
  const ts: typeof TypeScript = require(compiler)
  const fixtures: Record<string, unknown> = {}
  for (const name of names) {
    const file = path.join(
      root,
      `examples/spec/fixtures/semantic-interop-schema-1/${name}.d.ts`
    )
    const program = ts.createProgram([file], {
      strict: true,
      noEmit: true,
      skipLibCheck: false,
      target: ts.ScriptTarget.ES2022,
    })
    const source = program.getSourceFile(file)!
    const checker = program.getTypeChecker()
    const module = checker.getSymbolAtLocation(source)!
    fixtures[name] = {
      sourceDigest: digest(readFileSync(file)),
      exports: checker
        .getExportsOfModule(module)
        .map((symbol) => ({
          name: symbol.name,
          type: checker.typeToString(
            (symbol.flags & ts.SymbolFlags.Type) !== 0
              ? checker.getDeclaredTypeOfSymbol(symbol)
              : checker.getTypeOfSymbolAtLocation(symbol, source),
            source,
            ts.TypeFormatFlags.NoTruncation | ts.TypeFormatFlags.InTypeAlias
          ),
          flags: symbol.flags,
        }))
        .sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0)),
      diagnostics: ts
        .getPreEmitDiagnostics(program)
        .filter((diagnostic) => diagnostic.file?.fileName === file)
        .map((diagnostic) => ({
          code: diagnostic.code,
          start: diagnostic.start,
          length: diagnostic.length,
          message: ts.flattenDiagnosticMessageText(
            diagnostic.messageText,
            "\n"
          ),
        })),
    }
  }
  snapshots.push({
    version: ts.version,
    compilerDigest: digest(readFileSync(compiler)),
    fixtures,
  })
}
const actual = {
  schema: 1,
  kind: "seseragi-typescript-checker-drift",
  compilers: snapshots,
}
const golden = path.join(
  root,
  "examples/spec/artifacts/semantic-interop-schema-1/typescript-checker.json"
)
const serialized = `${JSON.stringify(actual, null, 2)}\n`
if (process.argv.includes("--write")) writeFileSync(golden, serialized)
if (!existsSync(golden) || readFileSync(golden, "utf8") !== serialized)
  throw new Error(
    "TypeScript declaration interpretation/toolchain drift: review exports, type relations, diagnostics and compiler digests before explicitly refreshing the snapshot"
  )
console.log(
  "Semantic corpus: TypeScript checker versions and declaration interpretation match reviewed snapshots"
)
