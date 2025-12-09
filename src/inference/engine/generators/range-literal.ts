/**
 * RangeLiteral の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import { addConstraint, type InferenceContext } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * 範囲リテラル [start..end] の制約を生成
 */
export function generateConstraintsForRangeLiteral(
  ctx: InferenceContext,
  range: AST.RangeLiteral,
  env: Map<string, AST.Type>
): AST.Type {
  // 範囲リテラルの開始と終了値の型を推論
  const startType = generateConstraintsForExpression(ctx, range.start, env)
  const endType = generateConstraintsForExpression(ctx, range.end, env)

  // 開始と終了は同じ型でなければならない
  addConstraint(
    ctx,
    new TypeConstraint(
      startType,
      endType,
      range.line,
      range.column,
      "Range start and end must have same type"
    )
  )

  // 範囲は数値型（Int）のリストを返す
  const intType = new AST.PrimitiveType("Int", range.line, range.column)

  addConstraint(
    ctx,
    new TypeConstraint(
      startType,
      intType,
      range.start.line,
      range.start.column,
      "Range values must be integers"
    )
  )

  // 範囲リテラルはArray<Int>を返す
  return new AST.GenericType("Array", [intType], range.line, range.column)
}
