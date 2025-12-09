/**
 * レコードアクセスの制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import {
  addConstraint,
  addError,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * レコードアクセスの制約を生成
 * record.field のような式を処理
 */
export function generateConstraintsForRecordAccess(
  ctx: InferenceContext,
  access: AST.RecordAccess,
  env: Map<string, AST.Type>
): AST.Type {
  const recordType = generateConstraintsForExpression(ctx, access.record, env)

  // 配列の.lengthアクセスを特別に処理
  if (access.fieldName === "length") {
    const elementType = freshTypeVariable(ctx, access.line, access.column)
    const arrayType = new AST.GenericType(
      "Array",
      [elementType],
      access.line,
      access.column
    )
    addConstraint(
      ctx,
      new TypeConstraint(
        recordType,
        arrayType,
        access.line,
        access.column,
        "Array length access"
      )
    )
    // lengthはInt型を返す
    return new AST.PrimitiveType("Int", access.line, access.column)
  }

  // StructTypeを直接チェック
  if (recordType.kind === "StructType") {
    const structType = recordType as AST.StructType
    const field = structType.fields.find((f) => f.name === access.fieldName)
    if (field) {
      return field.type
    }
    addError(
      ctx,
      `Field '${access.fieldName}' does not exist on struct '${structType.name}'`,
      access.line,
      access.column,
      `Field access .${access.fieldName}`
    )
    return freshTypeVariable(ctx, access.line, access.column)
  }

  // 型変数やその他の場合は制約ベースのアプローチ
  const fieldType = freshTypeVariable(ctx, access.line, access.column)

  const expectedRecordType = new AST.RecordType(
    [
      new AST.RecordField(
        access.fieldName,
        fieldType,
        access.line,
        access.column
      ),
    ],
    access.line,
    access.column
  )

  addConstraint(
    ctx,
    new TypeConstraint(
      recordType,
      expectedRecordType,
      access.line,
      access.column,
      `Field access .${access.fieldName}`
    )
  )

  return fieldType
}
