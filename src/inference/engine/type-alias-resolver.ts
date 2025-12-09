/**
 * 型エイリアス解決モジュール
 *
 * PrimitiveType名やGenericType名から実際の型定義を解決する
 */

import * as AST from "../../ast"
import { mergeRecordTypes } from "../type-comparison"
import type { PolymorphicTypeVariable } from "../type-variables"
import type { InferenceContext } from "./context"

/**
 * 型エイリアスを解決する
 * 循環参照を検出し、適切に処理する
 */
export function resolveTypeAlias(
  ctx: InferenceContext,
  type: AST.Type,
  visited: Set<string> = new Set()
): AST.Type {
  if (type.kind === "PrimitiveType") {
    const pt = type as AST.PrimitiveType

    // 循環参照チェック
    if (visited.has(pt.name)) {
      return type
    }

    // 環境から型エイリアスを検索
    const aliasedType = ctx.environment.get(pt.name)
    if (aliasedType) {
      const newVisited = new Set(visited)
      newVisited.add(pt.name)
      return resolveTypeAlias(ctx, aliasedType, newVisited)
    }
  } else if (type.kind === "GenericType") {
    const genericType = type as AST.GenericType

    // 循環参照チェック
    if (visited.has(genericType.name)) {
      return type
    }

    // 組み込み型は型引数を解決して返す
    const builtinTypes = ["Maybe", "Either", "List", "Array", "Signal", "Task"]
    if (builtinTypes.includes(genericType.name)) {
      const resolvedTypeArgs = genericType.typeArguments.map((arg) =>
        resolveTypeAlias(ctx, arg, visited)
      )
      return new AST.GenericType(
        genericType.name,
        resolvedTypeArgs,
        genericType.line,
        genericType.column
      )
    }

    // ジェネリック型エイリアスをチェック
    const typeAlias = ctx.typeAliases.get(genericType.name)
    if (
      typeAlias?.kind === "TypeAliasDeclaration" &&
      (typeAlias as AST.TypeAliasDeclaration).typeParameters
    ) {
      const genericAlias = typeAlias as AST.TypeAliasDeclaration

      // 型パラメータと型引数のマッピングを作成
      const instantiatedType = substituteTypeVariablesInGenericAlias(
        genericAlias.aliasedType,
        genericAlias.typeParameters ?? [],
        genericType.typeArguments
      )

      const newVisited = new Set(visited)
      newVisited.add(genericType.name)
      return resolveTypeAlias(ctx, instantiatedType, newVisited)
    }
  } else if (type.kind === "IntersectionType") {
    const intersectionType = type as AST.IntersectionType
    const resolvedTypes = intersectionType.types.map((t) =>
      resolveTypeAlias(ctx, t, visited)
    )

    // 全てがRecord型なら統合
    if (resolvedTypes.every((t) => t.kind === "RecordType")) {
      return mergeRecordTypes(resolvedTypes as AST.RecordType[])
    }

    return new AST.IntersectionType(resolvedTypes, type.line, type.column)
  } else if (type.kind === "UnionType") {
    const unionType = type as AST.UnionType
    const resolvedTypes = unionType.types.map((t) =>
      resolveTypeAlias(ctx, t, visited)
    )
    return new AST.UnionType(resolvedTypes, type.line, type.column)
  }

  return type
}

/**
 * ジェネリック型エイリアス内の型パラメータを具体的な型引数に置換
 */
function substituteTypeVariablesInGenericAlias(
  type: AST.Type,
  typeParameters: AST.TypeParameter[],
  typeArguments: AST.Type[]
): AST.Type {
  // パラメータ名→型引数のマッピングを作成
  const mapping = new Map<string, AST.Type>()
  for (let i = 0; i < typeParameters.length && i < typeArguments.length; i++) {
    const param = typeParameters[i]
    const arg = typeArguments[i]
    if (param && arg) {
      mapping.set(param.name, arg)
    }
  }

  return substituteInType(type, mapping)
}

/**
 * 型内の型変数を置換
 */
function substituteInType(
  type: AST.Type,
  mapping: Map<string, AST.Type>
): AST.Type {
  switch (type.kind) {
    case "PrimitiveType": {
      const pt = type as AST.PrimitiveType
      const substituted = mapping.get(pt.name)
      return substituted ?? type
    }

    case "PolymorphicTypeVariable": {
      const ptv = type as PolymorphicTypeVariable
      const substituted = mapping.get(ptv.name)
      return substituted ?? type
    }

    case "FunctionType": {
      const ft = type as AST.FunctionType
      return new AST.FunctionType(
        substituteInType(ft.paramType, mapping),
        substituteInType(ft.returnType, mapping),
        ft.line,
        ft.column
      )
    }

    case "GenericType": {
      const gt = type as AST.GenericType
      return new AST.GenericType(
        gt.name,
        gt.typeArguments.map((arg) => substituteInType(arg, mapping)),
        gt.line,
        gt.column
      )
    }

    case "TupleType": {
      const tt = type as AST.TupleType
      return new AST.TupleType(
        tt.elementTypes.map((elem) => substituteInType(elem, mapping)),
        tt.line,
        tt.column
      )
    }

    case "RecordType": {
      const rt = type as AST.RecordType
      return new AST.RecordType(
        rt.fields.map((field) => ({
          ...field,
          type: substituteInType(field.type, mapping),
        })),
        rt.line,
        rt.column
      )
    }

    case "UnionType": {
      const ut = type as AST.UnionType
      return new AST.UnionType(
        ut.types.map((t) => substituteInType(t, mapping)),
        ut.line,
        ut.column
      )
    }

    case "IntersectionType": {
      const it = type as AST.IntersectionType
      return new AST.IntersectionType(
        it.types.map((t) => substituteInType(t, mapping)),
        it.line,
        it.column
      )
    }

    default:
      return type
  }
}

/**
 * 再帰的に型エイリアスを解決（型名から直接解決）
 */
export function resolveTypeAliasRecursively(
  ctx: InferenceContext,
  typeName: string,
  visited: Set<string> = new Set()
): AST.Type | null {
  if (visited.has(typeName)) {
    return null
  }

  const aliasData = ctx.typeAliases.get(typeName)
  if (!aliasData) {
    return null
  }

  // TypeAliasDeclarationの場合は、その中のaliasedTypeを取得
  let aliasedType: AST.Type
  if (aliasData.kind === "TypeAliasDeclaration") {
    aliasedType = (aliasData as AST.TypeAliasDeclaration).aliasedType
  } else {
    aliasedType = aliasData
  }

  // RecordTypeなら直接返す
  if (aliasedType.kind === "RecordType") {
    return aliasedType
  }

  // 別のPrimitiveType（型エイリアス）なら再帰的に解決
  if (aliasedType.kind === "PrimitiveType") {
    visited.add(typeName)
    const nextTypeName = (aliasedType as AST.PrimitiveType).name
    return resolveTypeAliasRecursively(ctx, nextTypeName, visited)
  }

  return aliasedType
}
