/**
 * FunctionApplicationOperator ($) 演算子の制約生成
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
 * $ 演算子の制約を生成
 * f $ x = f(x)
 */
export function generateConstraintsForFunctionApplicationOperator(
  ctx: InferenceContext,
  funcApp: AST.FunctionApplicationOperator,
  env: Map<string, AST.Type>
): AST.Type {
  const funcType = generateConstraintsForExpression(ctx, funcApp.left, env)
  const argType = generateConstraintsForExpression(ctx, funcApp.right, env)

  const resultType = freshTypeVariable(ctx, funcApp.line, funcApp.column)

  // The function should be of type argType -> resultType
  const expectedFuncType = new AST.FunctionType(
    argType,
    resultType,
    funcApp.line,
    funcApp.column
  )

  addConstraint(
    ctx,
    new TypeConstraint(
      funcType,
      expectedFuncType,
      funcApp.line,
      funcApp.column,
      "Function application operator $"
    )
  )

  return resultType
}
