/**
 * パターンマッチ式の制約生成
 */

import * as AST from "../../../ast"
import { createFlattenedUnionType } from "../../type-comparison"
import {
  addError,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"
import { generateConstraintsForPattern } from "./pattern"

/**
 * パターンマッチ式の制約を生成
 * match expr { ... } のような式を処理
 */
export function generateConstraintsForMatchExpression(
  ctx: InferenceContext,
  match: AST.MatchExpression,
  env: Map<string, AST.Type>
): AST.Type {
  // マッチ対象の式の型を推論
  const exprType = generateConstraintsForExpression(ctx, match.expression, env)

  if (match.cases.length === 0) {
    addError(
      ctx,
      "Match expression must have at least one case",
      match.line,
      match.column
    )
    return freshTypeVariable(ctx, match.line, match.column)
  }

  // 各ケースの結果型を収集
  const caseResultTypes: AST.Type[] = []

  for (const caseItem of match.cases) {
    // パターンマッチングで新しい変数環境を作成
    const caseEnv = new Map(env)
    generateConstraintsForPattern(ctx, caseItem.pattern, exprType, caseEnv)

    // ケースの結果型を推論
    const caseResultType = generateConstraintsForExpression(
      ctx,
      caseItem.expression,
      caseEnv
    )

    caseResultTypes.push(caseResultType)
  }

  // 複数の結果型がある場合はUnion型として統合
  const firstResultType = caseResultTypes[0]
  if (!firstResultType) {
    return freshTypeVariable(ctx, match.line, match.column)
  }

  const resultType =
    caseResultTypes.length === 1
      ? firstResultType
      : createFlattenedUnionType(caseResultTypes, match.line, match.column)

  return resultType
}
