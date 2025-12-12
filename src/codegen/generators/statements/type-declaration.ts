/**
 * Type Declaration Generator - ADT（代数的データ型）宣言の生成
 */

import { GenericType, PrimitiveType, type TypeDeclaration } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { getIndent } from "../../context"
import { generateType } from "../../type-generators"

/**
 * 型宣言をTypeScriptコードに変換
 *
 * Seseragiの型宣言は以下2種類を表現:
 * 1. Union型（ADT）: バリアント毎にコンストラクタ関数を生成
 * 2. Struct型: TypeScript interfaceとして生成
 *
 * Union型の判定:
 * - フィールド型が Unit または Tuple<...> の場合はADT
 *
 * 生成例（Union型）:
 * ```
 * type Maybe = { type: 'Just', data: [number] } | { type: 'Nothing' };
 * const Nothing = { type: 'Nothing' as const };
 * function Just(data0: number) { return { type: 'Just' as const, data: [data0] }; }
 * ```
 *
 * 生成例（Struct型）:
 * ```
 * type Point = {
 *   x: number;
 *   y: number
 * };
 * ```
 */
export function generateTypeDeclaration(
  ctx: CodeGenContext,
  typeDecl: TypeDeclaration
): string {
  const indent = getIndent(ctx)

  // フィールドがない場合は空の型宣言
  if (!typeDecl.fields || typeDecl.fields.length === 0) {
    return `${indent}type ${typeDecl.name} = never; // Empty type declaration`
  }

  // Union型（ADT）かStruct型かを判定
  const isUnionType = typeDecl.fields.some(
    (f) =>
      (f.type instanceof PrimitiveType && f.type.name === "Unit") ||
      (f.type instanceof GenericType && f.type.name === "Tuple")
  )

  if (isUnionType) {
    return generateUnionType(ctx, typeDecl, indent)
  } else {
    return generateStructType(ctx, typeDecl, indent)
  }
}

/**
 * Union型（ADT）の生成
 *
 * 各バリアントのコンストラクタ関数を生成し、
 * TypeScript discriminated unionとして表現
 */
function generateUnionType(
  _ctx: CodeGenContext,
  typeDecl: TypeDeclaration,
  indent: string
): string {
  // 各バリアントの型定義を生成
  const variants = typeDecl.fields
    .map((field) => {
      if (field.type instanceof PrimitiveType && field.type.name === "Unit") {
        // Unitバリアント: データなし
        return `{ type: '${field.name}' }`
      } else if (
        field.type instanceof GenericType &&
        field.type.name === "Tuple"
      ) {
        // Tupleバリアント: データあり（配列形式）
        const dataTypes = field.type.typeArguments
          .map((t) => generateType(t))
          .join(" | ")
        return `{ type: '${field.name}', data: Array<${dataTypes}> }`
      } else {
        // フォールバック: 単一データを配列化
        return `{ type: '${field.name}', data: [${generateType(field.type)}] }`
      }
    })
    .join(" | ")

  // コンストラクタ関数を生成
  const constructors = typeDecl.fields
    .map((field) => {
      if (field.type instanceof PrimitiveType && field.type.name === "Unit") {
        // Unitバリアント: 定数オブジェクト
        return `${indent}const ${field.name} = { type: '${field.name}' as const };`
      } else if (
        field.type instanceof GenericType &&
        field.type.name === "Tuple"
      ) {
        // Tupleバリアント: 関数コンストラクタ（複数引数）
        const params = field.type.typeArguments
          .map((t, i) => `data${i}: ${generateType(t)}`)
          .join(", ")
        const dataArray = field.type.typeArguments
          .map((_, i) => `data${i}`)
          .join(", ")
        return `${indent}function ${field.name}(${params}) { return { type: '${field.name}' as const, data: [${dataArray}] }; }`
      } else {
        // フォールバック: 単一引数コンストラクタ
        return `${indent}function ${field.name}(data: ${generateType(field.type)}) { return { type: '${field.name}' as const, data: [data] }; }`
      }
    })
    .join("\n")

  return `${indent}type ${typeDecl.name} = ${variants};\n\n${constructors}`
}

/**
 * Struct型の生成
 *
 * TypeScript interfaceとして生成
 */
function generateStructType(
  _ctx: CodeGenContext,
  typeDecl: TypeDeclaration,
  indent: string
): string {
  const fields = typeDecl.fields
    .map((f) => `  ${f.name}: ${generateType(f.type)}`)
    .join(";\n")

  return `${indent}type ${typeDecl.name} = {\n${fields}\n};`
}
