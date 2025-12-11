/**
 * テンプレート式の生成
 */

import type { TemplateExpression } from "../../../ast"
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
    if (part.kind === "Literal" && (part as any).literalType === "string") {
      // 文字列リテラルはそのまま
      return (part as any).value
    }
    // 式は ${} で囲む
    return `\${${generateExpression(ctx, part)}}`
  })

  return `\`${parts.join("")}\``
}
