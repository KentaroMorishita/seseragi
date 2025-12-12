/**
 * TupleDestructuring の制約生成
 */

import type * as AST from "../../../ast"
import { type InferenceContext, setNodeType } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"
import { generateConstraintsForPattern } from "./pattern"

/**
 * タプル分割代入の制約を生成
 */
export function generateConstraintsForTupleDestructuring(
  ctx: InferenceContext,
  tupleDestr: AST.TupleDestructuring,
  env: Map<string, AST.Type>
): void {
  // 初期化式の型を推論
  const initType = generateConstraintsForExpression(
    ctx,
    tupleDestr.initializer,
    env
  )

  // タプルパターンを処理して変数を環境に追加
  generateConstraintsForPattern(ctx, tupleDestr.pattern, initType, env)

  // ノード型マップに情報を記録
  setNodeType(ctx, tupleDestr, initType)
  setNodeType(ctx, tupleDestr.initializer, initType)
}
