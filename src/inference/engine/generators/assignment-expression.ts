/**
 * AssignmentExpression の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import {
  addConstraint,
  addError,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * 代入式の制約を生成
 * Signal<T>への代入のみサポート
 */
export function generateConstraintsForAssignmentExpression(
  ctx: InferenceContext,
  assignment: AST.AssignmentExpression,
  env: Map<string, AST.Type>
): AST.Type {
  // target（代入先）の型を推論
  const targetType = generateConstraintsForExpression(
    ctx,
    assignment.target,
    env
  )
  // value（代入する値）の型を推論
  const valueType = generateConstraintsForExpression(ctx, assignment.value, env)

  // targetがSignal<T>型かチェック
  if (targetType.kind === "GenericType") {
    const genType = targetType as AST.GenericType
    if (genType.name === "Signal" && genType.typeArguments.length === 1) {
      const signalElementType = genType.typeArguments[0]

      // valueが T または T -> T のどちらかをチェック
      // 関数型かどうかを構文的に判定
      if (assignment.value.kind === "LambdaExpression") {
        // Lambda式の場合は関数型として扱う
        const functionType = new AST.FunctionType(
          signalElementType,
          signalElementType,
          assignment.value.line,
          assignment.value.column
        )
        addConstraint(
          ctx,
          new TypeConstraint(
            valueType,
            functionType,
            assignment.value.line,
            assignment.value.column,
            "Signal assignment function type"
          )
        )
      } else {
        // その他の場合は直接値として扱う
        addConstraint(
          ctx,
          new TypeConstraint(
            valueType,
            signalElementType,
            assignment.value.line,
            assignment.value.column,
            "Signal assignment value type"
          )
        )
      }

      // Signal代入は代入先のSignal型を返す
      return targetType
    }
  }

  addError(
    ctx,
    "Assignment target must be a Signal type",
    assignment.target.line,
    assignment.target.column
  )
  return freshTypeVariable(ctx, assignment.line, assignment.column)
}
