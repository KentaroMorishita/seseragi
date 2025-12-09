/**
 * RecordDestructuring の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import {
  addConstraint,
  freshTypeVariable,
  setNodeType,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * レコード分割代入の制約を生成
 */
export function generateConstraintsForRecordDestructuring(
  ctx: InferenceContext,
  recordDestr: AST.RecordDestructuring,
  env: Map<string, AST.Type>
): void {
  // 初期化式の型を推論
  const initType = generateConstraintsForExpression(
    ctx,
    recordDestr.initializer,
    env
  )

  // パターン内の各フィールドを環境に追加し、適切な型制約を設定
  for (const field of recordDestr.pattern.fields) {
    const variableName = field.alias || field.fieldName
    const fieldType = freshTypeVariable(ctx, field.line, field.column)

    // フィールド変数を環境に追加
    env.set(variableName, fieldType)
    setNodeType(ctx, field, fieldType)

    // 初期化式がレコード型で、該当フィールドを持つことを制約として追加
    const recordFieldType = freshTypeVariable(ctx, field.line, field.column)
    const expectedRecordType = new AST.RecordType(
      [
        new AST.RecordField(
          field.fieldName,
          recordFieldType,
          field.line,
          field.column
        ),
      ],
      recordDestr.line,
      recordDestr.column
    )

    // 初期化式のレコード型に該当フィールドが存在することを制約として追加
    addConstraint(
      ctx,
      new TypeConstraint(
        initType,
        expectedRecordType,
        field.line,
        field.column,
        `Record destructuring field ${field.fieldName}`
      )
    )

    // フィールド変数の型とレコードフィールドの型が一致することを制約として追加
    addConstraint(
      ctx,
      new TypeConstraint(
        fieldType,
        recordFieldType,
        field.line,
        field.column,
        `Record destructuring field type ${field.fieldName}`
      )
    )
  }
}
