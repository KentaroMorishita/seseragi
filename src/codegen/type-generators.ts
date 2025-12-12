/**
 * 型生成ユーティリティ
 * TypeScriptの型表現と文字列表現を生成する
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
  UnionType,
} from "../ast"

/**
 * 型をTypeScript型表現に変換
 * 例: Int -> number, Maybe<Int> -> Maybe<number>
 */
export function generateType(type: Type | undefined): string {
  if (!type) return "any"

  switch (type.kind) {
    case "PrimitiveType": {
      const primitiveType = type as PrimitiveType
      const typeMap: Record<string, string> = {
        Int: "number",
        Float: "number",
        Bool: "boolean",
        String: "string",
        Char: "string",
        Unit: "void",
        _: "any",
      }
      return typeMap[primitiveType.name] || primitiveType.name
    }

    case "FunctionType": {
      const funcType = type as FunctionType
      const paramType = generateType(funcType.paramType)
      const returnType = generateType(funcType.returnType)
      return `(arg: ${paramType}) => ${returnType}`
    }

    case "GenericType": {
      const genericType = type as GenericType
      if (genericType.typeArguments.length === 0) {
        return generateGenericTypeName(genericType.name)
      }
      const params = genericType.typeArguments
        .map((p) => generateType(p))
        .join(", ")
      return `${generateGenericTypeName(genericType.name)}<${params}>`
    }

    case "RecordType": {
      const recordType = type as RecordType
      if (recordType.fields.length === 0) {
        return "{}"
      }
      const fields = recordType.fields
        .map((field) => `${field.name}: ${generateType(field.type)}`)
        .join(", ")
      return `{ ${fields} }`
    }

    case "TupleType": {
      const tupleType = type as TupleType
      if (tupleType.elementTypes.length === 0) {
        return "[]"
      }
      const elements = tupleType.elementTypes
        .map((elementType) => generateType(elementType))
        .join(", ")
      return `[${elements}]`
    }

    case "StructType": {
      const structType = type as StructType
      return structType.name
    }

    case "UnionType": {
      const unionType = type as UnionType
      const types = unionType.types.map((t) => generateType(t)).join(" | ")
      return `(${types})`
    }

    case "IntersectionType": {
      const intersectionType = type as IntersectionType
      const types = intersectionType.types
        .map((t) => generateType(t))
        .join(" & ")
      return `(${types})`
    }

    case "VoidType":
      return "void"

    default:
      return "any"
  }
}

/**
 * 型をSeseragi型文字列表現に変換（is式用）
 * 例: Int -> "Int", Maybe<Int> -> "Maybe<Int>"
 */
export function generateTypeString(type: Type): string {
  switch (type.kind) {
    case "PrimitiveType":
      return (type as PrimitiveType).name

    case "VoidType":
      return "never"

    case "RecordType": {
      const rt = type as RecordType
      const fields = rt.fields
        .map((field) => `${field.name}: ${generateTypeString(field.type)}`)
        .join(", ")
      return `{ ${fields} }`
    }

    case "GenericType": {
      const gt = type as GenericType
      if (gt.typeArguments.length === 0) {
        return gt.name
      }
      const args = gt.typeArguments.map((t) => generateTypeString(t)).join(", ")
      return `${gt.name}<${args}>`
    }

    case "StructType":
      return (type as StructType).name

    case "FunctionType": {
      const ft = type as FunctionType
      return `(${generateTypeString(ft.paramType)} -> ${generateTypeString(ft.returnType)})`
    }

    case "TupleType": {
      const tt = type as TupleType
      const elements = tt.elementTypes
        .map((t) => generateTypeString(t))
        .join(", ")
      return `(${elements})`
    }

    case "UnionType": {
      const ut = type as UnionType
      const types = ut.types.map((t) => generateTypeString(t)).join(" | ")
      return types
    }

    case "IntersectionType": {
      const it = type as IntersectionType
      const types = it.types.map((t) => generateTypeString(t)).join(" & ")
      return types
    }

    default:
      return (type as any).name || "unknown"
  }
}

/**
 * ジェネリック型名をTypeScript型名に変換
 */
function generateGenericTypeName(name: string): string {
  const typeNameMap: Record<string, string> = {
    List: "List",
    Array: "Array",
    Maybe: "Maybe",
    Either: "Either",
    Task: "Task",
    Signal: "Signal",
  }
  return typeNameMap[name] || name
}
