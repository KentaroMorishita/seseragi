/**
 * テンプレート式の生成
 */

import type { Expression, TemplateExpression } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * テンプレートリテラル式をTypeScriptコードに変換
 */
export function generateTemplateExpression(
  ctx: CodeGenContext,
  template: TemplateExpression
): string {
  const parts = template.parts.map((part) => {
    if (typeof part === "string") {
      // 文字列リテラルはそのまま
      return part
    }
    if (
      (part as any).kind === "Literal" &&
      (part as any).literalType === "string"
    ) {
      // Literal式の場合
      return (part as any).value
    }
    // 式は ${} で囲む
    return `\${${generateExpression(ctx, part as Expression)}}`
  })

  return `\`${parts.join("")}\``
}
