/**
 * テンプレート式の制約生成
 */

import * as AST from "../../../ast"
import type { InferenceContext } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * TemplateExpression: `Hello ${name}`
 */
export function generateConstraintsForTemplateExpression(
  ctx: InferenceContext,
  templateExpr: AST.TemplateExpression,
  env: Map<string, AST.Type>
): AST.Type {
  // テンプレートリテラルの結果型は常にString
  const resultType = new AST.PrimitiveType(
    "String",
    templateExpr.line,
    templateExpr.column
  )

  // 各埋め込み式の型を推論
  for (const part of templateExpr.parts) {
    if (typeof part !== "string") {
      // 埋め込み式の型を推論
      generateConstraintsForExpression(ctx, part, env)
    }
  }

  return resultType
}
