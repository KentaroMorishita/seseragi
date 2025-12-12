/**
 * ListComprehension の制約生成
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
 * 配列内包表記 [expr | gen <- iter, ...] の制約を生成
 */
export function generateConstraintsForListComprehension(
  ctx: InferenceContext,
  comp: AST.ListComprehension,
  env: Map<string, AST.Type>
): AST.Type {
  // 内包表記用の環境を作成
  const compEnv = new Map(env)

  // ジェネレータを処理してスコープに変数を追加
  for (const generator of comp.generators) {
    // ジェネレータのiterableの型を推論
    const iterableType = generateConstraintsForExpression(
      ctx,
      generator.iterable,
      compEnv
    )

    // iterableはリスト型またはArray型でなければならない
    const elementType = freshTypeVariable(ctx, generator.line, generator.column)
    const arrayType = new AST.GenericType(
      "Array",
      [elementType],
      generator.line,
      generator.column
    )

    // 配列内包表記では範囲リテラル（Array型）を直接受け入れる
    addConstraint(
      ctx,
      new TypeConstraint(
        iterableType,
        arrayType,
        generator.line,
        generator.column,
        "Generator iterable must be Array type for array comprehensions"
      )
    )

    // ジェネレータ変数をスコープに追加
    compEnv.set(generator.variable, elementType)
  }

  // フィルタ条件の型チェック
  for (const filter of comp.filters) {
    const filterType = generateConstraintsForExpression(ctx, filter, compEnv)
    const boolType = new AST.PrimitiveType("Bool", filter.line, filter.column)

    addConstraint(
      ctx,
      new TypeConstraint(
        filterType,
        boolType,
        filter.line,
        filter.column,
        "Array comprehension filter must be Bool"
      )
    )
  }

  // 内包表記の式の型を推論
  const expressionType = generateConstraintsForExpression(
    ctx,
    comp.expression,
    compEnv
  )

  // 結果はArray<expressionType>（配列内包表記なのでArrayを返す）
  return new AST.GenericType("Array", [expressionType], comp.line, comp.column)
}

/**
 * ListComprehensionSugar `[expr | gen <- iter, ...]` の制約を生成
 */
export function generateConstraintsForListComprehensionSugar(
  ctx: InferenceContext,
  comp: AST.ListComprehensionSugar,
  env: Map<string, AST.Type>
): AST.Type {
  // 内包表記用の環境を作成
  const compEnv = new Map(env)

  // ジェネレータを処理してスコープに変数を追加
  for (const generator of comp.generators) {
    // ジェネレータのiterableの型を推論
    const iterableType = generateConstraintsForExpression(
      ctx,
      generator.iterable,
      compEnv
    )

    // iterableはList型またはArray型を受け入れる
    const elementType = freshTypeVariable(ctx, generator.line, generator.column)
    const expectedListType = new AST.GenericType(
      "List",
      [elementType],
      generator.line,
      generator.column
    )

    // iterableがListまたはArrayであることを制約として追加
    addConstraint(
      ctx,
      new TypeConstraint(
        iterableType,
        expectedListType,
        generator.line,
        generator.column,
        "Generator iterable must be List type (Array conversion allowed)"
      )
    )

    // ジェネレータ変数をスコープに追加
    compEnv.set(generator.variable, elementType)
  }

  // フィルタ条件の型チェック
  for (const filter of comp.filters) {
    const filterType = generateConstraintsForExpression(ctx, filter, compEnv)
    const boolType = new AST.PrimitiveType("Bool", filter.line, filter.column)

    addConstraint(
      ctx,
      new TypeConstraint(
        filterType,
        boolType,
        filter.line,
        filter.column,
        "List comprehension filter must be Bool"
      )
    )
  }

  // 内包表記の式の型を推論
  const expressionType = generateConstraintsForExpression(
    ctx,
    comp.expression,
    compEnv
  )

  // 結果はList<expressionType>
  return new AST.GenericType("List", [expressionType], comp.line, comp.column)
}
