/**
 * SignalExpression の制約生成
 */

import * as AST from "../../../ast"
import { type InferenceContext } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * Signal式の制約を生成
 * Signal<T>の型を推論
 */
export function generateConstraintsForSignalExpression(
  ctx: InferenceContext,
  signal: AST.SignalExpression,
  env: Map<string, AST.Type>
): AST.Type {
  const valueType = generateConstraintsForExpression(
    ctx,
    signal.initialValue,
    env
  )
  return new AST.GenericType(
    "Signal",
    [valueType],
    signal.line,
    signal.column
  )
}
