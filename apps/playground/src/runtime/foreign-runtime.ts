import type { GeneratedProjectModule } from "./browser-execution"

export type ForeignRuntimeDiagnostic = Readonly<{
  code: string
  symbol?: string
  target?: "browser" | "node" | "bun"
  reason: string
}>
export type ForeignRuntimeModule = Readonly<{
  path: string
  source: string
  exactIdentity: string
  contentDigest: string
  imports: Readonly<Record<string, string>>
}>
export type ForeignRuntimeBundle = Readonly<{
  snapshotDigest: string
  /** Executable generated-module closure after TypeScript erases type/unused imports. */
  generatedPaths?: readonly string[]
  modules: Readonly<Record<string, ForeignRuntimeModule>>
  roots: Readonly<Record<string, Readonly<Record<string, string>>>>
}>
export type BrowserRuntimeRequest = Readonly<{
  schema: 1
  manifest?: string
  files: readonly Readonly<{ path: string; source: string }>[]
  imports: readonly Readonly<{ importer: string; specifier: string }>[]
}>
export type BrowserRuntimeResponse =
  | Readonly<{ status: "success"; schema: 1; bundle: ForeignRuntimeBundle }>
  | Readonly<{
      status: "failure"
      schema: 1
      diagnostics: readonly ForeignRuntimeDiagnostic[]
    }>
export class ForeignRuntimeError extends Error {
  readonly diagnostics: readonly ForeignRuntimeDiagnostic[]
  constructor(diagnostics: readonly ForeignRuntimeDiagnostic[]) {
    super(
      diagnostics
        .map(
          ({ code, target, reason }) =>
            `${code}${target === undefined ? "" : ` [${target}]`}: ${reason}`
        )
        .join("\n")
    )
    this.name = "ForeignRuntimeError"
    this.diagnostics = diagnostics
  }
}
function modulePath(path: string): string {
  return path.endsWith(".ssrg") ? `${path.slice(0, -5)}.js` : path
}
function relative(from: string, specifier: string): string | undefined {
  const parts = from.split("/")
  parts.pop()
  for (const part of specifier.split("/")) {
    if (part === "" || part === ".") continue
    if (part === "..") {
      if (parts.pop() === undefined) return undefined
    } else parts.push(part)
  }
  return parts.join("/").replace(/\.ts$/, ".js")
}
/** Generated dependency discovery only. Package selection, closure validation
 * and byte identities all come from the shared Rust resolver through WASM. */
export async function prepareBrowserForeignRuntime(
  modules: readonly GeneratedProjectModule[],
  files: BrowserRuntimeRequest["files"],
  manifest: string | undefined,
  resolve: (request: BrowserRuntimeRequest) => Promise<BrowserRuntimeResponse>,
  entryPath = modules[0]?.path
): Promise<ForeignRuntimeBundle> {
  const ts = await import("typescript")
  const generated = new Set(modules.map(({ path }) => modulePath(path)))
  const imports: { importer: string; specifier: string }[] = []
  const byPath = new Map(
    modules.map((module) => [modulePath(module.path), module])
  )
  const reachable = new Set<string>()
  const pending = entryPath === undefined ? [] : [modulePath(entryPath)]
  while (pending.length !== 0) {
    const importer = pending.pop()!
    if (reachable.has(importer)) continue
    reachable.add(importer)
    const module = byPath.get(importer)
    if (module === undefined)
      throw new ForeignRuntimeError([
        {
          code: "SES-F0203",
          target: "browser",
          reason: `Generated entry/import module is missing: ${importer}`,
        },
      ])
    const executable = ts.transpileModule(module.typescript, {
      compilerOptions: {
        target: ts.ScriptTarget.ES2022,
        module: ts.ModuleKind.ES2022,
        strict: true,
      },
    }).outputText
    const tree = ts.createSourceFile(
      importer,
      executable,
      ts.ScriptTarget.ES2022,
      true,
      ts.ScriptKind.TS
    )
    const specifiers = new Set<string>()
    const walk = (node: import("typescript").Node): void => {
      if (
        (ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) &&
        node.moduleSpecifier !== undefined &&
        ts.isStringLiteral(node.moduleSpecifier) &&
        !(ts.isImportDeclaration(node) && node.importClause?.isTypeOnly) &&
        !(ts.isExportDeclaration(node) && node.isTypeOnly)
      )
        specifiers.add(node.moduleSpecifier.text)
      if (
        ts.isCallExpression(node) &&
        (node.expression.kind === ts.SyntaxKind.ImportKeyword ||
          (ts.isIdentifier(node.expression) &&
            node.expression.text === "require"))
      ) {
        if (
          node.arguments.length !== 1 ||
          !ts.isStringLiteral(node.arguments[0]!)
        )
          throw new ForeignRuntimeError([
            {
              code: "SES-F0203",
              target: "browser",
              reason: "Computed runtime import has unresolved exact identity",
            },
          ])
        specifiers.add(node.arguments[0]!.text)
      }
      ts.forEachChild(node, walk)
    }
    walk(tree)
    for (const specifier of specifiers) {
      if (specifier.startsWith("@seseragi/runtime/")) continue
      if (
        specifier.startsWith(".") &&
        generated.has(relative(importer, specifier) ?? "")
      ) {
        pending.push(relative(importer, specifier)!)
        continue
      }
      imports.push({ importer, specifier })
    }
  }
  const response = await resolve({
    schema: 1,
    ...(manifest === undefined ? {} : { manifest }),
    files,
    imports,
  })
  if (response.status === "failure")
    throw new ForeignRuntimeError(response.diagnostics)
  return { ...response.bundle, generatedPaths: [...reachable] }
}
