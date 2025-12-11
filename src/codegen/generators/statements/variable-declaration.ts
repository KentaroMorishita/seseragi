/**
 * Variable Declaration Generator - 変数宣言の生成
 */

import type { VariableDeclaration } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { getIndent } from "../../context"
import { generateExpression } from "../dispatcher"
import { sanitizeIdentifier } from "../../helpers"

/**
 * 変数宣言をTypeScriptコードに変換
 *
 * 基本形式:
 * - `const ${name} = ${value};` (デフォルト・イミュータブル)
 *
 * 将来の拡張:
 * - 型注釈のサポート
 * - 型推論結果との統合
 * - 型エイリアスの __typename 付与
 * - ミュータブル変数のサポート（`let`）
 */
export function generateVariableDeclaration(
  ctx: CodeGenContext,
  varDecl: VariableDeclaration
): string {
  const indent = getIndent(ctx)
  const name = sanitizeIdentifier(varDecl.name)
  const value = generateExpression(ctx, varDecl.initializer)

  // 基本形式: const宣言
  // TODO: 型注釈のサポート (varDecl.type)
  // TODO: 型推論結果との統合 (ctx.typeInferenceResult)
  // TODO: 型エイリアスの __typename 付与
  return `${indent}const ${name} = ${value};`
}
