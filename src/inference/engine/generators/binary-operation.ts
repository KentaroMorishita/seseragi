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
 * 演算子オーバーロードを検索
 */
function findOperatorDefinition(
  ctx: InferenceContext,
  structType: AST.Type,
  operator: string
): AST.OperatorDeclaration | null {
  // StructType または同名の PrimitiveType の場合のみ検索
  let structTypeName: string | null = null
  if (structType.kind === "StructType") {
    structTypeName = (structType as AST.StructType).name
  } else if (structType.kind === "PrimitiveType") {
    // PrimitiveType名が構造体として定義されているかチェック
    const primName = (structType as AST.PrimitiveType).name
    // ctx.structTypes にあるか確認
    if (ctx.structTypes.has(primName)) {
      structTypeName = primName
    }
  }

  if (!structTypeName || !ctx.currentProgram) {
    return null
  }

  // ImplBlockを検索
  for (const stmt of ctx.currentProgram.statements) {
    if (stmt.kind === "ImplBlock") {
      const implBlock = stmt as AST.ImplBlock
      if (implBlock.typeName === structTypeName) {
        for (const op of implBlock.operators) {
          if (op.operator === operator) {
            return op
          }
        }
      }
    }
  }
  return null
}

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
      // 構造体の演算子オーバーロードをチェック
      const operatorDef = findOperatorDefinition(ctx, leftType, binOp.operator)
      if (operatorDef) {
        // 演算子定義が見つかった場合、その戻り値型を使用
        return operatorDef.returnType
      }

      // 演算子定義がない場合は従来の処理
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
      // 構造体の演算子オーバーロードをチェック
      const logicalOperatorDef = findOperatorDefinition(
        ctx,
        leftType,
        binOp.operator
      )
      if (logicalOperatorDef) {
        return logicalOperatorDef.returnType
      }

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
