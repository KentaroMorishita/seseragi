/**
 * ラムダ式の制約生成
 */

import * as AST from "../../../ast"
import { freshTypeVariable, type InferenceContext } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * ラムダ式の制約を生成
 * (\x -> body) のような式を処理
 */
export function generateConstraintsForLambdaExpression(
  ctx: InferenceContext,
  lambda: AST.LambdaExpression,
  env: Map<string, AST.Type>
): AST.Type {
  // ラムダ本体用の新しい環境を作成
  const lambdaEnv = new Map(env)

  // パラメータの型変数を作成
  const parameterTypes: AST.Type[] = []

  for (const param of lambda.parameters) {
    let paramType = param.type

    // パラメータ型がプレースホルダー "_" の場合、新しい型変数を生成
    if (
      paramType.kind === "PrimitiveType" &&
      (paramType as AST.PrimitiveType).name === "_"
    ) {
      paramType = freshTypeVariable(ctx, param.line, param.column)
    }

    parameterTypes.push(paramType)
    lambdaEnv.set(param.name, paramType)
  }

  // ラムダ本体の型を推論
  const bodyType = generateConstraintsForExpression(ctx, lambda.body, lambdaEnv)

  // カリー化: 右から左へ関数型を構築
  // \x y -> body は (x -> (y -> body)) になる
  let resultType: AST.Type = bodyType
  for (let i = lambda.parameters.length - 1; i >= 0; i--) {
    const paramType = parameterTypes[i]
    if (paramType) {
      resultType = new AST.FunctionType(
        paramType,
        resultType,
        lambda.line,
        lambda.column
      )
    }
  }

  return resultType
}
