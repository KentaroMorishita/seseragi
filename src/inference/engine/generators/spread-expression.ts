/**
 * SpreadExpression の制約生成
 */

import * as AST from "../../../ast"
import { type InferenceContext } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * スプレッド式 ...expr の制約を生成
 */
export function generateConstraintsForSpreadExpression(
  ctx: InferenceContext,
  spread: AST.SpreadExpression,
  env: Map<string, AST.Type>
): AST.Type {
  // スプレッド式自体は中身の式の型と同じ
  return generateConstraintsForExpression(ctx, spread.expression, env)
}
