/**
 * FunctorMap (<$>) 演算子の制約生成
 */

import * as AST from "../../../ast"
import { FunctorMapConstraint, TypeConstraint } from "../../constraints"
import {
  addConstraint,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * FunctorMap演算子の制約を生成
 * f <$> m => (a -> b) -> f a -> f b
 */
export function generateConstraintsForFunctorMap(
  ctx: InferenceContext,
  functorMap: AST.FunctorMap,
  env: Map<string, AST.Type>
): AST.Type {
  const funcType = generateConstraintsForExpression(ctx, functorMap.left, env)
  const containerType = generateConstraintsForExpression(
    ctx,
    functorMap.right,
    env
  )

  const inputType = freshTypeVariable(ctx, functorMap.line, functorMap.column)
  const outputType = freshTypeVariable(ctx, functorMap.line, functorMap.column)

  // 関数は (a -> b) 型
  const expectedFuncType = new AST.FunctionType(
    inputType,
    outputType,
    functorMap.line,
    functorMap.column
  )

  addConstraint(
    ctx,
    new TypeConstraint(
      funcType,
      expectedFuncType,
      functorMap.line,
      functorMap.column,
      "FunctorMap function type"
    )
  )

  // コンテナ型に応じた処理
  if (containerType.kind === "GenericType") {
    const gt = containerType as AST.GenericType

    if (gt.name === "Maybe" && gt.typeArguments.length === 1) {
      addConstraint(
        ctx,
        new TypeConstraint(
          gt.typeArguments[0],
          inputType,
          functorMap.line,
          functorMap.column,
          "FunctorMap Maybe container input type"
        )
      )
      return new AST.GenericType(
        "Maybe",
        [outputType],
        functorMap.line,
        functorMap.column
      )
    }

    if (gt.name === "Either" && gt.typeArguments.length === 2) {
      const errorType = gt.typeArguments[0]
      addConstraint(
        ctx,
        new TypeConstraint(
          gt.typeArguments[1],
          inputType,
          functorMap.line,
          functorMap.column,
          "FunctorMap Either container input type"
        )
      )
      return new AST.GenericType(
        "Either",
        [errorType, outputType],
        functorMap.line,
        functorMap.column
      )
    }

    if (gt.name === "List" && gt.typeArguments.length === 1) {
      addConstraint(
        ctx,
        new TypeConstraint(
          gt.typeArguments[0],
          inputType,
          functorMap.line,
          functorMap.column,
          "FunctorMap List container input type"
        )
      )
      return new AST.GenericType(
        "List",
        [outputType],
        functorMap.line,
        functorMap.column
      )
    }

    if (gt.name === "Task" && gt.typeArguments.length === 1) {
      addConstraint(
        ctx,
        new TypeConstraint(
          gt.typeArguments[0],
          inputType,
          functorMap.line,
          functorMap.column,
          "FunctorMap Task container input type"
        )
      )
      return new AST.GenericType(
        "Task",
        [outputType],
        functorMap.line,
        functorMap.column
      )
    }

    // 汎用ファンクター
    if (gt.typeArguments.length > 0) {
      addConstraint(
        ctx,
        new TypeConstraint(
          gt.typeArguments[gt.typeArguments.length - 1],
          inputType,
          functorMap.line,
          functorMap.column,
          "FunctorMap container input type"
        )
      )

      const newArgs = [...gt.typeArguments]
      newArgs[newArgs.length - 1] = outputType

      return new AST.GenericType(
        gt.name,
        newArgs,
        functorMap.line,
        functorMap.column
      )
    }
  }

  // コンテナが型変数の場合
  if (containerType.kind === "TypeVariable") {
    const resultType = freshTypeVariable(
      ctx,
      functorMap.line,
      functorMap.column
    )

    const functorMapConstraint = new FunctorMapConstraint(
      containerType,
      inputType,
      outputType,
      resultType,
      functorMap.line,
      functorMap.column,
      "FunctorMap with type variable container"
    )

    ctx.constraints.push(functorMapConstraint)
    return resultType
  }

  // フォールバック
  return freshTypeVariable(ctx, functorMap.line, functorMap.column)
}
