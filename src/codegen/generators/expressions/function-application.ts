/**
 * 関数適用（FunctionApplication）のコード生成
 */

import {
  FunctionApplication,
  Identifier,
  LambdaExpression,
  type Literal,
} from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * 関数適用をTypeScriptコードに変換
 */
export function generateFunctionApplication(
  ctx: CodeGenContext,
  app: FunctionApplication
): string {
  // ビルトイン関数の特別処理（run, tryRun）
  if (app.function.kind === "Identifier") {
    const identifier = app.function as Identifier
    const arg = generateExpression(ctx, app.argument)

    switch (identifier.name) {
      case "run":
        return `ssrgRun(${arg})`
      case "tryRun":
        return `ssrgTryRun(${arg})`
    }
  }

  const func = generateExpression(ctx, app.function)
  let arg = generateExpression(ctx, app.argument)

  // Unit値をvoid引数に渡す場合は.valueを付与
  if (
    app.argument.kind === "Literal" &&
    (app.argument as Literal).literalType === "unit"
  ) {
    arg = `${arg}.value`
  } else if (app.argument.kind === "Identifier") {
    const argName = (app.argument as Identifier).name
    if (
      arg === "Unit" ||
      argName.includes("unit") ||
      argName === "a'" ||
      arg === "a_prime"
    ) {
      arg = `${arg}.value`
    }
  }

  // ビルトイン関数の特別処理
  const builtinResult = tryGenerateBuiltinApplication(ctx, app, arg)
  if (builtinResult) {
    return builtinResult
  }

  // ネストした関数適用の処理
  const nestedResult = tryGenerateNestedApplication(ctx, app, arg)
  if (nestedResult) {
    return nestedResult
  }

  // 通常の関数適用
  return generateRegularApplication(app, func, arg)
}

/**
 * ビルトイン関数適用の生成を試みる
 */
function tryGenerateBuiltinApplication(
  _ctx: CodeGenContext,
  app: FunctionApplication,
  arg: string
): string | null {
  if (app.function instanceof Identifier) {
    const funcName = app.function.name
    const builtinMap: Record<string, string> = {
      print: `ssrgPrint(${arg})`,
      putStrLn: `ssrgPutStrLn(${arg})`,
      toString: `ssrgToString(${arg})`,
      toInt: `ssrgToInt(${arg})`,
      toFloat: `ssrgToFloat(${arg})`,
      head: `headList(${arg})`,
      tail: `tailList(${arg})`,
      show: `ssrgShow(${arg})`,
      typeof: generateTypeOfCall(app, arg),
      "typeof'": generateTypeOfWithAliasesCall(app, arg),
    }
    return builtinMap[funcName] || null
  }
  return null
}

/**
 * typeof呼び出しの生成
 */
function generateTypeOfCall(app: FunctionApplication, arg: string): string {
  if (app.argument.kind === "Identifier") {
    const variableName = (app.argument as Identifier).name
    return `ssrgTypeOf(${arg}, "${variableName}")`
  }
  return `ssrgTypeOf(${arg})`
}

/**
 * typeof'呼び出しの生成（エイリアス付き）
 */
function generateTypeOfWithAliasesCall(
  app: FunctionApplication,
  arg: string
): string {
  if (app.argument.kind === "Identifier") {
    const variableName = (app.argument as Identifier).name
    return `ssrgTypeOfWithAliases(${arg}, "${variableName}")`
  }
  return `ssrgTypeOfWithAliases(${arg})`
}

/**
 * ネストした関数適用の生成を試みる
 */
function tryGenerateNestedApplication(
  ctx: CodeGenContext,
  app: FunctionApplication,
  arg: string
): string | null {
  if (app.function instanceof FunctionApplication) {
    const nestedFunc = app.function.function
    if (nestedFunc instanceof Identifier) {
      const funcName = nestedFunc.name
      if (funcName === "print" || funcName === "putStrLn") {
        const firstArg = generateExpression(ctx, app.function.argument)
        return `console.log(${firstArg}, ${arg})`
      }
    }
  }
  return null
}

/**
 * 通常の関数適用の生成
 */
function generateRegularApplication(
  app: FunctionApplication,
  func: string,
  arg: string
): string {
  // ラムダ式は括弧で囲む
  if (app.function instanceof LambdaExpression) {
    return `(${func})(${arg})`
  }
  return `${func}(${arg})`
}
