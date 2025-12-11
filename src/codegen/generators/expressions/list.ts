/**
 * リスト関連式の生成
 */

import type { ListSugar, ConsExpression } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * リストシュガーをTypeScriptコードに変換
 * [1, 2, 3] -> Cons(1, Cons(2, Cons(3, Empty)))
 */
export function generateListSugar(
  ctx: CodeGenContext,
  listSugar: ListSugar
): string {
  if (listSugar.elements.length === 0) {
    return "Empty"
  }

  // リストを右からConsで構築
  let result = "Empty"
  for (let i = listSugar.elements.length - 1; i >= 0; i--) {
    const element = generateExpression(ctx, listSugar.elements[i])
    result = `Cons(${element}, ${result})`
  }

  return result
}

/**
 * Cons式をTypeScriptコードに変換
 * left : right -> Cons(left, right)
 */
export function generateConsExpression(
  ctx: CodeGenContext,
  consExpr: ConsExpression
): string {
  const left = generateExpression(ctx, consExpr.left)
  const right = generateExpression(ctx, consExpr.right)
  return `Cons(${left}, ${right})`
}
