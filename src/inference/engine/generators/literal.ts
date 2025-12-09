/**
 * リテラル式の制約生成
 */

import * as AST from "../../../ast"
import { freshTypeVariable, type InferenceContext } from "../context"

/**
 * リテラルの型を返す
 * リテラルは制約を生成せず、直接型を返す
 */
export function generateConstraintsForLiteral(
  ctx: InferenceContext,
  literal: AST.Literal
): AST.Type {
  switch (literal.literalType) {
    case "string":
      return new AST.PrimitiveType("String", literal.line, literal.column)
    case "integer":
      return new AST.PrimitiveType("Int", literal.line, literal.column)
    case "float":
      return new AST.PrimitiveType("Float", literal.line, literal.column)
    case "boolean":
      return new AST.PrimitiveType("Bool", literal.line, literal.column)
    case "unit":
      return new AST.PrimitiveType("Unit", literal.line, literal.column)
    default:
      return freshTypeVariable(ctx, literal.line, literal.column)
  }
}
