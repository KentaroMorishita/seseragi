/**
 * パターンバインディング生成
 *
 * パターンマッチングで変数をバインディングするコードを生成する
 */

import type {
  ArrayPattern,
  ConstructorPattern,
  GuardPattern,
  IdentifierPattern,
  ListSugarPattern,
  OrPattern,
  Pattern,
  TuplePattern,
} from "../../../ast"
import type { CodeGenContext } from "../../context"
import { isBuiltinConstructor, sanitizeIdentifier } from "../../helpers"

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

    case "IdentifierPattern":
      return generateIdentifierPatternBindings(
        pattern as IdentifierPattern,
        valueVar
      )

    case "WildcardPattern":
      // ワイルドカードパターンはバインディングなし
      return ""

    case "ConstructorPattern":
      return generateConstructorPatternBindings(
        ctx,
        pattern as ConstructorPattern,
        valueVar
      )

    case "TuplePattern":
      return generateTuplePatternBindings(
        ctx,
        pattern as TuplePattern,
        valueVar
      )

    case "OrPattern":
      return generateOrPatternBindings(ctx, pattern as OrPattern, valueVar)

    case "GuardPattern":
      return generateGuardPatternBindings(
        ctx,
        pattern as GuardPattern,
        valueVar
      )

    case "ListSugarPattern":
      return generateListSugarPatternBindings(
        ctx,
        pattern as ListSugarPattern,
        valueVar
      )

    case "ArrayPattern":
      return generateArrayPatternBindings(
        ctx,
        pattern as ArrayPattern,
        valueVar
      )

    default:
      return ""
  }
}

/**
 * 識別子パターンのバインディング生成
 */
function generateIdentifierPatternBindings(
  pattern: IdentifierPattern,
  valueVar: string
): string {
  // ワイルドカード "_" はバインディングなし
  if (pattern.name === "_") {
    return ""
  }
  return `const ${sanitizeIdentifier(pattern.name)} = ${valueVar};\n`
}

/**
 * コンストラクタパターンのバインディング生成
 */
function generateConstructorPatternBindings(
  ctx: CodeGenContext,
  pattern: ConstructorPattern,
  valueVar: string
): string {
  if (!pattern.patterns || pattern.patterns.length === 0) {
    return ""
  }

  if (isBuiltinConstructor(pattern.constructorName)) {
    return generateBuiltinConstructorBindings(ctx, pattern, valueVar)
  }
  return generateUserDefinedConstructorBindings(ctx, pattern, valueVar)
}

/**
 * ビルトインコンストラクタバインディング生成
 */
function generateBuiltinConstructorBindings(
  ctx: CodeGenContext,
  pattern: ConstructorPattern,
  valueVar: string
): string {
  if (isSingleValueConstructor(pattern.constructorName)) {
    return generateSingleValueConstructorBindings(ctx, pattern, valueVar)
  }
  if (pattern.constructorName === "Cons") {
    return generateConsConstructorBindings(ctx, pattern, valueVar)
  }
  return ""
}

/**
 * 単一値コンストラクタ判定
 */
function isSingleValueConstructor(constructorName: string): boolean {
  return (
    constructorName === "Just" ||
    constructorName === "Left" ||
    constructorName === "Right"
  )
}

/**
 * 単一値コンストラクタのバインディング
 */
function generateSingleValueConstructorBindings(
  ctx: CodeGenContext,
  pattern: ConstructorPattern,
  valueVar: string
): string {
  if (pattern.patterns.length > 0) {
    const subPattern = pattern.patterns[0]
    // 再帰的にサブパターンのバインディングを生成
    return generatePatternBindings(ctx, subPattern, `${valueVar}.value`)
  }
  return ""
}

/**
 * Consコンストラクタのバインディング
 */
function generateConsConstructorBindings(
  ctx: CodeGenContext,
  pattern: ConstructorPattern,
  valueVar: string
): string {
  let bindings = ""

  if (pattern.patterns.length > 0) {
    const headPattern = pattern.patterns[0]
    bindings += generatePatternBindings(ctx, headPattern, `${valueVar}.head`)
  }

  if (pattern.patterns.length > 1) {
    const tailPattern = pattern.patterns[1]
    bindings += generatePatternBindings(ctx, tailPattern, `${valueVar}.tail`)
  }

  return bindings
}

/**
 * ユーザー定義コンストラクタバインディング生成
 */
function generateUserDefinedConstructorBindings(
  ctx: CodeGenContext,
  pattern: ConstructorPattern,
  valueVar: string
): string {
  let bindings = ""

  for (let i = 0; i < pattern.patterns.length; i++) {
    const subPattern = pattern.patterns[i]
    bindings += generatePatternBindings(
      ctx,
      subPattern,
      `${valueVar}.data[${i}]`
    )
  }

  return bindings
}

/**
 * タプルパターンのバインディング生成
 */
function generateTuplePatternBindings(
  ctx: CodeGenContext,
  pattern: TuplePattern,
  valueVar: string
): string {
  let tupleBindings = ""
  for (let i = 0; i < pattern.patterns.length; i++) {
    const subPattern = pattern.patterns[i]
    tupleBindings += generatePatternBindings(
      ctx,
      subPattern,
      `${valueVar}.elements[${i}]`
    )
  }
  return tupleBindings
}

/**
 * Orパターンのバインディング生成
 */
function generateOrPatternBindings(
  ctx: CodeGenContext,
  pattern: OrPattern,
  valueVar: string
): string {
  if (pattern.patterns.length > 0 && pattern.patterns[0]) {
    // 最初のパターンからバインディングを生成
    // 実際のマッチングは条件で制御される
    return generatePatternBindings(ctx, pattern.patterns[0], valueVar)
  }
  return ""
}

/**
 * ガードパターンのバインディング生成
 */
function generateGuardPatternBindings(
  ctx: CodeGenContext,
  pattern: GuardPattern,
  valueVar: string
): string {
  return generatePatternBindings(ctx, pattern.pattern, valueVar)
}

/**
 * リスト糖衣構文パターンのバインディング生成
 */
function generateListSugarPatternBindings(
  ctx: CodeGenContext,
  pattern: ListSugarPattern,
  valueVar: string
): string {
  let bindings = ""
  let currentVar = valueVar

  // 各要素パターンのバインディング
  for (let i = 0; i < pattern.patterns.length; i++) {
    const elemPattern = pattern.patterns[i]
    const headVar = `${currentVar}.head`
    if (elemPattern) {
      bindings += generatePatternBindings(ctx, elemPattern, headVar)
    }
    currentVar = `${currentVar}.tail`
  }

  // restパターンのバインディング
  if (pattern.restPattern) {
    bindings += generatePatternBindings(ctx, pattern.restPattern, currentVar)
  }

  return bindings
}

/**
 * 配列パターンのバインディング生成
 */
function generateArrayPatternBindings(
  ctx: CodeGenContext,
  pattern: ArrayPattern,
  valueVar: string
): string {
  let bindings = ""

  // 各要素パターンのバインディング
  for (let i = 0; i < pattern.patterns.length; i++) {
    const elemPattern = pattern.patterns[i]
    const indexVar = `${valueVar}[${i}]`
    if (elemPattern) {
      bindings += generatePatternBindings(ctx, elemPattern, indexVar)
    }
  }

  // restパターンのバインディング
  if (pattern.restPattern) {
    const sliceVar = `${valueVar}.slice(${pattern.patterns.length})`
    bindings += generatePatternBindings(ctx, pattern.restPattern, sliceVar)
  }

  return bindings
}
