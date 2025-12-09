/**
 * 配列リテラルの制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import { addConstraint, freshTypeVariable, type InferenceContext } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * 配列リテラルの制約を生成
 * [a, b, c] のような式を処理
 */
export function generateConstraintsForArrayLiteral(
  ctx: InferenceContext,
  arrayLiteral: AST.ArrayLiteral,
  env: Map<string, AST.Type>
): AST.Type {
  if (arrayLiteral.elements.length === 0) {
    // 空配列の場合、要素型は新しい型変数
    const elementType = freshTypeVariable(
      ctx,
      arrayLiteral.line,
      arrayLiteral.column
    )
    return new AST.GenericType(
      "Array",
      [elementType],
      arrayLiteral.line,
      arrayLiteral.column
    )
  }

  // 最初の要素の型を推論
  const firstElement = arrayLiteral.elements[0]
  if (!firstElement) {
    // 安全のためのフォールバック
    const elementType = freshTypeVariable(
      ctx,
      arrayLiteral.line,
      arrayLiteral.column
    )
    return new AST.GenericType(
      "Array",
      [elementType],
      arrayLiteral.line,
      arrayLiteral.column
    )
  }

  const firstElementType = generateConstraintsForExpression(
    ctx,
    firstElement,
    env
  )

  // すべての要素が同じ型であることを制約として追加
  for (let i = 1; i < arrayLiteral.elements.length; i++) {
    const element = arrayLiteral.elements[i]
    if (element) {
      const elementType = generateConstraintsForExpression(ctx, element, env)
      addConstraint(
        ctx,
        new TypeConstraint(
          firstElementType,
          elementType,
          element.line,
          element.column,
          `Array element type consistency`
        )
      )
    }
  }

  return new AST.GenericType(
    "Array",
    [firstElementType],
    arrayLiteral.line,
    arrayLiteral.column
  )
}
