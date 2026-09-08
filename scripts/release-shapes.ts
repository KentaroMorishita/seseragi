import { readFileSync } from "node:fs"
import { dirname, resolve } from "node:path"
import ts from "typescript"

export type Predicate =
  | "newtype-erased"
  | "surface-sugar-erased"
  | "self-tail-loop"
export type Shape = { symbol: string; require: Predicate[] }
export type Inspection = {
  profile: string
  module?: string
  inspectionRoots: Record<string, string>
  newtypeConstructors: string[]
  effectRoots?: string[]
  outputs: { typescript: string }
}

// This is deliberately a conservative proof over the emitted TypeScript AST.
// Unknown representations fail closed. Metadata identifies binders, never claims
// that a pass succeeded. Type-only assertions/brands disappear at the TS boundary.
export function checkShapes(
  source: string,
  metadata: Inspection,
  shapes: Shape[]
): void {
  if (metadata.profile !== "release")
    throw new Error("shape inspection requires release output")
  const file = ts.createSourceFile(
    "inspection.ts",
    source,
    ts.ScriptTarget.ES2022,
    true
  )
  const host = ts.createCompilerHost({ noLib: true, noResolve: true })
  host.getSourceFile = (name) => (name === file.fileName ? file : undefined)
  const program = ts.createProgram(
    [file.fileName],
    { noLib: true, noResolve: true },
    host
  )
  if (program.getSyntacticDiagnostics(file).length !== 0)
    throw new Error("invalid generated TypeScript")
  const checker = program.getTypeChecker()
  const declarations = new Map<string, ts.VariableDeclaration>()
  for (const statement of file.statements) {
    if (ts.isVariableStatement(statement)) {
      for (const declaration of statement.declarationList.declarations) {
        if (ts.isIdentifier(declaration.name))
          declarations.set(declaration.name.text, declaration)
      }
    }
  }
  const constructorSymbols = new Set<ts.Symbol>()
  function collectConstructors(node: ts.Node) {
    if (
      ts.isIdentifier(node) &&
      metadata.newtypeConstructors.includes(node.text)
    ) {
      const symbol = checker.getSymbolAtLocation(node)
      if (
        symbol &&
        (symbol.declarations ?? []).some(
          (decl) =>
            ts.isImportSpecifier(decl) ||
            (ts.isVariableDeclaration(decl) &&
              declarations.get(node.text) === decl)
        )
      ) {
        constructorSymbols.add(symbol)
      }
    }
    ts.forEachChild(node, collectConstructors)
  }
  collectConstructors(file)
  for (const shape of shapes) {
    const separator = shape.symbol.lastIndexOf("::")
    const fixtureModule = shape.symbol.slice(0, separator)
    const moduleTail = metadata.module?.split("/").at(-1)
    const canonical =
      fixtureModule === moduleTail
        ? `${metadata.module}::${shape.symbol.slice(separator + 2)}`
        : shape.symbol
    const declaration = declarations.get(metadata.inspectionRoots[canonical])
    if (!declaration?.initializer)
      throw new Error(`missing inspection root ${shape.symbol}`)
    const rootSymbol = checker.getSymbolAtLocation(declaration.name)
    let rootBody: ts.Node = declaration.initializer
    while (ts.isArrowFunction(rootBody)) rootBody = rootBody.body
    const ownsLoop = (node: ts.Node): boolean => {
      let parent = node.parent
      while (parent && parent !== rootBody) {
        if (ts.isFunctionLike(parent)) return false
        parent = parent.parent
      }
      return parent === rootBody
    }
    for (const predicate of shape.require) {
      let invalid =
        predicate === "self-tail-loop" &&
        (metadata.effectRoots ?? []).includes(canonical)
      let loop = false
      let continuation = false
      const visited = new Set<ts.VariableDeclaration>()
      function visit(node: ts.Node): void {
        if (ts.isTypeNode(node)) return
        if (predicate === "self-tail-loop") {
          if (
            ts.isIdentifier(node) &&
            checker.getSymbolAtLocation(node) === rootSymbol
          )
            invalid = true
          if (
            ts.isWhileStatement(node) &&
            node.expression.kind === ts.SyntaxKind.TrueKeyword &&
            ownsLoop(node)
          )
            loop = true
          if (ts.isContinueStatement(node) && ownsLoop(node))
            continuation = true
        } else {
          // Objects/field projections can represent wrappers or dispatch. They
          // need a richer proof before this checker can certify them.
          if (
            ts.isObjectLiteralExpression(node) ||
            ts.isArrayLiteralExpression(node) ||
            ts.isNewExpression(node)
          )
            invalid = true
          if (predicate === "newtype-erased") {
            if (
              ts.isPropertyAccessExpression(node) ||
              ts.isElementAccessExpression(node)
            )
              invalid = true
            if (ts.isIdentifier(node)) {
              const symbol = checker.getSymbolAtLocation(node)
              if (symbol && constructorSymbols.has(symbol)) invalid = true
            }
          }
          if (
            predicate === "newtype-erased" &&
            ts.isCallExpression(node) &&
            ts.isIdentifier(node.expression)
          ) {
            const symbol = checker.getSymbolAtLocation(node.expression)
            for (const callee of symbol?.declarations ?? []) {
              if (
                ts.isVariableDeclaration(callee) &&
                callee.initializer &&
                !visited.has(callee)
              ) {
                visited.add(callee)
                visit(callee.initializer)
              }
            }
          }
        }
        ts.forEachChild(node, visit)
      }
      if (
        !["newtype-erased", "surface-sugar-erased", "self-tail-loop"].includes(
          predicate
        )
      )
        invalid = true
      visit(declaration.initializer)
      if (
        invalid ||
        (predicate === "self-tail-loop" && !(loop && continuation))
      ) {
        throw new Error(`${shape.symbol}: cannot prove ${predicate}`)
      }
    }
  }
}

if (import.meta.main) {
  const [metadataPath, expectationPath] = process.argv.slice(2)
  if (!metadataPath || !expectationPath)
    throw new Error(
      "usage: release-shapes.ts generated-module.json project.expect.json"
    )
  const metadata = JSON.parse(readFileSync(metadataPath, "utf8")) as Inspection
  const expectation = JSON.parse(readFileSync(expectationPath, "utf8")) as {
    shapes: Shape[]
  }
  checkShapes(
    readFileSync(
      resolve(dirname(metadataPath), metadata.outputs.typescript),
      "utf8"
    ),
    metadata,
    expectation.shapes
  )
  console.log(`release shapes: ${expectation.shapes.length} symbols passed`)
}
