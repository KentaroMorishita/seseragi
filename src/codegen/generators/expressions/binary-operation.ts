/**
 * 二項演算（BinaryOperation）のコード生成
 */

import type { BinaryOperation } from "../../../ast"
import { type CodeGenContext, getResolvedType } from "../../context"
import { isBasicOperator, isMonadOperator } from "../../helpers"
import {
  isArrayType,
  isEitherType,
  isIntType,
  isListType,
  isMaybeType,
  isPrimitiveType,
  isSignalType,
  isTaskType,
} from "../../type-utils"
import { generateExpression } from "../dispatcher"

/**
 * 二項演算をTypeScriptコードに変換
 */
export function generateBinaryOperation(
  ctx: CodeGenContext,
  binOp: BinaryOperation
): string {
  const left = generateExpression(ctx, binOp.left)
  const right = generateExpression(ctx, binOp.right)

  // CONS演算子の特別処理
  if (binOp.operator === ":") {
    return `Cons(${left}, ${right})`
  }

  // Signal代入演算子の特別処理
  if (binOp.operator === ":=") {
    return `${left}.setValue(${right})`
  }

  // モナド演算子の特別処理
  if (isMonadOperator(binOp.operator)) {
    const result = generateMonadOperation(ctx, binOp, left, right)
    if (result) return result
  }

  // 解決済みの型を取得
  const leftType = getResolvedType(ctx, binOp.left)
  const rightType = getResolvedType(ctx, binOp.right)

  // 両辺がプリミティブ型の場合は直接演算子を使用
  if (
    isBasicOperator(binOp.operator) &&
    isPrimitiveType(leftType) &&
    isPrimitiveType(rightType)
  ) {
    let operator = binOp.operator
    if (operator === "==") operator = "==="
    if (operator === "!=") operator = "!=="

    // Int/Int除算の特別処理 - Math.trunc()で切り捨て
    if (operator === "/" && isIntType(leftType) && isIntType(rightType)) {
      return `Math.trunc(${left} / ${right})`
    }

    return `(${left} ${operator} ${right})`
  }

  // 構造体の演算子オーバーロードの可能性がある場合は演算子ディスパッチを使用
  return generateOperatorDispatch(binOp.operator, left, right)
}

/**
 * モナド演算子（<$>, <*>, >>=）の生成
 */
function generateMonadOperation(
  ctx: CodeGenContext,
  binOp: BinaryOperation,
  left: string,
  right: string
): string | null {
  const leftType = getResolvedType(ctx, binOp.left)

  // Task型の場合
  if (isTaskType(ctx, leftType)) {
    switch (binOp.operator) {
      case "<$>":
        return `mapTask(${right}, ${left})`
      case "<*>":
        return `applyTask(${left}, ${right})`
      case ">>=":
        return `bindTask(${left}, ${right})`
    }
  }

  // Maybe型の場合
  if (isMaybeType(ctx, leftType)) {
    switch (binOp.operator) {
      case "<$>":
        return `mapMaybe(${left}, ${right})`
      case "<*>":
        return `applyMaybe(${left}, ${right})`
      case ">>=":
        return `bindMaybe(${left}, ${right})`
    }
  }

  // Either型の場合
  if (isEitherType(ctx, leftType)) {
    switch (binOp.operator) {
      case "<$>":
        return `mapEither(${left}, ${right})`
      case "<*>":
        return `applyEither(${left}, ${right})`
      case ">>=":
        return `bindEither(${left}, ${right})`
    }
  }

  // List型の場合
  if (isListType(ctx, leftType)) {
    switch (binOp.operator) {
      case "<$>":
        return `mapList(${left}, ${right})`
      case "<*>":
        return `applyList(${left}, ${right})`
      case ">>=":
        return `bindList(${left}, ${right})`
    }
  }

  // Array型の場合
  if (isArrayType(ctx, leftType)) {
    switch (binOp.operator) {
      case "<$>":
        return `mapArray(${left}, ${right})`
      case "<*>":
        return `applyArray(${left}, ${right})`
      case ">>=":
        return `bindArray(${left}, ${right})`
    }
  }

  // Signal型の場合
  if (isSignalType(ctx, leftType)) {
    switch (binOp.operator) {
      case "<$>":
        return `mapSignal(${left}, ${right})`
      case "<*>":
        return `applySignal(${left}, ${right})`
      case ">>=":
        return `bindSignal(${left}, ${right})`
    }
  }

  return null
}

/**
 * 演算子ディスパッチの生成（構造体の演算子オーバーロード用）
 */
function generateOperatorDispatch(
  operator: string,
  left: string,
  right: string
): string {
  return `__dispatchOperator(${left}, "${operator}", ${right})`
}
