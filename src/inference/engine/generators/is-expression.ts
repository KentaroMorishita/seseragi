/**
 * is式の制約生成
 */

import * as AST from "../../../ast"
import { type InferenceContext } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * is式の制約を生成
 * x is Type => Bool
 */
export function generateConstraintsForIsExpression(
  ctx: InferenceContext,
  isExpr: AST.IsExpression,
  env: Map<string, AST.Type>
): AST.Type {
  // 左辺の式の型を推論（副作用として制約を生成）
  generateConstraintsForExpression(ctx, isExpr.left, env)

  // is式は常にBool型を返す
  return new AST.PrimitiveType("Bool", isExpr.line, isExpr.column)
}
