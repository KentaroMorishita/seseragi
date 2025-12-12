/**
 * レコード関連式の生成
 */

import type {
  RecordAccess,
  RecordExpression,
  RecordInitField,
  RecordShorthandField,
  RecordSpreadField,
} from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * レコード式をTypeScriptコードに変換
 */
export function generateRecordExpression(
  ctx: CodeGenContext,
  record: RecordExpression
): string {
  if (record.fields.length === 0) {
    return "{}"
  }

  const fieldStrings = record.fields
    .map((field) => {
      if (field.kind === "RecordInitField") {
        const initField = field as RecordInitField
        if (!initField.value) return ""
        const value = generateExpression(ctx, initField.value)
        return `${initField.name}: ${value}`
      }
      if (field.kind === "RecordShorthandField") {
        const shorthandField = field as RecordShorthandField
        return shorthandField.name
      }
      if (field.kind === "RecordSpreadField") {
        const spreadField = field as RecordSpreadField
        if (!spreadField.spreadExpression?.expression) return ""
        const spreadValue = generateExpression(
          ctx,
          spreadField.spreadExpression.expression
        )
        return `...${spreadValue}`
      }
      return ""
    })
    .filter((f) => f !== "")

  return `{ ${fieldStrings.join(", ")} }`
}

/**
 * レコードアクセスをTypeScriptコードに変換
 */
export function generateRecordAccess(
  ctx: CodeGenContext,
  access: RecordAccess
): string {
  const record = generateExpression(ctx, access.record)

  if (access.fieldName === "length") {
    return `${record}.length`
  }

  return `${record}.${access.fieldName}`
}
