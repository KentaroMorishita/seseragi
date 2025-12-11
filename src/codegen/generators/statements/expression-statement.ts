/**
 * 式文の生成
 */

import type { ExpressionStatement } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * 式文をTypeScriptコードに変換
 */
export function generateExpressionStatement(
  ctx: CodeGenContext,
  stmt: ExpressionStatement
): string {
  return `${generateExpression(ctx, stmt.expression)};`
}
