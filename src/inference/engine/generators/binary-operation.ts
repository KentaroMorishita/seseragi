/**
 * 二項演算の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import { typeToString } from "../../type-formatter"
import {
  addConstraint,
  addError,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * 二項演算の制約を生成し、結果の型を返す
 */
export function generateConstraintsForBinaryOperation(
  ctx: InferenceContext,
  binOp: AST.BinaryOperation,
  env: Map<string, AST.Type>
): AST.Type {
  // 左右のオペランドの型を推論
  const leftType = generateConstraintsForExpression(ctx, binOp.left, env)
  const rightType = generateConstraintsForExpression(ctx, binOp.right, env)

  // 式に型情報を設定
  binOp.left.type = leftType
  binOp.right.type = rightType

  switch (binOp.operator) {
    case "+":
    case "-":
    case "*":
    case "/":
    case "%":
    case "**": {
      // 数値演算: 両オペランドは同じ型でなければならず、結果も同じ型
      addConstraint(
        ctx,
        new TypeConstraint(
          leftType,
          rightType,
          binOp.line,
          binOp.column,
          `Binary operation ${binOp.operator} operands must have same type`
        )
      )
      return leftType
    }

    case "==":
    case "!=":
    case "<":
    case ">":
    case "<=":
    case ">=": {
      // 比較演算: 両オペランドは同じ型、結果はBool
      addConstraint(
        ctx,
        new TypeConstraint(
          leftType,
          rightType,
          binOp.line,
          binOp.column,
          `Comparison ${binOp.operator} operands must match`
        )
      )
      return new AST.PrimitiveType("Bool", binOp.line, binOp.column)
    }

    case "&&":
    case "||": {
      // 論理演算: 両オペランドはBool、結果もBool
      const boolType = new AST.PrimitiveType("Bool", binOp.line, binOp.column)
      addConstraint(
        ctx,
        new TypeConstraint(
          leftType,
          boolType,
          binOp.left.line,
          binOp.left.column,
          `Logical operation ${binOp.operator} left operand`
        )
      )
      addConstraint(
        ctx,
        new TypeConstraint(
          rightType,
          boolType,
          binOp.right.line,
          binOp.right.column,
          `Logical operation ${binOp.operator} right operand`
        )
      )
      return boolType
    }

    case ":": {
      // CONS演算子: a : List<a> -> List<a>
      const expectedListType = new AST.GenericType(
        "List",
        [leftType],
        binOp.right.line,
        binOp.right.column
      )

      addConstraint(
        ctx,
        new TypeConstraint(
          rightType,
          expectedListType,
          binOp.right.line,
          binOp.right.column,
          `CONS operator (:) right operand must be List<${typeToString(leftType)}>`
        )
      )

      return expectedListType
    }

    case ":=": {
      // Signal代入演算子: Signal<a> := a -> Signal<a>
      if (leftType.kind === "GenericType") {
        const genType = leftType as AST.GenericType
        if (genType.name === "Signal" && genType.typeArguments.length === 1) {
          const signalValueType = genType.typeArguments[0]
          if (signalValueType) {
            addConstraint(
              ctx,
              new TypeConstraint(
                rightType,
                signalValueType,
                binOp.right.line,
                binOp.right.column,
                `Signal assignment (:=) value must match Signal type`
              )
            )
            return leftType
          }
        }
      }

      addError(
        ctx,
        `Signal assignment (:=) can only be applied to Signal types`,
        binOp.left.line,
        binOp.left.column
      )
      return freshTypeVariable(ctx, binOp.line, binOp.column)
    }

    default:
      addError(
        ctx,
        `Unknown binary operator: ${binOp.operator}`,
        binOp.line,
        binOp.column
      )
      return freshTypeVariable(ctx, binOp.line, binOp.column)
  }
}
