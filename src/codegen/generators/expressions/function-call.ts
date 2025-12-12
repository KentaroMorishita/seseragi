/**
 * 関数呼び出し（FunctionCall）のコード生成
 */

import type { FunctionCall, Identifier, Literal } from "../../../ast"
import { builtinFunctions, type CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * 関数呼び出しをTypeScriptコードに変換
 */
export function generateFunctionCall(
  ctx: CodeGenContext,
  call: FunctionCall
): string {
  // ビルトイン関数の場合は直接変換
  if (call.function.kind === "Identifier") {
    const identifier = call.function as Identifier
    const builtinFunc = builtinFunctions[identifier.name]

    if (builtinFunc) {
      const args = call.arguments.map((arg) => generateExpression(ctx, arg))
      return `${builtinFunc}(${args.join(", ")})`
    }
  }

  const func = generateExpression(ctx, call.function)

  // PromiseBlockの関数呼び出しの場合は括弧で囲む
  const wrappedFunc = call.function.kind === "PromiseBlock" ? `(${func})` : func

  // 引数の生成（シンプル版 - 型アサーションなし）
  const args = call.arguments.map((arg) => {
    const argCode = generateExpression(ctx, arg)

    // Unit値をvoid引数に渡す場合は.valueを付与
    if (arg.kind === "Literal" && (arg as Literal).literalType === "unit") {
      return `${argCode}.value`
    }

    if (arg.kind === "Identifier") {
      const argName = (arg as Identifier).name
      if (
        argCode === "Unit" ||
        argName.includes("unit") ||
        argName === "a_prime"
      ) {
        return `${argCode}.value`
      }
    }

    return argCode
  })

  // 型引数がある場合の処理
  if (call.typeArguments && call.typeArguments.length > 0) {
    // 型引数の生成は後で実装（generateTypeが必要）
    // 今はスキップ
    return `${wrappedFunc}(${args.join(", ")})`
  }

  return `${wrappedFunc}(${args.join(", ")})`
}
