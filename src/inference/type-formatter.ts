/**
 * 型フォーマッターユーティリティ (Type Formatter) for Seseragi Language
 *
 * 型を文字列に変換するユーティリティ関数
 */

import type * as AST from "../ast"
import type { PolymorphicTypeVariable, TypeVariable } from "./type-variables"

/**
 * 型を文字列に変換（内部用、シンプル版）
 */
export function formatType(
  type: AST.Type | TypeVariable | PolymorphicTypeVariable | null | undefined
): string {
  if (!type) {
    return "null"
  }

  switch (type.kind) {
    case "PrimitiveType":
      return (type as AST.PrimitiveType).name
    case "FunctionType": {
      const ft = type as AST.FunctionType
      return `(${formatType(ft.paramType)} -> ${formatType(ft.returnType)})`
    }
    case "TypeVariable":
      return (type as TypeVariable).name
    case "PolymorphicTypeVariable":
      return (type as PolymorphicTypeVariable).name
    case "GenericType": {
      const gt = type as AST.GenericType
      if (gt.typeArguments.length > 0) {
        return `${gt.name}<${gt.typeArguments.map((t) => formatType(t)).join(", ")}>`
      }
      return gt.name
    }
    case "TupleType": {
      const tt = type as AST.TupleType
      return `(${tt.elementTypes.map((t) => formatType(t)).join(", ")})`
    }
    case "RecordType": {
      const rt = type as AST.RecordType
      const fields = rt.fields
        .map((f) => `${f.name}: ${formatType(f.type)}`)
        .join(", ")
      return `{ ${fields} }`
    }
    case "StructType": {
      const st = type as AST.StructType
      return st.name
    }
    case "VoidType":
      return "Void"
    case "UnionType": {
      const ut = type as AST.UnionType
      const types = ut.types.map((t) => formatType(t)).join(" | ")
      return `(${types})`
    }
    case "IntersectionType": {
      const it = type as AST.IntersectionType
      const types = it.types.map((t) => formatType(t)).join(" & ")
      return `(${types})`
    }
    default:
      return `UnknownType(${type.kind})`
  }
}

/**
 * 型を文字列に変換（公開API用）
 */
export function typeToString(type: AST.Type): string {
  switch (type.kind) {
    case "PrimitiveType":
      return (type as AST.PrimitiveType).name

    case "TypeVariable":
      return (type as TypeVariable).name

    case "PolymorphicTypeVariable":
      return `'${(type as PolymorphicTypeVariable).name}`

    case "FunctionType": {
      const ft = type as AST.FunctionType
      return `(${typeToString(ft.paramType)} -> ${typeToString(ft.returnType)})`
    }

    case "GenericType": {
      const gt = type as AST.GenericType
      const args = gt.typeArguments.map((t) => typeToString(t)).join(", ")
      return `${gt.name}<${args}>`
    }

    case "RecordType": {
      const rt = type as AST.RecordType
      const fields = rt.fields
        .map((field) => `${field.name}: ${typeToString(field.type)}`)
        .join(", ")
      return `{${fields}}`
    }

    case "TupleType": {
      const tupleType = type as AST.TupleType
      const elements = tupleType.elementTypes
        .map((elementType) => typeToString(elementType))
        .join(", ")
      return `(${elements})`
    }

    case "StructType": {
      const st = type as AST.StructType
      return st.name
    }

    case "VoidType":
      return "Void"

    case "UnionType": {
      const ut = type as AST.UnionType
      const types = ut.types.map((t) => typeToString(t)).join(" | ")
      return `(${types})`
    }

    case "IntersectionType": {
      const it = type as AST.IntersectionType
      const types = it.types.map((t) => typeToString(t)).join(" & ")
      return `(${types})`
    }

    default:
      return "Unknown"
  }
}

/**
 * 正規化された型の文字列表現を生成（比較用）
 * Union型の要素はソートされる
 */
export function typeToCanonicalString(type: AST.Type): string {
  switch (type.kind) {
    case "PrimitiveType":
      return (type as AST.PrimitiveType).name
    case "GenericType": {
      const gt = type as AST.GenericType
      const args = gt.typeArguments
        .map((t) => typeToCanonicalString(t))
        .join(", ")
      return `${gt.name}<${args}>`
    }
    case "UnionType": {
      const ut = type as AST.UnionType
      const types = ut.types.map((t) => typeToCanonicalString(t)).sort()
      return types.join(" | ")
    }
    default:
      return typeToString(type)
  }
}
