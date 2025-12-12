/**
 * 配列関連式の生成
 */

import type { ArrayAccess, ArrayLiteral, RangeLiteral } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * 配列リテラルをTypeScriptコードに変換
 */
export function generateArrayLiteral(
  ctx: CodeGenContext,
  arrayLiteral: ArrayLiteral
): string {
  if (arrayLiteral.elements.length === 0) {
    return "[]"
  }

  const elements = arrayLiteral.elements.map((element) =>
    generateExpression(ctx, element)
  )

  return `[${elements.join(", ")}]`
}

/**
 * 配列アクセスをTypeScriptコードに変換
 * 安全なアクセス: Maybe型を返す
 */
export function generateArrayAccess(
  ctx: CodeGenContext,
  arrayAccess: ArrayAccess
): string {
  const array = generateExpression(ctx, arrayAccess.array)
  const index = generateExpression(ctx, arrayAccess.index)

  // 安全な配列アクセス: Maybe型を返す
  const actualArray = `(${array}.tag === 'Tuple' ? ${array}.elements : ${array})`
  return `((${index}) >= 0 && (${index}) < ${actualArray}.length ? { tag: 'Just', value: ${actualArray}[${index}] } : { tag: 'Nothing' })`
}

/**
 * 範囲リテラルをTypeScriptコードに変換
 */
export function generateRangeLiteral(
  ctx: CodeGenContext,
  range: RangeLiteral
): string {
  const start = generateExpression(ctx, range.start)
  const end = generateExpression(ctx, range.end)

  if (range.inclusive) {
    // 1..=5 -> Array.from({length: 5 - 1 + 1}, (_, i) => i + 1)
    return `Array.from({length: ${end} - ${start} + 1}, (_, i) => i + ${start})`
  } else {
    // 1..5 -> Array.from({length: 5 - 1}, (_, i) => i + 1)
    return `Array.from({length: ${end} - ${start}}, (_, i) => i + ${start})`
  }
}
