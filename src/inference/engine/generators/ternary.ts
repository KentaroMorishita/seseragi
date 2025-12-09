/**
 * 三項演算子の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import { createFlattenedUnionType, typesEqual } from "../../type-comparison"
import { addConstraint, type InferenceContext } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * 三項演算子の制約を生成
 * cond ? trueExpr : falseExpr のような式を処理
 */
export function generateConstraintsForTernaryExpression(
  ctx: InferenceContext,
  ternary: AST.TernaryExpression,
  env: Map<string, AST.Type>,
  expectedType?: AST.Type
): AST.Type {
  const condType = generateConstraintsForExpression(ctx, ternary.condition, env)
  const trueType = generateConstraintsForExpression(
    ctx,
    ternary.trueExpression,
    env,
    expectedType
  )
  const falseType = generateConstraintsForExpression(
    ctx,
    ternary.falseExpression,
    env,
    expectedType
  )

  // 条件はBool型でなければならない
  addConstraint(
    ctx,
    new TypeConstraint(
      condType,
      new AST.PrimitiveType(
        "Bool",
        ternary.condition.line,
        ternary.condition.column
      ),
      ternary.condition.line,
      ternary.condition.column,
      `Ternary expression condition`
    )
  )

  // trueとfalseの型が同じかチェック
  if (typesEqual(trueType, falseType)) {
    return trueType
  }

  // 期待される型が指定されている場合
  if (expectedType) {
    addConstraint(
      ctx,
      new TypeConstraint(
        trueType,
        expectedType,
        ternary.trueExpression.line,
        ternary.trueExpression.column,
        `Ternary true branch type`
      )
    )
    addConstraint(
      ctx,
      new TypeConstraint(
        falseType,
        expectedType,
        ternary.falseExpression.line,
        ternary.falseExpression.column,
        `Ternary false branch type`
      )
    )
    return expectedType
  }

  // 期待される型が指定されていない場合は、ユニオン型として返す
  return createFlattenedUnionType(
    [trueType, falseType],
    ternary.line,
    ternary.column
  )
}
