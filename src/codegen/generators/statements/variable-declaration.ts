/**
 * Variable Declaration Generator - 変数宣言の生成
 */

import type {
  GenericType,
  PrimitiveType,
  RecordType,
  StructType,
  TupleType,
  Type,
  VariableDeclaration,
} from "../../../ast"
import {
  type CodeGenContext,
  getIndent,
  registerVariableType,
} from "../../context"
import { sanitizeIdentifier } from "../../helpers"
import { generateExpression } from "../dispatcher"

/**
 * 変数宣言をTypeScriptコードに変換
 *
 * 基本形式:
 * - `const ${name} = ${value};` (デフォルト・イミュータブル)
 */
export function generateVariableDeclaration(
  ctx: CodeGenContext,
  varDecl: VariableDeclaration
): string {
  const indent = getIndent(ctx)
  const name = sanitizeIdentifier(varDecl.name)
  const value = generateExpression(ctx, varDecl.initializer)

  // 変数型情報を登録（型推論結果または明示的型注釈から）
  if (varDecl.type) {
    registerVariableTypeInfo(ctx, varDecl.name, varDecl.type)
  } else if (ctx.typeInferenceResult) {
    // 型推論結果から型情報を取得
    const inferredType = ctx.typeInferenceResult.environment.get(varDecl.name)
    if (inferredType) {
      registerVariableTypeInfo(ctx, varDecl.name, inferredType)
    }
  }

  // 型エイリアスが使われている場合、__typename を付与
  if (varDecl.type && isTypeAlias(ctx, varDecl.type)) {
    const typeName = getTypeAliasName(varDecl.type)
    if (typeName) {
      return `${indent}const ${name} = { ...${value}, __typename: "${typeName}" };`
    }
  }

  return `${indent}const ${name} = ${value};`
}

/**
 * 変数型情報を登録
 */
function registerVariableTypeInfo(
  ctx: CodeGenContext,
  variableName: string,
  type: Type
): void {
  let displayType: string

  // 型エイリアスの場合は構造型を表示
  if (isTypeAlias(ctx, type)) {
    const typeName = (type as any).name || type.kind
    const aliasedType = ctx.typeAliases.get(typeName)
    if (aliasedType) {
      displayType = typeToStructuralString(aliasedType)
    } else {
      displayType = typeToStructuralString(type)
    }
  } else {
    displayType = typeToStructuralString(type)
  }

  // 型エイリアスを検索
  const aliases = findMatchingAliases(ctx, displayType)
  registerVariableType(ctx, variableName, displayType, aliases)
}

/**
 * 型エイリアスかどうかチェック
 */
function isTypeAlias(ctx: CodeGenContext, type: Type): boolean {
  if (type.kind === "IntersectionType") {
    return false
  }
  if (type.kind === "Identifier" || type.kind === "PrimitiveType") {
    const name = (type as any).name
    return ctx.typeAliases.has(name)
  }
  return false
}

/**
 * 型エイリアス名を取得
 */
function getTypeAliasName(type: Type): string | null {
  if (type.kind === "Identifier" || type.kind === "PrimitiveType") {
    return (type as any).name || null
  }
  return null
}

/**
 * 型を構造的型文字列に変換
 */
function typeToStructuralString(type: Type): string {
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
        .sort()
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
        .sort()
        .join(", ")
      return `${structType.name} { ${fields} }`
    }
    case "FunctionType":
      return "function"
    default:
      return "unknown"
  }
}

/**
 * 表示型に一致する型エイリアスを検索
 */
function findMatchingAliases(
  ctx: CodeGenContext,
  displayType: string
): string[] {
  const aliases: string[] = []
  for (const [aliasName, aliasedType] of ctx.typeAliases) {
    const aliasDisplayType = typeToStructuralString(aliasedType)
    if (aliasDisplayType === displayType) {
      aliases.push(aliasName)
    }
  }
  return aliases
}
