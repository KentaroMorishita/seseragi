/** Real npm declarations → the normal converter/shared semantic report, no handwritten API catalog. */
import { createHash } from "node:crypto"
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs"
import { createRequire } from "node:module"
import path from "node:path"
import { fileURLToPath } from "node:url"
import type * as TypeScript from "typescript"

const require = createRequire(import.meta.url)
const ts: typeof TypeScript = require("typescript")
const root = path.resolve(fileURLToPath(new URL("..", import.meta.url)))
const fixture = path.join(
  root,
  "examples/spec/fixtures/semantic-interop-schema-1/npm"
)
const args = process.argv.slice(2)
let modules = path.join(fixture, "node_modules")
let write = false
for (let index = 0; index < args.length; index++) {
  if (args[index] === "--write") write = true
  else if (args[index] === "--node-modules" && args[index + 1])
    modules = path.resolve(args[++index]!)
  else throw new Error(`Unknown real package corpus option ${args[index]}`)
}
const cases = JSON.parse(
  readFileSync(path.join(fixture, "cases.json"), "utf8")
) as {
  package: string
  declaration: string
  specifier: string
  domains: string[]
  targets: string[]
}[]
const lock = JSON.parse(
  readFileSync(path.join(fixture, "bun.lock"), "utf8").replace(
    /,(\s*[}\]])/g,
    "$1"
  )
) as { packages: Record<string, [string, string, unknown, string]> }
function run(cmd: string[]) {
  const process = Bun.spawnSync({
    cmd,
    cwd: root,
    stdout: "pipe",
    stderr: "pipe",
  })
  if (process.exitCode !== 0)
    throw new Error(`${cmd.join(" ")}: ${process.stderr.toString()}`)
  return process.stdout.toString()
}
if (!existsSync(path.join(modules, "nanoid/package.json")))
  throw new Error(
    `Install the pinned corpus explicitly: bun install --frozen-lockfile --ignore-scripts --cwd ${fixture}`
  )
run([
  "cargo",
  "build",
  "--locked",
  "--quiet",
  "-p",
  "seseragi-dts",
  "--example",
  "semantic-package-report",
])
const binary = path.join(
  root,
  "target/debug/examples/semantic-package-report" +
    (process.platform === "win32" ? ".exe" : "")
)
const destination = path.join(
  root,
  "examples/spec/artifacts/semantic-interop-schema-1/npm"
)
mkdirSync(destination, { recursive: true })
for (const item of cases) {
  const packageRoot = path.join(modules, item.package)
  const declaration = path.join(packageRoot, item.declaration)
  const manifest = path.join(packageRoot, "package.json")
  const metadata = JSON.parse(readFileSync(manifest, "utf8")) as {
    name: string
    version: string
  }
  const pinned = lock.packages[item.package]
  if (!pinned || pinned[0] !== `${metadata.name}@${metadata.version}`)
    throw new Error(
      `Real package corpus version differs from lock: ${item.package}`
    )
  const data = JSON.parse(run([binary, declaration, manifest, item.specifier]))
  const program = ts.createProgram([declaration], {
    strict: true,
    noEmit: true,
    skipLibCheck: true,
    target: ts.ScriptTarget.ES2022,
    moduleResolution: ts.ModuleResolutionKind.Node10,
  })
  const source = program.getSourceFile(declaration)!
  const checker = program.getTypeChecker()
  const module = checker.getSymbolAtLocation(source)
  const exports = module
    ? checker
        .getExportsOfModule(module)
        .map((symbol) => ({
          name: symbol.name,
          declarationNames: (symbol.declarations ?? []).flatMap(
            (declaration) => {
              const name = (declaration as TypeScript.NamedDeclaration).name
              return name && ts.isIdentifier(name) ? [name.text] : []
            }
          ),
        }))
        .sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0))
    : []
  const privateSymbols = new Set<string>(
    data.conversion.semantic?.privateSymbols ?? []
  )
  const known = new Set(
    Object.entries(data.conversion.semantic?.symbols ?? {})
      .filter(([id]) => !privateSymbols.has(id))
      .map(([, value]) => (value as { publicName: string }).publicName)
  )
  data.checkerProvenance = {
    version: ts.version,
    exports,
    missingPublicRoots: exports
      .filter((symbol) => !known.has(symbol.name))
      .map((symbol) => symbol.name),
    diagnostics: ts
      .getPreEmitDiagnostics(program)
      .filter((diagnostic) => diagnostic.file?.fileName === declaration)
      .map((diagnostic) => ({
        code: diagnostic.code,
        message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
      })),
  }
  if (data.checkerProvenance.missingPublicRoots.length > 0)
    data.surfaceStatus = "partially-interpreted-public-surface"
  data.registryProvenance = {
    package: metadata.name,
    version: metadata.version,
    integrity: pinned[3],
    declaration: item.declaration,
    declarationDigest: createHash("sha256")
      .update(readFileSync(declaration))
      .digest("hex"),
    domains: item.domains,
    targets: item.targets,
  }
  const expected = path.join(destination, `${item.package}.json`)
  const actual = `${JSON.stringify(data, null, 2)}\n`
  if (write) writeFileSync(expected, actual)
  if (!existsSync(expected) || readFileSync(expected, "utf8") !== actual)
    throw new Error(
      `Real package semantic/projection drift in ${item.package}; inspect reasons and registry provenance before --write`
    )
  console.log(
    `${metadata.name}@${metadata.version}: ${data.surfaceStatus}; ${Object.keys(data.interpretation?.symbols ?? {}).length} interpreted public symbols; generated=${data.conversion.generated !== null}`
  )
}
