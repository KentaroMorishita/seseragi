/**
 * StructDestructuring の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import {
  addConstraint,
  addError,
  freshTypeVariable,
  setNodeType,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * 構造体分割代入の制約を生成
 */
export function generateConstraintsForStructDestructuring(
  ctx: InferenceContext,
  structDestr: AST.StructDestructuring,
  env: Map<string, AST.Type>
): void {
  // 初期化式の型を推論
  const initType = generateConstraintsForExpression(
    ctx,
    structDestr.initializer,
    env
  )

  // 初期化式が指定された構造体型であることを制約として追加
  const expectedStructType = env.get(structDestr.pattern.structName)
  if (expectedStructType && expectedStructType.kind === "StructType") {
    addConstraint(
      ctx,
      new TypeConstraint(
        initType,
        expectedStructType,
        structDestr.line,
        structDestr.column,
        `Struct destructuring of ${structDestr.pattern.structName}`
      )
    )

    // パターン内の各フィールドを環境に追加し、適切な型制約を設定
    const structType = expectedStructType as AST.StructType
    for (const field of structDestr.pattern.fields) {
      const variableName = field.alias || field.fieldName

      // 構造体定義から該当フィールドの型を取得
      const structField = structType.fields.find(
        (f) => f.name === field.fieldName
      )
      if (structField) {
        // フィールドの実際の型を使用
        env.set(variableName, structField.type)
        setNodeType(ctx, field, structField.type)

        // 分割代入された変数を追跡するために、仮想的な変数宣言ノードを作成してnodeTypeMapに追加
        const virtualVarDecl = {
          kind: "VariableDeclaration",
          name: variableName,
          type: structField.type,
          line: field.line,
          column: field.column,
          isDestructured: true,
        } as unknown as AST.ASTNode
        setNodeType(ctx, virtualVarDecl, structField.type)
      } else {
        // フィールドが見つからない場合はエラー
        addError(
          ctx,
          `Field '${field.fieldName}' does not exist in struct ${structDestr.pattern.structName}`,
          field.line,
          field.column
        )
        const fieldType = freshTypeVariable(ctx, field.line, field.column)
        env.set(variableName, fieldType)
      }
    }
  } else {
    // 構造体型が見つからない場合はfreshTypeVariableで処理
    for (const field of structDestr.pattern.fields) {
      const variableName = field.alias || field.fieldName
      const fieldType = freshTypeVariable(ctx, field.line, field.column)
      env.set(variableName, fieldType)
    }
  }
}
