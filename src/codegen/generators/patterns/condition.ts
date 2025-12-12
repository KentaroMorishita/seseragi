/**
 * パターン条件生成
 *
 * パターンマッチングの条件式を生成する
 */

import type {
  ArrayPattern,
  ConstructorPattern,
  GuardPattern,
  IdentifierPattern,
  ListSugarPattern,
  LiteralPattern,
  OrPattern,
  Pattern,
  TuplePattern,
} from "../../../ast"
import type { CodeGenContext } from "../../context"
import { isBuiltinConstructor } from "../../helpers"
import { generateExpression } from "../dispatcher"

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
    case "LiteralPattern":
      return generateLiteralPatternCondition(
        pattern as LiteralPattern,
        valueVar
      )

    case "IdentifierPattern":
      return generateIdentifierPatternCondition(
        pattern as IdentifierPattern,
        valueVar
      )

    case "ConstructorPattern":
      return generateConstructorPatternCondition(
        ctx,
        pattern as ConstructorPattern,
        valueVar
      )

    case "TuplePattern":
      return generateTuplePatternCondition(
        ctx,
        pattern as TuplePattern,
        valueVar
      )

    case "WildcardPattern":
      return "true"

    case "OrPattern":
      return generateOrPatternCondition(ctx, pattern as OrPattern, valueVar)

    case "GuardPattern":
      return generateGuardPatternCondition(
        ctx,
        pattern as GuardPattern,
        valueVar
      )

    case "ListSugarPattern":
      return generateListSugarPatternCondition(
        pattern as ListSugarPattern,
        valueVar
      )

    case "ArrayPattern":
      return generateArrayPatternCondition(pattern as ArrayPattern, valueVar)

    default:
      // 後方互換性のための古い形式をチェック
      if ((pattern as any).constructor) {
        return `${valueVar}.type === '${(pattern as any).constructor}'`
      }
      return `${valueVar} === ${JSON.stringify(pattern.toString())}`
  }
}

/**
 * リテラルパターンの条件生成
 */
function generateLiteralPatternCondition(
  pattern: LiteralPattern,
  valueVar: string
): string {
  if (typeof pattern.value === "string") {
    return `${valueVar} === ${JSON.stringify(pattern.value)}`
  }
  return `${valueVar} === ${pattern.value}`
}

/**
 * 識別子パターンの条件生成
 */
function generateIdentifierPatternCondition(
  pattern: IdentifierPattern,
  _valueVar: string
): string {
  if (pattern.name === "_") {
    return "true" // ワイルドカードパターン
  }
  // 変数にバインドする場合（常に true）
  return "true"
}

/**
 * コンストラクタパターンの条件生成
 */
function generateConstructorPatternCondition(
  ctx: CodeGenContext,
  pattern: ConstructorPattern,
  valueVar: string
): string {
  const constructorCondition = generateConstructorCondition(pattern, valueVar)
  const subConditions = generateSubPatternConditions(ctx, pattern, valueVar)

  if (subConditions.length > 0) {
    return `${constructorCondition} && ${subConditions.join(" && ")}`
  }

  return constructorCondition
}

/**
 * コンストラクタ条件の生成
 */
function generateConstructorCondition(
  pattern: ConstructorPattern,
  valueVar: string
): string {
  if (isBuiltinConstructor(pattern.constructorName)) {
    return `${valueVar}.tag === '${pattern.constructorName}'`
  }
  return `${valueVar}.type === '${pattern.constructorName}'`
}

/**
 * サブパターン条件の生成
 */
function generateSubPatternConditions(
  ctx: CodeGenContext,
  pattern: ConstructorPattern,
  valueVar: string
): string[] {
  if (!pattern.patterns || pattern.patterns.length === 0) {
    return []
  }

  const subConditions: string[] = []

  for (let i = 0; i < pattern.patterns.length; i++) {
    const subPattern = pattern.patterns[i]
    if (subPattern.kind === "LiteralPattern") {
      const valueAccess = generateValueAccess(pattern, valueVar, i)
      const condition = generateLiteralPatternCondition(
        subPattern as LiteralPattern,
        valueAccess
      )
      subConditions.push(condition)
    } else if (
      subPattern.kind !== "IdentifierPattern" &&
      subPattern.kind !== "WildcardPattern"
    ) {
      // 再帰的にサブパターンを処理
      const valueAccess = generateValueAccess(pattern, valueVar, i)
      const condition = generatePatternCondition(ctx, subPattern, valueAccess)
      if (condition !== "true") {
        subConditions.push(condition)
      }
    }
    // IdentifierPatternの場合は常にtrue（バインディングのみ）
  }

  return subConditions
}

/**
 * 値アクセス文字列の生成
 */
function generateValueAccess(
  pattern: ConstructorPattern,
  valueVar: string,
  index: number
): string {
  if (isBuiltinConstructor(pattern.constructorName)) {
    return generateBuiltinValueAccess(pattern.constructorName, valueVar, index)
  }
  return `${valueVar}.data[${index}]`
}

/**
 * ビルトイン型の値アクセス
 */
function generateBuiltinValueAccess(
  constructorName: string,
  valueVar: string,
  index: number
): string {
  if (
    constructorName === "Just" ||
    constructorName === "Left" ||
    constructorName === "Right"
  ) {
    return `${valueVar}.value`
  }
  if (constructorName === "Cons") {
    return index === 0 ? `${valueVar}.head` : `${valueVar}.tail`
  }
  return `${valueVar}.data[${index}]`
}

/**
 * タプルパターンの条件生成
 */
function generateTuplePatternCondition(
  ctx: CodeGenContext,
  pattern: TuplePattern,
  valueVar: string
): string {
  const tupleConditions = pattern.patterns.map((subPattern, i) => {
    return generatePatternCondition(
      ctx,
      subPattern,
      `${valueVar}.elements[${i}]`
    )
  })
  const nonTrivialConditions = tupleConditions.filter((c) => c !== "true")
  if (nonTrivialConditions.length === 0) {
    return "true"
  }
  return nonTrivialConditions.join(" && ")
}

/**
 * Orパターンの条件生成
 */
function generateOrPatternCondition(
  ctx: CodeGenContext,
  pattern: OrPattern,
  valueVar: string
): string {
  const orConditions = pattern.patterns.map((subPattern: Pattern) => {
    return generatePatternCondition(ctx, subPattern, valueVar)
  })
  return `(${orConditions.join(" || ")})`
}

/**
 * ガードパターンの条件生成
 */
function generateGuardPatternCondition(
  ctx: CodeGenContext,
  pattern: GuardPattern,
  valueVar: string
): string {
  const patternCondition = generatePatternCondition(
    ctx,
    pattern.pattern,
    valueVar
  )
  const guardCondition = generateExpression(ctx, pattern.guard)
  return `(${patternCondition} && (${guardCondition}))`
}

/**
 * リスト糖衣構文パターンの条件生成
 */
function generateListSugarPatternCondition(
  pattern: ListSugarPattern,
  valueVar: string
): string {
  // 空リストパターン []
  if (pattern.patterns.length === 0 && !pattern.restPattern) {
    return `${valueVar}.tag === 'Empty'`
  }

  // restのみのパターン [...rest]
  if (pattern.patterns.length === 0 && pattern.restPattern) {
    return "true" // すべてのリストにマッチ
  }

  // パターンを構築
  const conditions: string[] = []
  let currentVar = valueVar

  // 各要素パターンをチェック
  for (let i = 0; i < pattern.patterns.length; i++) {
    conditions.push(`${currentVar}.tag === 'Cons'`)
    currentVar = `${currentVar}.tail`
  }

  // restパターンがない場合、残りはEmptyである必要がある
  if (!pattern.restPattern) {
    conditions.push(`${currentVar}.tag === 'Empty'`)
  }

  return `(${conditions.join(" && ")})`
}

/**
 * 配列パターンの条件生成
 */
function generateArrayPatternCondition(
  pattern: ArrayPattern,
  valueVar: string
): string {
  // 空配列パターン []
  if (pattern.patterns.length === 0 && !pattern.restPattern) {
    return `Array.isArray(${valueVar}) && ${valueVar}.length === 0`
  }

  // restのみのパターン [...rest]
  if (pattern.patterns.length === 0 && pattern.restPattern) {
    return `Array.isArray(${valueVar})` // 配列かどうかのみチェック
  }

  const conditions: string[] = []

  // 配列型チェックを最初に追加
  conditions.push(`Array.isArray(${valueVar})`)

  // 要素数チェック
  if (!pattern.restPattern) {
    // restがない場合、正確な長さを要求
    conditions.push(`${valueVar}.length === ${pattern.patterns.length}`)
  } else {
    // restがある場合、最小長さを要求
    conditions.push(`${valueVar}.length >= ${pattern.patterns.length}`)
  }

  return `(${conditions.join(" && ")})`
}
