/**
 * Promise関連式のコード生成
 *
 * - PromiseBlock: promise<T> { ... }
 * - ResolveExpression: resolve<T>(value)
 * - RejectExpression: reject<T>(error)
 * - TryExpression: try expr as ErrorType
 */

import type {
  Expression,
  FunctionCall,
  Identifier,
  PromiseBlock,
  RejectExpression,
  ResolveExpression,
  TryExpression,
  Type,
} from "../../../ast"
import type { CodeGenContext } from "../../context"
import {
  enterPromiseBlock,
  exitPromiseBlock,
  isInsidePromiseBlock,
} from "../../context"
import { generateExpression } from "../dispatcher"
import { generateStatement } from "../statements"

/**
 * 型をTypeScript文字列に変換（シンプル版）
 */
function generateType(type: Type): string {
  if (!type) return "any"

  switch (type.kind) {
    case "PrimitiveType": {
      // Seseragiの型をTypeScriptの型に変換
      const typeMap: Record<string, string> = {
        Int: "number",
        Float: "number",
        Bool: "boolean",
        String: "string",
        Char: "string",
        Unit: "void",
      }
      return typeMap[(type as any).name] || (type as any).name
    }
    case "FunctionType":
      return `(${generateType((type as any).paramType)}) => ${generateType((type as any).returnType)}`
    case "GenericType": {
      const args =
        (type as any).typeArguments?.map(generateType).join(", ") || ""
      return args ? `${(type as any).name}<${args}>` : (type as any).name
    }
    case "RecordType": {
      const fields = (type as any).fields
        ?.map((f: any) => `${f.name}: ${generateType(f.type)}`)
        .join(", ")
      return `{ ${fields} }`
    }
    case "TupleType": {
      const elements =
        (type as any).elementTypes?.map(generateType).join(", ") || ""
      return `[${elements}]`
    }
    default:
      return "any"
  }
}

/**
 * PromiseBlock式の生成
 *
 * promise<T> { ... } -> () => new Promise<T>((resolve, reject) => { ... })
 */
export function generatePromiseBlock(
  ctx: CodeGenContext,
  expr: PromiseBlock
): string {
  // promiseブロックに入る
  enterPromiseBlock(ctx)

  let body = ""
  const indent = "  "

  // ステートメント生成
  for (const stmt of expr.statements) {
    body += `${indent}${generateStatement(ctx, stmt)}\n`
  }

  // 戻り値式があれば追加
  if (expr.returnExpression) {
    const returnExpr = generateExpression(ctx, expr.returnExpression)
    // resolve/rejectはそのまま
    body += `${indent}${returnExpr};\n`
  }

  // promiseブロックから出る
  exitPromiseBlock(ctx)

  // 型引数がある場合はTypeScript型注釈を追加
  let typeAnnotation = ""
  if (expr.typeArgument) {
    typeAnnotation = `<${generateType(expr.typeArgument)}>`
  }

  return `() => new Promise${typeAnnotation}((resolve, reject) => {\n${body}})`
}

/**
 * ResolveExpression式の生成
 *
 * promiseブロック内: resolve(value)
 * promiseブロック外: () => Promise.resolve(value)
 */
export function generateResolveExpression(
  ctx: CodeGenContext,
  expr: ResolveExpression
): string {
  const value = generateExpression(ctx, expr.value)

  if (isInsidePromiseBlock(ctx)) {
    // promiseブロック内では型引数に関係なくローカルresolve呼び出し
    return `resolve(${value})`
  }
  // promiseブロック外では独立関数として処理
  if (expr.typeArgument) {
    const typeAnnotation = generateType(expr.typeArgument)
    return `() => Promise.resolve<${typeAnnotation}>(${value})`
  }
  // 型推論の場合
  return `() => Promise.resolve(${value})`
}

/**
 * RejectExpression式の生成
 *
 * promiseブロック内: reject(error)
 * promiseブロック外: () => Promise.reject(error)
 */
export function generateRejectExpression(
  ctx: CodeGenContext,
  expr: RejectExpression
): string {
  const value = generateExpression(ctx, expr.value)

  if (isInsidePromiseBlock(ctx)) {
    // promiseブロック内では型引数に関係なくローカルreject呼び出し
    return `reject(${value})`
  }
  // promiseブロック外では独立関数として処理
  if (expr.typeArgument) {
    const typeAnnotation = generateType(expr.typeArgument)
    return `() => Promise.reject<${typeAnnotation}>(${value})`
  }
  // 型推論の場合
  return `() => Promise.reject(${value})`
}

/**
 * TryExpression式の生成
 *
 * 同期版: () => Either<L, T>
 * 非同期版: async () => Promise<Either<L, T>>
 */
export function generateTryExpression(
  ctx: CodeGenContext,
  expr: TryExpression
): string {
  const innerExpr = generateExpression(ctx, expr.expression)

  // Promise型検出：resolve呼び出しやpromiseブロックかどうか
  const isPromise = isPromiseExpression(ctx, expr.expression)

  if (isPromise) {
    // seseragiのPromise関数（Unit -> Promise<T>）かどうか判定
    const needsExecution = isSeseragiPromiseFunction(ctx, expr.expression)
    const awaitTarget = needsExecution ? `(${innerExpr})()` : innerExpr

    // 非同期版: () => Promise<Either<L, T>>
    if (expr.errorType) {
      // 型指定あり: error as L
      return `async (): Promise<Either<${generateType(expr.errorType)}, any>> => {
  try {
    const value = await ${awaitTarget};
    return { tag: "Right" as const, value };
  } catch (error) {
    return { tag: "Left" as const, value: error as ${generateType(expr.errorType)} };
  }
}`
    }
    // 型指定なし: String(error)
    return `async (): Promise<Either<string, any>> => {
  try {
    const value = await ${awaitTarget};
    return { tag: "Right" as const, value };
  } catch (error) {
    return { tag: "Left" as const, value: String(error) };
  }
}`
  }
  // 同期版: () => Either<L, T>
  if (expr.errorType) {
    // 型指定あり: error as L
    return `(): Either<${generateType(expr.errorType)}, any> => {
  try {
    return { tag: "Right" as const, value: ${innerExpr} };
  } catch (error) {
    return { tag: "Left" as const, value: error as ${generateType(expr.errorType)} };
  }
}`
  }
  // 型指定なし: String(error)
  return `(): Either<string, any> => {
  try {
    return { tag: "Right" as const, value: ${innerExpr} };
  } catch (error) {
    return { tag: "Left" as const, value: String(error) };
  }
}`
}

// ============================================================
// ヘルパー関数
// ============================================================

/**
 * Promise式かどうかを判定
 */
function isPromiseExpression(ctx: CodeGenContext, expr: Expression): boolean {
  if (!expr || !expr.kind) {
    return false
  }

  // ParenthesesExpressionの場合は中身を確認
  if (expr.kind === "ParenthesesExpression") {
    return isPromiseExpression(ctx, (expr as any).expression)
  }

  // ResolveExpression / RejectExpression の場合
  if (expr.kind === "ResolveExpression" || expr.kind === "RejectExpression") {
    return true
  }

  // resolve/reject呼び出しの場合（FunctionCallとして解析される場合）
  if (expr.kind === "FunctionCall") {
    const funcCall = expr as FunctionCall
    if (
      funcCall.function.kind === "Identifier" &&
      ((funcCall.function as Identifier).name === "resolve" ||
        (funcCall.function as Identifier).name === "reject")
    ) {
      return true
    }
  }

  // promiseブロックの場合
  if (expr.kind === "PromiseBlock") {
    return true
  }

  // TernaryExpressionの場合、両分岐がPromiseならPromise
  if (expr.kind === "TernaryExpression") {
    const ternaryExpr = expr as any // TernaryExpression
    const trueIsPromise = isPromiseExpression(ctx, ternaryExpr.trueExpression)
    const falseIsPromise = isPromiseExpression(ctx, ternaryExpr.falseExpression)
    return trueIsPromise && falseIsPromise
  }

  // Identifierの場合は型推論結果を重点的にチェック
  if (expr.kind === "Identifier") {
    // 型推論結果を使ってPromise型をチェック
    if (ctx.typeInferenceResult?.nodeTypeMap?.has(expr)) {
      const exprType = ctx.typeInferenceResult.nodeTypeMap.get(expr)
      if (exprType && isPromiseType(exprType)) {
        return true
      }
    }
  }

  // 型推論結果を使ってPromise型をチェック
  if (ctx.typeInferenceResult?.nodeTypeMap?.has(expr)) {
    const exprType = ctx.typeInferenceResult.nodeTypeMap.get(expr)
    if (exprType && isPromiseType(exprType)) {
      return true
    }
  }

  // 関数呼び出しで関数名に"async"が含まれる場合（簡易判定）
  if (expr.kind === "FunctionCall") {
    const funcCall = expr as FunctionCall
    if (funcCall.function.kind === "Identifier") {
      const funcName = (funcCall.function as Identifier).name
      return funcName.includes("async") || funcName.includes("Promise")
    }
  }

  return false
}

/**
 * Promise型かどうかを判定
 */
function isPromiseType(type: Type): boolean {
  if (type.kind === "GenericType") {
    const genericType = type as any
    return genericType.name === "Promise"
  }

  // 関数型の場合、戻り値型がPromiseかチェック
  if (type.kind === "FunctionType") {
    const funcType = type as any
    return isPromiseType(funcType.returnType)
  }

  // UnionType の場合、すべてのメンバーがPromise型かチェック
  if (type.kind === "UnionType") {
    const unionType = type as any // UnionType
    return unionType.types?.every((memberType: Type) =>
      isPromiseType(memberType)
    )
  }

  return false
}

/**
 * Seseragi Promise関数（Unit -> Promise<T>）かどうかを判定
 */
function isSeseragiPromiseFunction(
  ctx: CodeGenContext,
  expr: Expression
): boolean {
  if (!expr || !expr.kind) {
    return false
  }

  // ParenthesesExpressionの場合は中身を確認
  if (expr.kind === "ParenthesesExpression") {
    return isSeseragiPromiseFunction(ctx, (expr as any).expression)
  }

  // promiseブロック（Unit -> Promise<T>として生成される）
  if (expr.kind === "PromiseBlock") {
    return true
  }

  // TernaryExpressionの場合、両分岐をチェック
  if (expr.kind === "TernaryExpression") {
    const ternaryExpr = expr as any // TernaryExpression
    const trueNeedsExecution = isSeseragiPromiseFunction(
      ctx,
      ternaryExpr.trueExpression
    )
    const falseNeedsExecution = isSeseragiPromiseFunction(
      ctx,
      ternaryExpr.falseExpression
    )
    // 両方とも同じ実行要件を持つ場合のみtrue
    return trueNeedsExecution && falseNeedsExecution
  }

  // ResolveExpression / RejectExpression（Unit -> Promise<T>として生成される）
  if (expr.kind === "ResolveExpression" || expr.kind === "RejectExpression") {
    return true
  }

  // resolve/reject呼び出し（FunctionCallとして解析される場合）
  if (expr.kind === "FunctionCall") {
    const funcCall = expr as FunctionCall
    if (funcCall.function.kind === "Identifier") {
      const funcName = (funcCall.function as Identifier).name
      return funcName === "resolve" || funcName === "reject"
    }
  }

  // Identifierの場合、Promise関数を格納している変数は呼び出し必要
  if (expr.kind === "Identifier") {
    // 型推論結果を使ってPromise関数型かチェック
    if (ctx.typeInferenceResult?.nodeTypeMap?.has(expr)) {
      const exprType = ctx.typeInferenceResult.nodeTypeMap.get(expr)
      if (exprType && isPromiseType(exprType)) {
        return true // Promise関数なので()付きで呼び出し必要
      }
    }
  }

  return false
}
