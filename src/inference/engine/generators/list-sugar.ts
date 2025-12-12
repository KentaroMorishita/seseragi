/**
 * リスト糖衣構文の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import {
  addConstraint,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * リスト糖衣構文の制約を生成
 * [|a, b, c|] のような式を処理
 */
export function generateConstraintsForListSugar(
  ctx: InferenceContext,
  listSugar: AST.ListSugar,
  env: Map<string, AST.Type>
): AST.Type {
  if (listSugar.elements.length === 0) {
    // 空リストの場合、要素型は新しい型変数
    const elementType = freshTypeVariable(ctx, listSugar.line, listSugar.column)
    return new AST.GenericType(
      "List",
      [elementType],
      listSugar.line,
      listSugar.column
    )
  }

  // 最初の要素の型を推論
  const firstElement = listSugar.elements[0]
  if (!firstElement) {
    const elementType = freshTypeVariable(ctx, listSugar.line, listSugar.column)
    return new AST.GenericType(
      "List",
      [elementType],
      listSugar.line,
      listSugar.column
    )
  }

  const firstElementType = generateConstraintsForExpression(
    ctx,
    firstElement,
    env
  )

  // すべての要素が同じ型であることを制約として追加
  for (let i = 1; i < listSugar.elements.length; i++) {
    const element = listSugar.elements[i]
    if (element) {
      const elementType = generateConstraintsForExpression(ctx, element, env)
      addConstraint(
        ctx,
        new TypeConstraint(
          firstElementType,
          elementType,
          element.line,
          element.column,
          `List element type consistency`
        )
      )
    }
  }

  return new AST.GenericType(
    "List",
    [firstElementType],
    listSugar.line,
    listSugar.column
  )
}
