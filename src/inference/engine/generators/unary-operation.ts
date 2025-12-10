/**
 * 単項演算の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import { addConstraint, addError, type InferenceContext } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * 単項演算の制約を生成
 */
export function generateConstraintsForUnaryOperation(
  ctx: InferenceContext,
  expr: AST.UnaryOperation,
  env: Map<string, AST.Type>
): AST.Type {
  const operandType = generateConstraintsForExpression(ctx, expr.operand, env)

  switch (expr.operator) {
    case "-": {
      // 数値の否定: オペランドはInt|Float、結果も同じ
      // Int -> Int, Float -> Float
      // オペランドが数値型であることを確認
      const intType = new AST.PrimitiveType("Int", expr.line, expr.column)
      const floatType = new AST.PrimitiveType("Float", expr.line, expr.column)

      // 数値型のユニオンを期待
      const numericUnion = new AST.UnionType(
        [intType, floatType],
        expr.line,
        expr.column
      )
      addConstraint(
        ctx,
        new TypeConstraint(operandType, numericUnion, expr.line, expr.column)
      )

      // 結果はオペランドと同じ型
      return operandType
    }

    case "!":
    case "not": {
      // 論理否定: Bool -> Bool
      const boolType = new AST.PrimitiveType("Bool", expr.line, expr.column)
      addConstraint(
        ctx,
        new TypeConstraint(operandType, boolType, expr.line, expr.column)
      )
      return boolType
    }

    case "*": {
      // Signal値取得: Signal<T> -> T
      // operandTypeがSignal<T>の場合、Tを返す
      if (operandType.kind === "GenericType") {
        const genType = operandType as AST.GenericType
        if (genType.name === "Signal" && genType.typeArguments.length === 1) {
          return genType.typeArguments[0]! // Signal<T> -> T
        }
      }

      // Signal型でない場合はエラー
      addError(
        ctx,
        `Dereference operator (*) can only be applied to Signal types`,
        expr.line,
        expr.column
      )
      return operandType
    }

    default:
      addError(
        ctx,
        `Unknown unary operator: ${expr.operator}`,
        expr.line,
        expr.column
      )
      return operandType
  }
}
