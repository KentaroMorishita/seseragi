import type * as TypeScript from "typescript"

/** Cooperative guard for compiler-generated DOM code, not a JavaScript sandbox. */
export function createExecutionCheckpoint() {
  const now = performance.now.bind(performance)
  let calls = 0
  let started = 0
  return () => {
    if (calls === 0) {
      started = now()
      setTimeout(() => {
        calls = 0
      }, 0)
    }
    calls++
    if (calls > 1_000_000 || (calls % 1024 === 0 && now() - started > 500))
      throw new Error(
        "Program computation limit exceeded. Simplify the program and run again."
      )
  }
}

export function checkpointTransformer(
  ts: typeof TypeScript,
  name: string
): TypeScript.TransformerFactory<TypeScript.SourceFile> {
  return (context) => {
    const check = () =>
      ts.factory.createExpressionStatement(
        ts.factory.createCallExpression(
          ts.factory.createIdentifier(name),
          undefined,
          []
        )
      )
    const prepend = (body: TypeScript.Statement): TypeScript.Block =>
      ts.factory.createBlock(
        [check(), ...(ts.isBlock(body) ? body.statements : [body])],
        true
      )
    const visit = (
      node: TypeScript.Node
    ): TypeScript.VisitResult<TypeScript.Node> => {
      const next = ts.visitEachChild(node, visit, context)
      if (
        ts.isBlock(next) &&
        node.parent !== undefined &&
        ts.isFunctionLike(node.parent)
      )
        return prepend(next)
      if (ts.isArrowFunction(next) && !ts.isBlock(next.body))
        return ts.factory.updateArrowFunction(
          next,
          next.modifiers,
          next.typeParameters,
          next.parameters,
          next.type,
          next.equalsGreaterThanToken,
          ts.factory.createBlock(
            [check(), ts.factory.createReturnStatement(next.body)],
            true
          )
        )
      if (
        ts.isForStatement(next) ||
        ts.isForInStatement(next) ||
        ts.isForOfStatement(next) ||
        ts.isWhileStatement(next) ||
        ts.isDoStatement(next)
      )
        return ts.visitEachChild(
          next,
          (child) =>
            child === next.statement ? prepend(next.statement) : child,
          context
        )
      return next
    }
    return (file) => ts.visitNode(file, visit) as TypeScript.SourceFile
  }
}
