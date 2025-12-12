/**
 * 識別子式の生成
 */

import type { Identifier } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { sanitizeIdentifier } from "../../helpers"

/**
 * 識別子をTypeScriptコードに変換
 */
export function generateIdentifier(
  _ctx: CodeGenContext,
  identifier: Identifier
): string {
  return sanitizeIdentifier(identifier.name)
}
