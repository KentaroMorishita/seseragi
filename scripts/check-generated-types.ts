import { readdirSync } from "node:fs"
import { join, resolve } from "node:path"
import ts from "typescript"

// Validate the generated program against the staged runtime types. Diagnostics
// are requested for output files, so independent runtime implementation checks
// remain in their own gate instead of hiding generated-code errors in that noise.
export function checkGeneratedTypes(directory: string): void {
  const files: string[] = []
  function collect(path: string) {
    for (const item of readdirSync(path, { withFileTypes: true })) {
      if (item.name === "node_modules") continue
      const child = join(path, item.name)
      if (item.isDirectory()) collect(child)
      else if (item.name.endsWith(".ts")) files.push(child)
    }
  }
  collect(resolve(directory))
  if (files.length === 0) throw new Error("no generated TypeScript files")
  const program = ts.createProgram(files, {
    strict: true,
    noEmit: true,
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.Preserve,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    allowImportingTsExtensions: true,
    skipLibCheck: true,
  })
  const diagnostics = files.flatMap((path) => {
    const source = program.getSourceFile(path)
    if (!source) throw new Error(`missing generated source ${path}`)
    return [
      ...program.getSyntacticDiagnostics(source),
      ...program.getSemanticDiagnostics(source),
    ]
  })
  if (diagnostics.length) {
    throw new Error(
      ts.formatDiagnostics(diagnostics, {
        getCanonicalFileName: (path) => path,
        getCurrentDirectory: () => directory,
        getNewLine: () => "\n",
      })
    )
  }
}

if (import.meta.main) {
  const directory = process.argv[2]
  if (!directory) throw new Error("usage: check-generated-types.ts OUTPUT_DIR")
  checkGeneratedTypes(directory)
  console.log("generated TypeScript: strict check passed")
}
