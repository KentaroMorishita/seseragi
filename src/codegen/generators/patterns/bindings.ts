/**
 * パターンバインディング生成のディスパッチャー
 *
 * パターンマッチングで変数をバインディングするコードを生成する
 */

import type { IdentifierPattern, Pattern } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { sanitizeIdentifier } from "../../helpers"

/**
 * レガシーコードジェネレーターへフォールバック
 */
function fallbackToLegacy(
  ctx: CodeGenContext,
  pattern: Pattern,
  valueVar: string
): string {
  if ((ctx as any).legacyGenerator) {
    return (ctx as any).legacyGenerator.generatePatternBindings(
      pattern,
      valueVar
    )
  }
  throw new Error(
    `Pattern bindings generation not implemented for: ${pattern.kind}`
  )
}

/**
 * パターンバインディングを生成
 *
 * @param ctx - コード生成コンテキスト
 * @param pattern - パターン
 * @param valueVar - バインディング元の変数名
 * @returns バインディングコードの文字列
 */
export function generatePatternBindings(
  ctx: CodeGenContext,
  pattern: Pattern,
  valueVar: string
): string {
  if (!pattern) return ""

  switch (pattern.kind) {
    case "LiteralPattern":
      // リテラルパターンはバインディングなし
      return ""

    case "IdentifierPattern": {
      const idPattern = pattern as IdentifierPattern
      // ワイルドカード "_" はバインディングなし
      if (idPattern.name === "_") {
        return ""
      }
      // 通常の識別子はバインディング
      return `const ${sanitizeIdentifier(idPattern.name)} = ${valueVar};\n`
    }

    case "WildcardPattern":
      // ワイルドカードパターンはバインディングなし
      return ""

    case "ConstructorPattern":
      return fallbackToLegacy(ctx, pattern, valueVar)

    case "TuplePattern":
      return fallbackToLegacy(ctx, pattern, valueVar)

    case "OrPattern":
      return fallbackToLegacy(ctx, pattern, valueVar)

    case "GuardPattern":
      return fallbackToLegacy(ctx, pattern, valueVar)

    case "ListSugarPattern":
      return fallbackToLegacy(ctx, pattern, valueVar)

    case "ArrayPattern":
      return fallbackToLegacy(ctx, pattern, valueVar)

    default:
      return fallbackToLegacy(ctx, pattern, valueVar)
  }
}
