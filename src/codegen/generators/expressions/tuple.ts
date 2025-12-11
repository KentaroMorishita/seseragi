/**
 * タプル式の生成
 */

import type { TupleExpression } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * タプル式をTypeScriptコードに変換
 */
export function generateTupleExpression(
  ctx: CodeGenContext,
  tuple: TupleExpression
): string {
  if (tuple.elements.length === 0) {
    return "{ tag: 'Tuple', elements: [] }"
  }

  const elements = tuple.elements.map((element) =>
    generateExpression(ctx, element)
  )

  return `{ tag: 'Tuple', elements: [${elements.join(", ")}] }`
}
