/**
 * 型操作の式生成
 * 型アサーション (as) と型チェック (is) の生成
 */

import type { Identifier, IsExpression, TypeAssertion } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateType, generateTypeString } from "../../type-generators"
import { generateExpression } from "../dispatcher"

/**
 * 型アサーションをTypeScriptコードに変換
 * 例: x as Int -> (x as number)
 */
export function generateTypeAssertion(
  ctx: CodeGenContext,
  assertion: TypeAssertion
): string {
  const expr = generateExpression(ctx, assertion.expression)
  const targetType = generateType(assertion.targetType)

  // TypeScript風の型アサーション構文で生成
  return `(${expr} as ${targetType})`
}

/**
 * is式をTypeScriptコードに変換
 * 例: x is Int -> ssrgIsType(x, "Int", "x")
 */
export function generateIsExpression(
  ctx: CodeGenContext,
  isExpr: IsExpression
): string {
  const leftExpr = generateExpression(ctx, isExpr.left)
  const rightType = generateTypeString(isExpr.rightType)

  // 左辺が単純な変数の場合は変数名も渡す
  if (isExpr.left.kind === "Identifier") {
    const variableName = (isExpr.left as Identifier).name
    return `ssrgIsType(${leftExpr}, "${rightType}", "${variableName}")`
  }

  return `ssrgIsType(${leftExpr}, "${rightType}")`
}
