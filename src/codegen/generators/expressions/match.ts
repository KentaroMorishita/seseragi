/**
 * MatchExpression コード生成
 *
 * パターンマッチング式をTypeScriptのif-elseチェーンに変換する
 */

import type {
  Expression,
  GuardPattern,
  MatchExpression,
  Pattern,
} from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"
import { generatePatternBindings, generatePatternCondition } from "../patterns"

/**
 * MatchExpression を生成
 *
 * match式をIIFE（即時実行関数式）でラップしたif-elseチェーンに変換する
 *
 * 生成例:
 * ```typescript
 * (() => {
 *   const matchValue = <expression>;
 *   if (<pattern1_condition>) {
 *     <pattern1_bindings>
 *     return <expression1>;
 *   }
 *   if (<pattern2_condition>) {
 *     <pattern2_bindings>
 *     return <expression2>;
 *   }
 *   else {
 *     throw new Error('Non-exhaustive pattern match');
 *   }
 * })()
 * ```
 *
 * @param ctx - コード生成コンテキスト
 * @param expr - MatchExpression
 * @returns 生成されたTypeScriptコード
 */
export function generateMatchExpression(
  ctx: CodeGenContext,
  expr: MatchExpression
): string {
  // マッチング対象の式を生成
  const matchValueExpr = generateExpression(ctx, expr.expression)

  // if-elseチェーンを構築
  let result = `(() => {\n  const matchValue = ${matchValueExpr};\n`

  // 各ケースを処理
  for (let i = 0; i < expr.cases.length; i++) {
    const matchCase = expr.cases[i]
    if (!matchCase) continue

    const pattern = matchCase.pattern
    const caseExpression = matchCase.expression

    // GuardPatternの場合は特別な処理が必要
    if (pattern.kind === "GuardPattern") {
      result += generateGuardPatternCase(
        ctx,
        pattern as GuardPattern,
        caseExpression
      )
    } else {
      result += generateNormalPatternCase(ctx, pattern, caseExpression)
    }
  }

  // 網羅性チェック失敗時のエラー
  result +=
    " else {\n    throw new Error('Non-exhaustive pattern match');\n  }\n})()"

  return result
}

/**
 * ガードパターンのケースを生成
 *
 * ガードパターンでは、まず基本パターンをチェックし、
 * その後でガード条件をチェックする二段階の検証を行う
 *
 * @param ctx - コード生成コンテキスト
 * @param pattern - ガードパターン
 * @param caseExpression - ケースの結果式
 * @returns 生成されたケースコード
 */
function generateGuardPatternCase(
  ctx: CodeGenContext,
  pattern: GuardPattern,
  caseExpression: Expression
): string {
  // 基本パターンの条件を生成
  const baseCondition = generatePatternCondition(
    ctx,
    pattern.pattern,
    "matchValue"
  )

  // 基本パターンのバインディングを生成
  const bindings = generatePatternBindings(ctx, pattern.pattern, "matchValue")

  // ガード条件を生成
  const guardCondition = generateExpression(ctx, pattern.guard)

  // ケースの結果式を生成
  const body = generateExpression(ctx, caseExpression)

  // GuardPatternでは常にifを使用（else ifではなく）
  // これにより、ガード条件が失敗したときに次のパターンへ続行できる
  return `  if (${baseCondition}) {\n    ${bindings}if (${guardCondition}) {\n      return ${body};\n    }\n  }`
}

/**
 * 通常パターンのケースを生成
 *
 * @param ctx - コード生成コンテキスト
 * @param pattern - パターン
 * @param caseExpression - ケースの結果式
 * @returns 生成されたケースコード
 */
function generateNormalPatternCase(
  ctx: CodeGenContext,
  pattern: Pattern,
  caseExpression: Expression
): string {
  // パターンの条件を生成
  const condition = generatePatternCondition(ctx, pattern, "matchValue")

  // パターンのバインディングを生成
  const bindings = generatePatternBindings(ctx, pattern, "matchValue")

  // ケースの結果式を生成
  const body = generateExpression(ctx, caseExpression)

  // 通常のパターンは常に'if'を使用（ガードパターンと同様の連続的チェック）
  return `  if (${condition}) {\n    ${bindings}return ${body};\n  }`
}
