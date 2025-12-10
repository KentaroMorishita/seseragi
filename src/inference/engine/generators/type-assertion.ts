/**
 * TypeAssertion（型アサーション）の制約生成
 */

import * as AST from "../../../ast"
import { setNodeType, type InferenceContext } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * TypeAssertion（型アサーション）の制約を生成
 *
 * 型アサーション（as演算子）は制約を生成せずにターゲット型を直接返す
 * これによりTypeScript風の型チェック緩和を実現
 */
export function generateConstraintsForTypeAssertion(
  ctx: InferenceContext,
  assertion: AST.TypeAssertion,
  env: Map<string, AST.Type>
): AST.Type {
  // 元の式の型を推論
  const exprType = generateConstraintsForExpression(ctx, assertion.expression, env)

  // 型アサーションの場合、制約を生成せずに直接ターゲット型を返す
  const targetType = assertion.targetType

  // ノードタイプマップに記録
  setNodeType(ctx, assertion.expression, exprType)
  setNodeType(ctx, assertion, targetType)

  return targetType
}
