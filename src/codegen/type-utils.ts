/**
 * 型関連のユーティリティ関数
 * 型述語と型文字列生成
 */

import {
  type Type,
  type GenericType,
  type PrimitiveType,
  type RecordType,
  type TupleType,
  type StructType,
  type IntersectionType,
  type Identifier,
} from "../ast"
import type { CodeGenContext } from "./context"

// ============================================================
// 型述語 (Type Predicates)
// ============================================================

/**
 * Maybe型かどうかをチェック
 */
export function isMaybeType(
  ctx: CodeGenContext,
  type: Type | undefined
): boolean {
  if (!type) return false

  // 型推論結果がある場合は置換を適用
  if (ctx.typeInferenceResult?.substitution) {
    const resolvedType = ctx.typeInferenceResult.substitution.apply(type)
    if (
      resolvedType &&
      resolvedType.kind === "GenericType" &&
      (resolvedType as GenericType).name === "Maybe"
    ) {
      return true
    }
  }

  // 直接GenericTypeの場合もチェック
  if (type.kind === "GenericType" && (type as GenericType).name === "Maybe") {
    return true
  }

  return false
}

/**
 * Either型かどうかをチェック
 */
export function isEitherType(
  ctx: CodeGenContext,
  type: Type | undefined
): boolean {
  if (!type) return false

  if (ctx.typeInferenceResult?.substitution) {
    const resolvedType = ctx.typeInferenceResult.substitution.apply(type)
    if (
      resolvedType &&
      resolvedType.kind === "GenericType" &&
      (resolvedType as GenericType).name === "Either"
    ) {
      return true
    }
  }

  if (type.kind === "GenericType" && (type as GenericType).name === "Either") {
    return true
  }

  return false
}

/**
 * Task型かどうかをチェック
 */
export function isTaskType(
  ctx: CodeGenContext,
  type: Type | undefined
): boolean {
  if (!type) return false

  if (ctx.typeInferenceResult?.substitution) {
    const resolvedType = ctx.typeInferenceResult.substitution.apply(type)
    if (
      resolvedType &&
      resolvedType.kind === "GenericType" &&
      (resolvedType as GenericType).name === "Task"
    ) {
      return true
    }
  }

  if (type.kind === "GenericType" && (type as GenericType).name === "Task") {
    return true
  }

  return false
}

/**
 * Signal型かどうかをチェック
 */
export function isSignalType(
  ctx: CodeGenContext,
  type: Type | undefined
): boolean {
  if (!type) return false

  // 直接GenericTypeの場合をチェック（最優先）
  if (type.kind === "GenericType" && (type as GenericType).name === "Signal") {
    return true
  }

  if (ctx.typeInferenceResult?.substitution) {
    const resolvedType = ctx.typeInferenceResult.substitution.apply(type)
    if (
      resolvedType &&
      resolvedType.kind === "GenericType" &&
      (resolvedType as GenericType).name === "Signal"
    ) {
      return true
    }
  }

  return false
}

/**
 * List型かどうかをチェック
 */
export function isListType(
  ctx: CodeGenContext,
  type: Type | undefined
): boolean {
  if (!type) return false

  if (ctx.typeInferenceResult?.substitution) {
    const resolvedType = ctx.typeInferenceResult.substitution.apply(type)
    if (
      resolvedType &&
      resolvedType.kind === "GenericType" &&
      (resolvedType as GenericType).name === "List"
    ) {
      return true
    }
  }

  if (type.kind === "GenericType" && (type as GenericType).name === "List") {
    return true
  }

  return false
}

/**
 * Array型かどうかをチェック
 */
export function isArrayType(
  ctx: CodeGenContext,
  type: Type | undefined
): boolean {
  if (!type) return false

  if (ctx.typeInferenceResult?.substitution) {
    const resolvedType = ctx.typeInferenceResult.substitution.apply(type)
    if (
      resolvedType &&
      resolvedType.kind === "GenericType" &&
      (resolvedType as GenericType).name === "Array"
    ) {
      return true
    }
  }

  if (type.kind === "GenericType" && (type as GenericType).name === "Array") {
    return true
  }

  return false
}

/**
 * Tuple型かどうかをチェック
 */
export function isTupleType(
  ctx: CodeGenContext,
  type: Type | undefined
): boolean {
  if (!type) return false

  if (ctx.typeInferenceResult?.substitution) {
    const resolvedType = ctx.typeInferenceResult.substitution.apply(type)
    if (resolvedType && resolvedType.kind === "TupleType") {
      return true
    }
  }

  if (type.kind === "TupleType") {
    return true
  }

  return false
}

/**
 * Int型かどうかをチェック
 */
export function isIntType(type: Type | undefined): boolean {
  return (
    type?.kind === "PrimitiveType" && (type as PrimitiveType).name === "Int"
  )
}

/**
 * プリミティブ型かどうかをチェック
 */
export function isPrimitiveType(type: Type | undefined): boolean {
  if (!type || type.kind !== "PrimitiveType") {
    return false
  }
  const primitiveTypes = ["Int", "Float", "Bool", "String", "Char", "Unit"]
  return primitiveTypes.includes((type as PrimitiveType).name)
}

/**
 * 型エイリアスかどうかをチェック
 */
export function isTypeAlias(ctx: CodeGenContext, type: Type): boolean {
  // 交差型（IntersectionType）は型エイリアスではない
  if (type.kind === "IntersectionType") {
    return false
  }
  if (type.kind === "Identifier") {
    const identifier = type as Identifier
    return ctx.typeAliases.has(identifier.name)
  }
  if (type.kind === "PrimitiveType") {
    const primitiveType = type as PrimitiveType
    return ctx.typeAliases.has(primitiveType.name)
  }
  return false
}

/**
 * 交差型エイリアスかどうかをチェック
 */
export function isIntersectionTypeAlias(
  ctx: CodeGenContext,
  type: Type
): boolean {
  if (type.kind === "IntersectionType") {
    return true
  }

  // 型エイリアスの場合、解決した型を確認
  if (type.kind === "PrimitiveType") {
    const aliasedType = ctx.typeAliases.get((type as PrimitiveType).name)
    if (aliasedType && aliasedType.kind === "IntersectionType") {
      return true
    }
  }

  return false
}

/**
 * レコード系の型かどうかを判定
 */
export function isRecordLikeType(type: Type): boolean {
  if (type.kind === "RecordType" || type.kind === "StructType") {
    return true
  }

  // IntersectionType（&で結合された型）の場合、すべての型がレコード系かチェック
  if (type.kind === "IntersectionType") {
    const intersectionType = type as IntersectionType
    return intersectionType.types.some((t) => isRecordLikeType(t))
  }

  return false
}

/**
 * 型エイリアス名を取得
 */
export function getTypeAliasName(type: Type): string | null {
  if (type.kind === "PrimitiveType") {
    return (type as PrimitiveType).name
  }
  if (type.kind === "Identifier") {
    return (type as Identifier).name
  }
  return null
}

// ============================================================
// 型文字列生成
// ============================================================

/**
 * 型を構造的型文字列に変換
 */
export function typeToStructuralString(type: Type): string {
  switch (type.kind) {
    case "PrimitiveType":
      return (type as PrimitiveType).name
    case "VoidType":
      return "Void"
    case "GenericType": {
      const genericType = type as GenericType
      if (genericType.typeArguments && genericType.typeArguments.length > 0) {
        const args = genericType.typeArguments
          .map((arg) => typeToStructuralString(arg))
          .join(", ")
        return `${genericType.name}<${args}>`
      }
      return genericType.name
    }
    case "RecordType": {
      const recordType = type as RecordType
      const fields = recordType.fields
        .map((field) => `${field.name}: ${typeToStructuralString(field.type)}`)
        .sort() // キーをソートして順序統一
        .join(", ")
      return `{ ${fields} }`
    }
    case "TupleType": {
      const tupleType = type as TupleType
      const elements = tupleType.elementTypes
        .map((t) => typeToStructuralString(t))
        .join(", ")
      return `(${elements})`
    }
    case "StructType": {
      const structType = type as StructType
      const fields = structType.fields
        .map((field) => `${field.name}: ${typeToStructuralString(field.type)}`)
        .sort() // キーをソートして順序統一
        .join(", ")
      return `${structType.name} { ${fields} }`
    }
    case "FunctionType":
      return "function" // 関数型は簡略化
    default:
      return "unknown"
  }
}

/**
 * 構造的型文字列に一致する型エイリアスを検索
 */
export function findMatchingAliases(
  ctx: CodeGenContext,
  structuralType: string
): string[] {
  const aliases: string[] = []
  for (const [aliasName, aliasType] of ctx.typeAliases) {
    const aliasStructural = typeToStructuralString(aliasType)
    if (aliasStructural === structuralType) {
      aliases.push(aliasName)
    }
  }
  return aliases
}
