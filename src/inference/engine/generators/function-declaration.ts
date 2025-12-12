/**
 * 関数宣言の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import { substituteTypeVariables } from "../../type-substitution-utils"
import { PolymorphicTypeVariable } from "../../type-variables"
import {
  addConstraint,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { resolveTypeAlias } from "../type-alias-resolver"
import { generateConstraintsForExpression } from "./dispatcher"
import { generalize } from "./helpers"

/**
 * 関数宣言の制約を生成
 * fn name<T>(param: Type): ReturnType = body のような宣言を処理
 */
export function generateConstraintsForFunctionDeclaration(
  ctx: InferenceContext,
  func: AST.FunctionDeclaration,
  env: Map<string, AST.Type>
): void {
  // 型パラメータを多相型変数として扱うマップを作成
  const typeParameterMap = new Map<string, AST.Type>()
  if (func.typeParameters) {
    for (const typeParam of func.typeParameters) {
      typeParameterMap.set(
        typeParam.name,
        new PolymorphicTypeVariable(
          typeParam.name,
          typeParam.line,
          typeParam.column
        )
      )
    }
  }

  // 型の解決において、型パラメータを多相型変数で置換
  const resolveTypeWithTypeParameters = (
    type: AST.Type | undefined
  ): AST.Type => {
    if (!type) {
      return freshTypeVariable(ctx, func.line, func.column)
    }
    return substituteTypeVariables(type, typeParameterMap)
  }

  // 戻り値の型が指定されていない場合は型変数を作成
  let returnType = resolveTypeWithTypeParameters(func.returnType)

  // 型エイリアスの解決
  if (returnType) {
    returnType = resolveTypeAlias(ctx, returnType)
  }

  // パラメータの型を事前に決定
  const paramTypes: AST.Type[] = []
  for (const param of func.parameters) {
    const paramType = resolveTypeWithTypeParameters(param.type)
    paramTypes.push(paramType)
  }

  // 関数シグネチャを構築
  let funcType: AST.Type = returnType

  // パラメータから関数シグネチャを構築（カリー化）
  if (func.parameters.length === 0) {
    // 引数なしの関数は Unit -> ReturnType
    const unitType = new AST.PrimitiveType("Unit", func.line, func.column)
    funcType = new AST.FunctionType(unitType, funcType, func.line, func.column)
  } else {
    // 引数ありの関数は通常のカリー化（後ろから前に構築）
    for (let i = paramTypes.length - 1; i >= 0; i--) {
      const paramType = paramTypes[i]
      if (paramType) {
        funcType = new AST.FunctionType(
          paramType,
          funcType,
          func.line,
          func.column
        )
      }
    }
  }

  // 関数を環境に追加
  const generalizedType = generalize(funcType, env)
  env.set(func.name, generalizedType)

  // 関数本体の型推論用の環境を作成
  const bodyEnv = new Map(env)

  // パラメータの型を環境に追加（型エイリアス解決後）
  for (let i = 0; i < func.parameters.length; i++) {
    const param = func.parameters[i]
    const paramType = paramTypes[i]

    if (param && paramType) {
      // パラメータ型も型エイリアス解決を行う
      const resolvedParamType = resolveTypeAlias(ctx, paramType)
      bodyEnv.set(param.name, resolvedParamType)
    }
  }

  // 関数本体の型を推論
  const bodyType = generateConstraintsForExpression(
    ctx,
    func.body,
    bodyEnv,
    returnType
  )

  // 関数本体の型と戻り値型が一致することを制約として追加
  addConstraint(
    ctx,
    new TypeConstraint(
      bodyType,
      returnType,
      func.body.line,
      func.body.column,
      `Function ${func.name} body type`
    )
  )

  // ノードタイプマップに記録
  ctx.nodeTypeMap.set(func, generalizedType)
}
