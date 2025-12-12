/**
 * パターン条件生成のディスパッチャー
 *
 * パターンマッチングの条件式を生成する
 */

import type { LiteralPattern, Pattern } from "../../../ast"
import type { CodeGenContext } from "../../context"

/**
 * レガシーコードジェネレーターへフォールバック
 */
function fallbackToLegacy(
  ctx: CodeGenContext,
  pattern: Pattern,
  valueVar: string
): string {
  if ((ctx as any).legacyGenerator) {
    return (ctx as any).legacyGenerator.generatePatternCondition(
      pattern,
      valueVar
    )
  }
  throw new Error(
    `Pattern condition generation not implemented for: ${pattern.kind}`
  )
}

/**
 * パターン条件を生成
 *
 * @param ctx - コード生成コンテキスト
 * @param pattern - パターン
 * @param valueVar - マッチング対象の変数名
 * @returns 条件式の文字列
 */
export function generatePatternCondition(
  ctx: CodeGenContext,
  pattern: Pattern,
  valueVar: string
): string {
  if (!pattern) {
    return "true" // ワイルドカードパターン
  }

  switch (pattern.kind) {
    case "LiteralPattern": {
      const lit = pattern as LiteralPattern
      if (typeof lit.value === "string") {
        return `${valueVar} === ${JSON.stringify(lit.value)}`
      }
      return `${valueVar} === ${lit.value}`
    }

    case "IdentifierPattern":
      // 識別子パターンは常にマッチ（値をバインドする）
      return "true"

    case "WildcardPattern":
      // ワイルドカードパターンは常にマッチ
      return "true"

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
