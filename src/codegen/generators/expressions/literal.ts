/**
 * リテラル式の生成
 */

import type { Literal } from "../../../ast"
import type { CodeGenContext } from "../../context"

/**
 * リテラル式をTypeScriptコードに変換
 */
export function generateLiteral(ctx: CodeGenContext, literal: Literal): string {
  switch (literal.literalType) {
    case "string":
      return `"${literal.value}"`
    case "integer":
    case "float":
      return literal.value.toString()
    case "boolean":
      return literal.value.toString()
    case "unit":
      return "Unit" // Unit値は専用オブジェクトとして表現
    default:
      return literal.value.toString()
  }
}
