/**
 * 型宣言（ADT）の制約生成
 */

import * as AST from "../../../ast"
import { addError, type InferenceContext } from "../context"

/**
 * コンストラクタ型を作成
 * データコンストラクタの型を構築する
 */
function createConstructorType(
  field: AST.TypeField,
  adtType: AST.Type
): AST.Type {
  if (field.type instanceof AST.PrimitiveType && field.type.name === "Unit") {
    // データなしのコンストラクタ (Red, Green, Blue)
    return adtType
  } else if (
    field.type instanceof AST.GenericType &&
    field.type.name === "Tuple"
  ) {
    // データ付きのコンストラクタ (RGB Int Int Int)
    let resultType = adtType

    // 型引数から逆順でカリー化された関数シグネチャを構築
    for (let i = field.type.typeArguments.length - 1; i >= 0; i--) {
      const paramType = field.type.typeArguments[i]
      if (paramType) {
        resultType = new AST.FunctionType(
          paramType,
          resultType,
          field.line,
          field.column
        )
      }
    }

    return resultType
  } else {
    // その他のケース（単一データ）
    return new AST.FunctionType(field.type, adtType, field.line, field.column)
  }
}

/**
 * 型宣言（ADT）の制約を生成
 * type Color = Red | Green | Blue のような宣言を処理
 */
export function generateConstraintsForTypeDeclaration(
  ctx: InferenceContext,
  typeDecl: AST.TypeDeclaration,
  env: Map<string, AST.Type>
): void {
  // ビルトイン型との名前衝突をチェック
  const builtinTypes = ["Maybe", "Either", "List"]
  if (builtinTypes.includes(typeDecl.name)) {
    addError(
      ctx,
      `Type '${typeDecl.name}' conflicts with builtin type. Use a different name.`,
      typeDecl.line,
      typeDecl.column
    )
    return
  }

  // ADT型を環境に追加
  const adtType = new AST.PrimitiveType(
    typeDecl.name,
    typeDecl.line,
    typeDecl.column
  )

  // ADT型自体を環境に登録
  env.set(typeDecl.name, adtType)

  // ADT型をコンテキストにも登録
  ctx.adtTypes.set(typeDecl.name, typeDecl)

  // 各バリアント（コンストラクタ）を環境に追加
  for (const field of typeDecl.fields) {
    const constructorType = createConstructorType(field, adtType)
    env.set(field.name, constructorType)
  }

  // ノードタイプマップに記録
  ctx.nodeTypeMap.set(typeDecl, adtType)
}

/**
 * 型エイリアス宣言の制約を生成
 * type alias MyType = ExistingType のような宣言を処理
 */
export function generateConstraintsForTypeAliasDeclaration(
  ctx: InferenceContext,
  typeAlias: AST.TypeAliasDeclaration,
  env: Map<string, AST.Type>
): void {
  // ジェネリック型エイリアス情報を保存
  ctx.typeAliases.set(typeAlias.name, typeAlias)

  // 非ジェネリック型エイリアスの場合は環境に追加
  if (!typeAlias.typeParameters || typeAlias.typeParameters.length === 0) {
    env.set(typeAlias.name, typeAlias.aliasedType)
    ctx.environment.set(typeAlias.name, typeAlias.aliasedType)
  }
  // ジェネリック型エイリアスは resolveTypeAlias で具体化時に処理

  // ノードタイプマップに記録
  ctx.nodeTypeMap.set(typeAlias, typeAlias.aliasedType)
}
