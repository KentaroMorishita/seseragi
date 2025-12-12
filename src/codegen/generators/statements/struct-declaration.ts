/**
 * Struct Declaration Generator - 構造体宣言の生成
 */

import type { StructDeclaration } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { getIndent } from "../../context"
import { generateType } from "../../type-generators"
import { isMaybeType } from "../../type-utils"

/**
 * 構造体宣言をTypeScriptコードに変換
 *
 * 構造体はclassとして生成され、以下の特徴を持つ:
 * - フィールド定義
 * - コンストラクタ（オブジェクトリテラルを受け取る）
 * - Maybe型フィールドはオプショナルパラメータになり、デフォルト値Nothingが設定される
 *
 * 例:
 * ```seseragi
 * struct Point {
 *   x: Int,
 *   y: Int
 * }
 * ```
 * ↓
 * ```typescript
 * class Point {
 *   x: number;
 *   y: number;
 *
 *   constructor(fields: { x: number, y: number }) {
 *     this.x = fields.x;
 *     this.y = fields.y;
 *   }
 * }
 * ```
 */
export function generateStructDeclaration(
  ctx: CodeGenContext,
  structDecl: StructDeclaration
): string {
  const indent = getIndent(ctx)

  // フィールド定義
  const fieldDeclarations = structDecl.fields
    .map((f) => `${indent}  ${f.name}: ${generateType(f.type)};`)
    .join("\n")

  // コンストラクタ引数の型定義
  const constructorParamType = structDecl.fields
    .map((f) => {
      // Maybe型フィールドはオプショナルにする
      const isOptional = isMaybeType(ctx, f.type) ? "?" : ""
      return `${f.name}${isOptional}: ${generateType(f.type)}`
    })
    .join(", ")

  // コンストラクタ本体でデフォルト値を適用
  const fieldAssignments = structDecl.fields
    .map((f) => {
      if (isMaybeType(ctx, f.type)) {
        return `${indent}    this.${f.name} = fields.${f.name} ?? Nothing;`
      } else {
        return `${indent}    this.${f.name} = fields.${f.name};`
      }
    })
    .join("\n")

  return `${indent}class ${structDecl.name} {
${fieldDeclarations}

${indent}  constructor(fields: { ${constructorParamType} }) {
${fieldAssignments}
${indent}  }
${indent}}`
}
