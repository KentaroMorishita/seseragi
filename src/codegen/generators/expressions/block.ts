/**
 * ブロック式の生成
 */

import type { BlockExpression } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"
import { generateStatement } from "../statements"

/**
 * ブロック式をTypeScriptコードに変換
 * IIFE (即時実行関数式) として生成
 */
export function generateBlockExpression(
  ctx: CodeGenContext,
  expr: BlockExpression
): string {
  const lines: string[] = []

  // 各文を生成
  for (const stmt of expr.statements) {
    const code = generateStatement(ctx, stmt)
    if (code.trim()) {
      lines.push(code)
    }
  }

  // 返り値の式があれば追加
  if (expr.returnExpression) {
    lines.push(`return ${generateExpression(ctx, expr.returnExpression)};`)
  }

  // IIFEとして生成
  return `(() => {\n${lines.map((line) => `  ${line}`).join("\n")}\n})()`
}
