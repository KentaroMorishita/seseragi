/**
 * 変数宣言の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import {
  addConstraint,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * 型を一般化する（自由型変数を多相型変数に変換）
 * TODO: 完全な実装は別途必要
 */
function generalize(
  _ctx: InferenceContext,
  type: AST.Type,
  _env: Map<string, AST.Type>
): AST.Type {
  // 簡略版：そのまま返す
  // 完全な実装では、環境に束縛されていない型変数を多相型変数に変換
  return type
}

/**
 * 変数宣言の制約を生成
 * let name: Type = initializer のような宣言を処理
 */
export function generateConstraintsForVariableDeclaration(
  ctx: InferenceContext,
  varDecl: AST.VariableDeclaration,
  env: Map<string, AST.Type>
): AST.Type {
  // 型注釈がある場合は期待される型として渡す
  let expectedType: AST.Type | undefined
  if (varDecl.type) {
    expectedType = varDecl.type
    // 型エイリアスの解決
    if (varDecl.type.kind === "PrimitiveType") {
      const aliasedType = env.get((varDecl.type as AST.PrimitiveType).name)
      if (aliasedType) {
        expectedType = aliasedType
      }
    }
  }

  // 初期化式の型を推論
  const initType = generateConstraintsForExpression(
    ctx,
    varDecl.initializer,
    env,
    expectedType
  )

  let finalType: AST.Type
  if (varDecl.type) {
    // 明示的な型注釈がある場合
    const resolvedType = expectedType!

    // 制約を追加
    addConstraint(
      ctx,
      new TypeConstraint(
        initType,
        resolvedType,
        varDecl.line,
        varDecl.column,
        `Variable ${varDecl.name} type annotation`
      )
    )
    env.set(varDecl.name, resolvedType)
    finalType = resolvedType
  } else {
    // 型注釈がない場合
    if (varDecl.initializer.kind === "LambdaExpression") {
      // ラムダ式は多相性を保つため一般化
      const generalizedType = generalize(ctx, initType, env)
      env.set(varDecl.name, generalizedType)
      finalType = generalizedType
    } else {
      // 値型の場合：推論された型をそのまま使用
      env.set(varDecl.name, initType)
      finalType = initType
    }
  }

  // ノード型マップに記録
  ctx.nodeTypeMap.set(varDecl, finalType)
  ctx.nodeTypeMap.set(varDecl.initializer, initType)

  return finalType
}
