/**
 * ApplicativeApply (<*>) 演算子の制約生成
 */

import * as AST from "../../../ast"
import { ApplicativeApplyConstraint, TypeConstraint } from "../../constraints"
import {
  addConstraint,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * ApplicativeApply演算子の制約を生成
 * f (a -> b) <*> f a => f b
 */
export function generateConstraintsForApplicativeApply(
  ctx: InferenceContext,
  applicativeApply: AST.ApplicativeApply,
  env: Map<string, AST.Type>
): AST.Type {
  const funcContainerType = generateConstraintsForExpression(
    ctx,
    applicativeApply.left,
    env
  )
  const valueContainerType = generateConstraintsForExpression(
    ctx,
    applicativeApply.right,
    env
  )

  const inputType = freshTypeVariable(
    ctx,
    applicativeApply.line,
    applicativeApply.column
  )
  const outputType = freshTypeVariable(
    ctx,
    applicativeApply.line,
    applicativeApply.column
  )

  const funcType = new AST.FunctionType(
    inputType,
    outputType,
    applicativeApply.line,
    applicativeApply.column
  )

  if (funcContainerType.kind === "GenericType") {
    const funcGt = funcContainerType as AST.GenericType

    if (valueContainerType.kind === "GenericType") {
      const valueGt = valueContainerType as AST.GenericType
      if (funcGt.name !== valueGt.name) {
        addConstraint(
          ctx,
          new TypeConstraint(
            funcContainerType,
            valueContainerType,
            applicativeApply.line,
            applicativeApply.column,
            "ApplicativeApply container type mismatch"
          )
        )
      }
    } else {
      const expectedValueType = new AST.GenericType(
        funcGt.name,
        [inputType],
        applicativeApply.line,
        applicativeApply.column
      )
      addConstraint(
        ctx,
        new TypeConstraint(
          valueContainerType,
          expectedValueType,
          applicativeApply.line,
          applicativeApply.column,
          "ApplicativeApply value container type constraint"
        )
      )
    }

    // 特定の型を処理
    if (funcGt.name === "Maybe" && funcGt.typeArguments.length === 1) {
      addConstraint(
        ctx,
        new TypeConstraint(
          funcGt.typeArguments[0],
          funcType,
          applicativeApply.line,
          applicativeApply.column,
          "ApplicativeApply Maybe function container type"
        )
      )
      return new AST.GenericType(
        "Maybe",
        [outputType],
        applicativeApply.line,
        applicativeApply.column
      )
    }

    if (funcGt.name === "Either" && funcGt.typeArguments.length === 2) {
      const errorType = funcGt.typeArguments[0]
      addConstraint(
        ctx,
        new TypeConstraint(
          funcGt.typeArguments[1],
          funcType,
          applicativeApply.line,
          applicativeApply.column,
          "ApplicativeApply Either function container type"
        )
      )
      return new AST.GenericType(
        "Either",
        [errorType, outputType],
        applicativeApply.line,
        applicativeApply.column
      )
    }

    if (funcGt.name === "List" && funcGt.typeArguments.length === 1) {
      addConstraint(
        ctx,
        new TypeConstraint(
          funcGt.typeArguments[0],
          funcType,
          applicativeApply.line,
          applicativeApply.column,
          "ApplicativeApply List function container type"
        )
      )
      return new AST.GenericType(
        "List",
        [outputType],
        applicativeApply.line,
        applicativeApply.column
      )
    }

    if (funcGt.name === "Task" && funcGt.typeArguments.length === 1) {
      addConstraint(
        ctx,
        new TypeConstraint(
          funcGt.typeArguments[0],
          funcType,
          applicativeApply.line,
          applicativeApply.column,
          "ApplicativeApply Task function container type"
        )
      )
      return new AST.GenericType(
        "Task",
        [outputType],
        applicativeApply.line,
        applicativeApply.column
      )
    }

    // 汎用
    const funcArgIndex = funcGt.typeArguments.length - 1
    addConstraint(
      ctx,
      new TypeConstraint(
        funcGt.typeArguments[funcArgIndex],
        funcType,
        applicativeApply.line,
        applicativeApply.column,
        "ApplicativeApply generic function container type"
      )
    )
    const newArgs = [...funcGt.typeArguments]
    newArgs[funcArgIndex] = outputType
    return new AST.GenericType(
      funcGt.name,
      newArgs,
      applicativeApply.line,
      applicativeApply.column
    )
  }

  // 型変数の場合
  if (funcContainerType.kind === "TypeVariable") {
    const resultType = freshTypeVariable(
      ctx,
      applicativeApply.line,
      applicativeApply.column
    )
    const applicativeApplyConstraint = new ApplicativeApplyConstraint(
      funcContainerType,
      valueContainerType,
      inputType,
      outputType,
      resultType,
      applicativeApply.line,
      applicativeApply.column,
      "ApplicativeApply with type variable containers"
    )
    ctx.constraints.push(applicativeApplyConstraint)
    return resultType
  }

  // フォールバック
  return new AST.GenericType(
    "Applicative",
    [outputType],
    applicativeApply.line,
    applicativeApply.column
  )
}
