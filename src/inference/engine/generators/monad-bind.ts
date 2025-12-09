/**
 * MonadBind (>>=) 演算子の制約生成
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
 * MonadBind演算子の制約を生成
 * m a >>= (a -> m b) => m b
 */
export function generateConstraintsForMonadBind(
  ctx: InferenceContext,
  monadBind: AST.MonadBind,
  env: Map<string, AST.Type>
): AST.Type {
  const monadType = generateConstraintsForExpression(ctx, monadBind.left, env)
  const funcType = generateConstraintsForExpression(ctx, monadBind.right, env)

  const inputType = freshTypeVariable(ctx, monadBind.line, monadBind.column)
  const outputType = freshTypeVariable(ctx, monadBind.line, monadBind.column)

  // Create a generic monad variable that will be constrained later
  const monadVar = freshTypeVariable(ctx, monadBind.line, monadBind.column)

  // Constrain the left side to be a monad of inputType
  addConstraint(
    ctx,
    new TypeConstraint(
      monadType,
      monadVar,
      monadBind.line,
      monadBind.column,
      "MonadBind left side monad type"
    )
  )

  // The function should take inputType and return a monad of outputType
  const expectedOutputMonad = freshTypeVariable(
    ctx,
    monadBind.line,
    monadBind.column
  )
  const expectedFuncType = new AST.FunctionType(
    inputType,
    expectedOutputMonad,
    monadBind.line,
    monadBind.column
  )

  addConstraint(
    ctx,
    new TypeConstraint(
      funcType,
      expectedFuncType,
      monadBind.line,
      monadBind.column,
      "MonadBind function type"
    )
  )

  // The result should have the same monad structure as the input but with outputType
  const resultType = freshTypeVariable(ctx, monadBind.line, monadBind.column)

  // Add constraint that the output monad and result should be the same
  addConstraint(
    ctx,
    new TypeConstraint(
      expectedOutputMonad,
      resultType,
      monadBind.line,
      monadBind.column,
      "MonadBind result type"
    )
  )

  // Add specific constraints for known monad types
  if (monadType.kind === "GenericType") {
    const monadGt = monadType as AST.GenericType

    if (monadGt.name === "Maybe" && monadGt.typeArguments.length === 1) {
      addConstraint(
        ctx,
        new TypeConstraint(
          monadGt.typeArguments[0],
          inputType,
          monadBind.line,
          monadBind.column,
          "MonadBind Maybe input type"
        )
      )

      const outputMonadType = new AST.GenericType(
        "Maybe",
        [outputType],
        monadBind.line,
        monadBind.column
      )

      addConstraint(
        ctx,
        new TypeConstraint(
          expectedOutputMonad,
          outputMonadType,
          monadBind.line,
          monadBind.column,
          "MonadBind Maybe output type"
        )
      )

      return new AST.GenericType(
        "Maybe",
        [outputType],
        monadBind.line,
        monadBind.column
      )
    }

    if (monadGt.name === "Either" && monadGt.typeArguments.length === 2) {
      const errorType = monadGt.typeArguments[0]
      const valueType = monadGt.typeArguments[1]

      addConstraint(
        ctx,
        new TypeConstraint(
          valueType,
          inputType,
          monadBind.line,
          monadBind.column,
          "MonadBind Either input type"
        )
      )

      const outputMonadType = new AST.GenericType(
        "Either",
        [errorType, outputType],
        monadBind.line,
        monadBind.column
      )

      addConstraint(
        ctx,
        new TypeConstraint(
          expectedOutputMonad,
          outputMonadType,
          monadBind.line,
          monadBind.column,
          "MonadBind Either output type"
        )
      )

      return new AST.GenericType(
        "Either",
        [errorType, outputType],
        monadBind.line,
        monadBind.column
      )
    }

    if (monadGt.name === "List" && monadGt.typeArguments.length === 1) {
      addConstraint(
        ctx,
        new TypeConstraint(
          monadGt.typeArguments[0],
          inputType,
          monadBind.line,
          monadBind.column,
          "MonadBind List input type"
        )
      )

      const outputMonadType = new AST.GenericType(
        "List",
        [outputType],
        monadBind.line,
        monadBind.column
      )

      addConstraint(
        ctx,
        new TypeConstraint(
          expectedOutputMonad,
          outputMonadType,
          monadBind.line,
          monadBind.column,
          "MonadBind List output type"
        )
      )

      return new AST.GenericType(
        "List",
        [outputType],
        monadBind.line,
        monadBind.column
      )
    }

    if (monadGt.name === "Task" && monadGt.typeArguments.length === 1) {
      addConstraint(
        ctx,
        new TypeConstraint(
          monadGt.typeArguments[0],
          inputType,
          monadBind.line,
          monadBind.column,
          "MonadBind Task input type"
        )
      )

      const outputMonadType = new AST.GenericType(
        "Task",
        [outputType],
        monadBind.line,
        monadBind.column
      )

      addConstraint(
        ctx,
        new TypeConstraint(
          expectedOutputMonad,
          outputMonadType,
          monadBind.line,
          monadBind.column,
          "MonadBind Task output type"
        )
      )

      return new AST.GenericType(
        "Task",
        [outputType],
        monadBind.line,
        monadBind.column
      )
    }
  }

  // Handle Array type (JavaScript arrays)
  if (
    monadType.kind === "PrimitiveType" &&
    (monadType as AST.PrimitiveType).name === "Array"
  ) {
    return new AST.PrimitiveType("Array", monadBind.line, monadBind.column)
  }

  // For unknown types, let constraint resolution figure it out
  return resultType
}
