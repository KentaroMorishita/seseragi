/**
 * 条件式（if-then-else）の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import { createFlattenedUnionType, typesEqual } from "../../type-comparison"
import { addConstraint, type InferenceContext } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * 条件式の制約を生成
 */
export function generateConstraintsForConditional(
  ctx: InferenceContext,
  cond: AST.ConditionalExpression,
  env: Map<string, AST.Type>
): AST.Type {
  const condType = generateConstraintsForExpression(ctx, cond.condition, env)
  const thenType = generateConstraintsForExpression(
    ctx,
    cond.thenExpression,
    env
  )
  const elseType = generateConstraintsForExpression(
    ctx,
    cond.elseExpression,
    env
  )

  // 条件はBool型でなければならない
  addConstraint(
    ctx,
    new TypeConstraint(
      condType,
      new AST.PrimitiveType("Bool", cond.condition.line, cond.condition.column),
      cond.condition.line,
      cond.condition.column,
      `Conditional expression condition`
    )
  )

  // thenとelseの型が同じかチェック
  if (typesEqual(thenType, elseType)) {
    // 同じ型の場合はそのまま返す
    return thenType
  }

  // 異なる型の場合は、ユニオン型として返す
  return createFlattenedUnionType([thenType, elseType], cond.line, cond.column)
}
