/**
 * 条件式の生成
 */

import type { ConditionalExpression, TernaryExpression } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * 条件式をTypeScriptコードに変換
 */
export function generateConditionalExpression(
  ctx: CodeGenContext,
  cond: ConditionalExpression
): string {
  const condition = generateExpression(ctx, cond.condition)
  const thenBranch = generateExpression(ctx, cond.thenExpression)
  const elseBranch = generateExpression(ctx, cond.elseExpression)

  return `(${condition} ? ${thenBranch} : ${elseBranch})`
}

/**
 * 三項演算子をTypeScriptコードに変換
 */
export function generateTernaryExpression(
  ctx: CodeGenContext,
  ternary: TernaryExpression
): string {
  const condition = generateExpression(ctx, ternary.condition)
  const trueBranch = generateExpression(ctx, ternary.trueExpression)
  const falseBranch = generateExpression(ctx, ternary.falseExpression)

  return `(${condition} ? ${trueBranch} : ${falseBranch})`
}
