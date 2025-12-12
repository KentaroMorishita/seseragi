/**
 * ビルトイン関数呼び出し（BuiltinFunctionCall）のコード生成
 */

import type { BuiltinFunctionCall, Identifier } from "../../../ast"
import { builtinFunctions, type CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * ビルトイン関数呼び出しをTypeScriptコードに変換
 */
export function generateBuiltinFunctionCall(
  ctx: CodeGenContext,
  call: BuiltinFunctionCall
): string {
  const args = call.arguments.map((arg) => generateExpression(ctx, arg))

  // 基本的なビルトイン関数のマッピング
  const builtinFunc = builtinFunctions[call.functionName]
  if (builtinFunc) {
    // 引数チェックが必要な関数
    const singleArgFunctions = [
      "toString",
      "toInt",
      "toFloat",
      "head",
      "tail",
      "show",
    ]
    if (singleArgFunctions.includes(call.functionName) && args.length !== 1) {
      throw new Error(`${call.functionName} requires exactly one argument`)
    }

    return singleArgFunctions.includes(call.functionName)
      ? `${builtinFunc}(${args[0]})`
      : `${builtinFunc}(${args.join(", ")})`
  }

  // 特殊なビルトイン関数の処理
  switch (call.functionName) {
    case "typeof":
      if (args.length !== 1) {
        throw new Error("typeof requires exactly one argument")
      }
      // 引数が単純な変数の場合は変数名も渡す
      if (call.arguments[0]?.kind === "Identifier") {
        const variableName = (call.arguments[0] as Identifier).name
        return `ssrgTypeOf(${args[0]}, "${variableName}")`
      }
      return `ssrgTypeOf(${args[0]})`

    case "typeof'":
      if (args.length !== 1) {
        throw new Error("typeof' requires exactly one argument")
      }
      // 引数が単純な変数の場合は変数名も渡す
      if (call.arguments[0]?.kind === "Identifier") {
        const variableName = (call.arguments[0] as Identifier).name
        return `ssrgTypeOfWithAliases(${args[0]}, "${variableName}")`
      }
      return `ssrgTypeOfWithAliases(${args[0]})`

    case "subscribe":
      if (args.length !== 2) {
        throw new Error("subscribe requires exactly two arguments")
      }
      return `ssrgSignalSubscribe(${args[0]}, ${args[1]})`

    case "unsubscribe":
      if (args.length !== 1) {
        throw new Error("unsubscribe requires exactly one argument")
      }
      return `ssrgSignalUnsubscribe(${args[0]})`

    case "detach":
      if (args.length !== 1) {
        throw new Error("detach requires exactly one argument")
      }
      return `ssrgSignalDetach(${args[0]})`

    default:
      throw new Error(`Unknown builtin function: ${call.functionName}`)
  }
}
