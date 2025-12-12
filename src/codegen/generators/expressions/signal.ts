/**
 * Signal関連式の生成
 */

import type { AssignmentExpression, SignalExpression } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * Signal式をTypeScriptコードに変換
 * Signal 42 -> createSignal(42)
 */
export function generateSignalExpression(
  ctx: CodeGenContext,
  signalExpr: SignalExpression
): string {
  const initialValue = generateExpression(ctx, signalExpr.initialValue)
  return `createSignal(${initialValue})`
}

/**
 * Signal代入式をTypeScriptコードに変換
 * signal := value -> setSignal(signal, value)
 */
export function generateAssignmentExpression(
  ctx: CodeGenContext,
  assignmentExpr: AssignmentExpression
): string {
  const target = generateExpression(ctx, assignmentExpr.target)
  const value = generateExpression(ctx, assignmentExpr.value)
  return `setSignal(${target}, ${value})`
}
