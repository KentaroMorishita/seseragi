/**
 * ConsExpression の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import { addConstraint, type InferenceContext } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * Cons式 (head : tail) の制約を生成
 */
export function generateConstraintsForConsExpression(
  ctx: InferenceContext,
  consExpr: AST.ConsExpression,
  env: Map<string, AST.Type>
): AST.Type {
  // head : tail の型推論
  const headType = generateConstraintsForExpression(ctx, consExpr.left, env)
  const tailType = generateConstraintsForExpression(ctx, consExpr.right, env)

  // tailはList<T>型でなければならない
  const expectedTailType = new AST.GenericType(
    "List",
    [headType],
    consExpr.right.line,
    consExpr.right.column
  )

  addConstraint(
    ctx,
    new TypeConstraint(
      tailType,
      expectedTailType,
      consExpr.right.line,
      consExpr.right.column,
      "Cons tail must be List type"
    )
  )

  // 結果の型もList<T>
  return new AST.GenericType(
    "List",
    [headType],
    consExpr.line,
    consExpr.column
  )
}
