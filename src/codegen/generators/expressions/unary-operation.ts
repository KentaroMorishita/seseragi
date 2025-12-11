/**
 * 単項演算式の生成
 */

import type { UnaryOperation } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * 単項演算をTypeScriptコードに変換
 */
export function generateUnaryOperation(
  ctx: CodeGenContext,
  unaryOp: UnaryOperation
): string {
  const operand = generateExpression(ctx, unaryOp.operand)

  switch (unaryOp.operator) {
    case "*":
      // Signal getValue: *signal -> signal.getValue()
      return `(${operand}.getValue())`
    default:
      // 演算子をそのまま使用（TypeScriptと同じ）
      return `(${unaryOp.operator}${operand})`
  }
}
