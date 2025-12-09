/**
 * 構造体式の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import { isMaybeType } from "../../type-inspection"
import {
  addConstraint,
  addError,
  freshTypeVariable,
  lookupType,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * 構造体式の制約を生成
 * Point { x: 1, y: 2 } のような式を処理
 */
export function generateConstraintsForStructExpression(
  ctx: InferenceContext,
  structExpr: AST.StructExpression,
  env: Map<string, AST.Type>
): AST.Type {
  // 構造体型を環境から取得
  const structType = lookupType(ctx, structExpr.structName) ?? env.get(structExpr.structName)

  if (!structType) {
    addError(
      ctx,
      `Unknown struct type: ${structExpr.structName}`,
      structExpr.line,
      structExpr.column
    )
    return freshTypeVariable(ctx, structExpr.line, structExpr.column)
  }

  if (structType.kind !== "StructType") {
    addError(
      ctx,
      `${structExpr.structName} is not a struct type`,
      structExpr.line,
      structExpr.column
    )
    return freshTypeVariable(ctx, structExpr.line, structExpr.column)
  }

  const st = structType as AST.StructType

  // フィールドの型チェック
  const providedFieldMap = new Map<
    string,
    { field: AST.RecordInitField | AST.RecordSpreadField | AST.RecordShorthandField; type: AST.Type }
  >()

  // まずスプレッドフィールドを処理
  for (const field of structExpr.fields) {
    if (field.kind === "RecordSpreadField") {
      const spreadField = field as AST.RecordSpreadField
      const spreadType = generateConstraintsForExpression(
        ctx,
        spreadField.spreadExpression.expression,
        env
      )

      // スプレッド元が同じ構造体型であることを確認
      if (spreadType.kind === "StructType") {
        const sourceStruct = spreadType as AST.StructType
        for (const sourceField of sourceStruct.fields) {
          providedFieldMap.set(sourceField.name, {
            field: spreadField,
            type: sourceField.type,
          })
        }
      } else {
        addError(
          ctx,
          `Cannot spread non-struct type in struct literal`,
          spreadField.line,
          spreadField.column
        )
      }
    }
  }

  // 次に明示的なフィールドで上書き
  for (const field of structExpr.fields) {
    if (field.kind === "RecordInitField") {
      const initField = field as AST.RecordInitField
      const fieldType = generateConstraintsForExpression(ctx, initField.value, env)
      providedFieldMap.set(initField.name, {
        field: initField,
        type: fieldType,
      })
    } else if (field.kind === "RecordShorthandField") {
      const shorthandField = field as AST.RecordShorthandField
      const variableType = env.get(shorthandField.name)
      if (!variableType) {
        addError(
          ctx,
          `Undefined variable '${shorthandField.name}' in shorthand property`,
          shorthandField.line,
          shorthandField.column
        )
        const fallbackType = freshTypeVariable(ctx, shorthandField.line, shorthandField.column)
        providedFieldMap.set(shorthandField.name, {
          field: shorthandField,
          type: fallbackType,
        })
      } else {
        providedFieldMap.set(shorthandField.name, {
          field: shorthandField,
          type: variableType,
        })
      }
    }
  }

  // 必要なフィールドがすべて提供されているかチェック
  for (const field of st.fields) {
    const providedData = providedFieldMap.get(field.name)

    if (!providedData) {
      // Maybe型フィールドは省略可能
      if (!isMaybeType(field.type)) {
        addError(
          ctx,
          `Missing field '${field.name}' in struct ${structExpr.structName}`,
          structExpr.line,
          structExpr.column
        )
      }
      continue
    }

    // フィールドの型と値の型が一致することを制約として追加
    addConstraint(
      ctx,
      new TypeConstraint(
        providedData.type,
        field.type,
        providedData.field.line,
        providedData.field.column,
        `Struct field ${field.name}`
      )
    )
  }

  // 余分なフィールドがないかチェック
  for (const [fieldName] of providedFieldMap) {
    if (!st.fields.find((f) => f.name === fieldName)) {
      addError(
        ctx,
        `Unknown field '${fieldName}' in struct ${structExpr.structName}`,
        structExpr.line,
        structExpr.column
      )
    }
  }

  return structType
}
