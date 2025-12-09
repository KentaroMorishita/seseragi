/**
 * タプル式の制約生成
 */

import * as AST from "../../../ast"
import type { InferenceContext } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * タプル式の制約を生成
 * (a, b, c) のような式を処理
 */
export function generateConstraintsForTupleExpression(
  ctx: InferenceContext,
  tuple: AST.TupleExpression,
  env: Map<string, AST.Type>
): AST.Type {
  // 各要素の型を推論
  const elementTypes: AST.Type[] = []

  for (const element of tuple.elements) {
    const elementType = generateConstraintsForExpression(ctx, element, env)
    elementTypes.push(elementType)
  }

  // タプル型を作成
  return new AST.TupleType(elementTypes, tuple.line, tuple.column)
}
