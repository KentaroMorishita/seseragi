/**
 * 識別子の制約生成
 */

import type * as AST from "../../../ast"
import { addError, freshTypeVariable, type InferenceContext } from "../context"
import { instantiatePolymorphicType } from "./helpers"

/**
 * 識別子の型を環境から検索し、多相型の場合はインスタンス化する
 */
export function generateConstraintsForIdentifier(
  ctx: InferenceContext,
  identifier: AST.Identifier,
  env: Map<string, AST.Type>
): AST.Type {
  const type = env.get(identifier.name)

  if (!type) {
    addError(
      ctx,
      `Undefined variable: ${identifier.name}`,
      identifier.line,
      identifier.column
    )
    return freshTypeVariable(ctx, identifier.line, identifier.column)
  }

  // 多相型の場合はインスタンス化して返す
  return instantiatePolymorphicType(
    ctx,
    type,
    identifier.line,
    identifier.column
  )
}
