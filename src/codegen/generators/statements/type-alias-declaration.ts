/**
 * Type Alias Declaration Generator - 型エイリアス宣言の生成
 */

import type {
  FunctionType,
  GenericType,
  IntersectionType,
  PrimitiveType,
  RecordType,
  StructType,
  TupleType,
  Type,
  TypeAliasDeclaration,
  UnionType,
} from "../../../ast"
import type { CodeGenContext } from "../../context"
import { getIndent, registerTypeAlias } from "../../context"
import { generateType } from "../../type-generators"

/**
 * 型エイリアス宣言をTypeScriptコードに変換
 *
 * 機能:
 * 1. 型エイリアスを型レジストリ（__typeRegistry）に登録
 * 2. ジェネリック型パラメータのサポート
 * 3. 型エイリアスをコンテキストに登録（内部追跡用）
 *
 * 生成例（基本）:
 * ```
 * __typeRegistry["UserId"] = {"kind":"primitive","name":"Int"};
 * type UserId = number;
 * ```
 *
 * 生成例（ジェネリック）:
 * ```
 * __typeRegistry["Result"] = {"kind":"generic","name":"Either","args":[...]};
 * type Result<E, T> = Either<E, T>;
 * ```
 */
export function generateTypeAliasDeclaration(
  ctx: CodeGenContext,
  typeAlias: TypeAliasDeclaration
): string {
  const indent = getIndent(ctx)

  // 型エイリアスをコンテキストに登録
  registerTypeAlias(ctx, typeAlias.name, typeAlias.aliasedType)

  // TypeScript型表現を生成
  const aliasedType = generateType(typeAlias.aliasedType)

  // 型情報をシリアライズして型レジストリに登録
  const typeInfo = serializeTypeInfo(typeAlias.aliasedType)
  const registryEntry = `__typeRegistry["${typeAlias.name}"] = ${JSON.stringify(typeInfo)};`

  // ジェネリック型パラメータがある場合は追加
  let typeParametersStr = ""
  if (typeAlias.typeParameters && typeAlias.typeParameters.length > 0) {
    const paramNames = typeAlias.typeParameters.map((param) => param.name)
    typeParametersStr = `<${paramNames.join(", ")}>`
  }

  return `${registryEntry}\n${indent}type ${typeAlias.name}${typeParametersStr} = ${aliasedType};`
}

/**
 * 型情報のシリアライゼーション
 *
 * ランタイム型チェック・is式・デバッグ用に型情報をJSON形式でシリアライズ
 *
 * 対応型:
 * - PrimitiveType: { kind: "primitive", name: "Int" }
 * - RecordType: { kind: "record", fields: {...} }
 * - TupleType: { kind: "tuple", elements: [...] }
 * - GenericType: { kind: "maybe", innerType: {...} } など
 * - UnionType: { kind: "union", types: [...] }
 * - IntersectionType: { kind: "intersection", types: [...] }
 * - FunctionType: { kind: "function", paramType: {...}, returnType: {...} }
 * - StructType: { kind: "record", fields: {...} }
 */
function serializeTypeInfo(type: Type): any {
  switch (type.kind) {
    case "PrimitiveType":
      return { kind: "primitive", name: (type as PrimitiveType).name }

    case "RecordType": {
      const recordType = type as RecordType
      const fields: Record<string, any> = {}
      for (const field of recordType.fields) {
        fields[field.name] = serializeTypeInfo(field.type)
      }
      return { kind: "record", fields }
    }

    case "TupleType": {
      const tupleType = type as TupleType
      return {
        kind: "tuple",
        elements: tupleType.elementTypes.map((t) => serializeTypeInfo(t)),
      }
    }

    case "GenericType": {
      const genericType = type as GenericType
      const args =
        genericType.typeArguments?.map((arg) => serializeTypeInfo(arg)) || []

      // 組み込み型の特別処理（ランタイムでの識別を容易にする）
      switch (genericType.name) {
        case "Maybe":
          return {
            kind: "maybe",
            innerType: args[0] || { kind: "primitive", name: "unknown" },
          }
        case "Either":
          return {
            kind: "either",
            leftType: args[0] || { kind: "primitive", name: "unknown" },
            rightType: args[1] || { kind: "primitive", name: "unknown" },
          }
        case "Array":
          return {
            kind: "array",
            elementType: args[0] || { kind: "primitive", name: "unknown" },
          }
        default:
          return { kind: "generic", name: genericType.name, args }
      }
    }

    case "UnionType": {
      const unionType = type as UnionType
      return {
        kind: "union",
        types: unionType.types.map((t) => serializeTypeInfo(t)),
      }
    }

    case "FunctionType": {
      const funcType = type as FunctionType
      return {
        kind: "function",
        paramType: serializeTypeInfo(funcType.paramType),
        returnType: serializeTypeInfo(funcType.returnType),
      }
    }

    case "StructType": {
      const structType = type as StructType
      const fields: Record<string, any> = {}
      for (const field of structType.fields) {
        fields[field.name] = serializeTypeInfo(field.type)
      }
      return { kind: "record", fields }
    }

    case "IntersectionType": {
      const intersectionType = type as IntersectionType
      return {
        kind: "intersection",
        types: intersectionType.types.map((t) => serializeTypeInfo(t)),
      }
    }

    default:
      return { kind: "unknown", originalKind: type.kind }
  }
}
