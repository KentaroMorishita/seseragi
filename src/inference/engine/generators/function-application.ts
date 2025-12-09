/**
 * 関数適用（単一引数）の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import {
  addConstraint,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"
import { instantiatePolymorphicType } from "./helpers"

/**
 * 関数適用の制約を生成
 * f x のような式（単一引数適用）を処理
 */
export function generateConstraintsForFunctionApplication(
  ctx: InferenceContext,
  app: AST.FunctionApplication,
  env: Map<string, AST.Type>
): AST.Type {
  // 関数の型を取得
  let funcType: AST.Type

  // 関数が識別子の場合、環境から直接取得を優先（効率化）
  if (app.function.kind === "Identifier") {
    const identifier = app.function as AST.Identifier
    const rawFuncType = env.get(identifier.name)
    if (rawFuncType) {
      // 多相型のインスタンス化
      funcType = instantiatePolymorphicType(ctx, rawFuncType, app.line, app.column)
    } else {
      // 環境にない場合は式から推論
      funcType = generateConstraintsForExpression(ctx, app.function, env)
      funcType = instantiatePolymorphicType(ctx, funcType, app.line, app.column)
    }
  } else {
    // 関数が複雑な式の場合
    funcType = generateConstraintsForExpression(ctx, app.function, env)
    funcType = instantiatePolymorphicType(ctx, funcType, app.line, app.column)
  }

  // 期待されるパラメータ型を決定
  let expectedParamType: AST.Type
  if (funcType.kind === "FunctionType") {
    expectedParamType = (funcType as AST.FunctionType).paramType
  } else {
    expectedParamType = freshTypeVariable(ctx, app.line, app.column)
  }

  // 引数の型を推論（期待型をヒントとして渡す）
  const argType = generateConstraintsForExpression(
    ctx,
    app.argument,
    env,
    expectedParamType
  )

  // 関数型が既知の場合（高速パス）
  if (funcType.kind === "FunctionType") {
    const ft = funcType as AST.FunctionType

    // 引数型の制約を追加
    addConstraint(
      ctx,
      new TypeConstraint(
        argType,
        ft.paramType,
        app.argument.line,
        app.argument.column,
        `Function application argument type`
      )
    )

    // 戻り値型を直接返す
    return ft.returnType
  }

  // 関数型が不明な場合（標準パス）
  const resultType = freshTypeVariable(ctx, app.line, app.column)

  // 関数型の構造制約: funcType ~ (argType -> resultType)
  const expectedFuncType = new AST.FunctionType(
    expectedParamType,
    resultType,
    app.line,
    app.column
  )

  addConstraint(
    ctx,
    new TypeConstraint(
      funcType,
      expectedFuncType,
      app.line,
      app.column,
      `Function application structure`
    )
  )

  // パラメータ型の統一制約
  addConstraint(
    ctx,
    new TypeConstraint(
      argType,
      expectedParamType,
      app.argument.line,
      app.argument.column,
      `Function application parameter type`
    )
  )

  return resultType
}
