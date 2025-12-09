/**
 * 配列アクセスの制約生成
 */

import * as AST from "../../../ast"
import { ArrayAccessConstraint, TypeConstraint } from "../../constraints"
import {
  addConstraint,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * 配列アクセスの制約を生成
 * array[index] のような式を処理
 */
export function generateConstraintsForArrayAccess(
  ctx: InferenceContext,
  arrayAccess: AST.ArrayAccess,
  env: Map<string, AST.Type>
): AST.Type {
  const arrayType = generateConstraintsForExpression(
    ctx,
    arrayAccess.array,
    env
  )
  const indexType = generateConstraintsForExpression(
    ctx,
    arrayAccess.index,
    env
  )

  // インデックスはInt型でなければならない
  addConstraint(
    ctx,
    new TypeConstraint(
      indexType,
      new AST.PrimitiveType(
        "Int",
        arrayAccess.index.line,
        arrayAccess.index.column
      ),
      arrayAccess.index.line,
      arrayAccess.column,
      "Array index must be Int"
    )
  )

  // 戻り値の型変数を作成
  const resultType = freshTypeVariable(ctx, arrayAccess.line, arrayAccess.column)

  // ArrayAccessConstraintを追加（配列またはタプルアクセス用）
  const constraint = new ArrayAccessConstraint(
    arrayType,
    resultType,
    arrayAccess.line,
    arrayAccess.column,
    "Array or Tuple access"
  )
  ctx.constraints.push(constraint)

  return resultType
}
