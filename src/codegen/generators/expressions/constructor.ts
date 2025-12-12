/**
 * コンストラクタ式の生成
 */

import type { ConstructorExpression } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * コンストラクタ式をTypeScriptコードに変換
 */
export function generateConstructorExpression(
  ctx: CodeGenContext,
  expr: ConstructorExpression
): string {
  const name = expr.constructorName
  const args = expr.arguments.map((arg) => generateExpression(ctx, arg))

  // 特殊なコンストラクタのマッピング
  const constructorMap: Record<
    string,
    {
      name?: string
      generator?: (args: string[]) => string
    }
  > = {
    Nothing: { generator: () => "Nothing" },
    Just: { name: "Just" },
    Left: { name: "Left" },
    Right: { name: "Right" },
    Empty: { generator: () => "Empty" },
    Cons: {
      generator: (args) =>
        args.length === 2 ? `Cons(${args[0]}, ${args[1]})` : "Cons",
    },
    Task: { name: "Task" },
    Signal: { name: "createSignal" },
  }

  const constructorConfig = constructorMap[name]
  if (constructorConfig) {
    if (constructorConfig.generator) {
      return constructorConfig.generator(args)
    }
    const funcName = constructorConfig.name || name
    return args.length > 0 ? `${funcName}(${args[0]})` : funcName
  }

  // 一般的なコンストラクタ
  return args.length > 0 ? `${name}(${args.join(", ")})` : name
}
