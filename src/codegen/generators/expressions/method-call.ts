/**
 * メソッド呼び出し（MethodCall）のコード生成
 */

import type { MethodCall } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * メソッド呼び出しをTypeScriptコードに変換
 */
export function generateMethodCall(
  ctx: CodeGenContext,
  call: MethodCall
): string {
  const receiver = generateExpression(ctx, call.receiver)
  const args = call.arguments.map((arg) => generateExpression(ctx, arg))

  // ディスパッチテーブルを使用したメソッド呼び出し
  const allArgs = args.length === 0 ? "" : `, ${args.join(", ")}`
  return `__dispatchMethod(${receiver}, "${call.methodName}"${allArgs})`
}
