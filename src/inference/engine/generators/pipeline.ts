/**
 * パイプライン演算子の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import { addConstraint, freshTypeVariable, type InferenceContext } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * パイプライン演算子の制約を生成
 * left |> right のような式を処理
 * left の値を right 関数に適用する
 */
export function generateConstraintsForPipeline(
  ctx: InferenceContext,
  pipeline: AST.Pipeline,
  env: Map<string, AST.Type>
): AST.Type {
  // 左辺（引数）の型を推論
  const leftType = generateConstraintsForExpression(ctx, pipeline.left, env)
  // 右辺（関数）の型を推論
  const rightType = generateConstraintsForExpression(ctx, pipeline.right, env)
  // 結果型の型変数を生成
  const resultType = freshTypeVariable(ctx, pipeline.line, pipeline.column)

  // 右辺は leftType -> resultType という関数型でなければならない
  const expectedFuncType = new AST.FunctionType(
    leftType,
    resultType,
    pipeline.line,
    pipeline.column
  )

  addConstraint(
    ctx,
    new TypeConstraint(
      rightType,
      expectedFuncType,
      pipeline.line,
      pipeline.column,
      `Pipeline operator`
    )
  )

  return resultType
}
