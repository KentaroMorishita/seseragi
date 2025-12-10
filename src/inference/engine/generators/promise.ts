/**
 * Promise関連の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import {
  addConstraint,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * PromiseBlock: promise { ... } 構文
 */
export function generateConstraintsForPromiseBlock(
  ctx: InferenceContext,
  promiseBlock: AST.PromiseBlock,
  env: Map<string, AST.Type>
): AST.Type {
  // Promise block内の環境を作成（resolveとrejectが利用可能）
  const promiseEnv = new Map(env)

  // 型引数が明示的に指定されている場合はそれを使用
  let resolveType: AST.Type
  if (promiseBlock.typeArgument) {
    resolveType = promiseBlock.typeArgument
  } else {
    // 型引数がない場合は新しい型変数を生成
    resolveType = freshTypeVariable(ctx, promiseBlock.line, promiseBlock.column)
  }

  // resolve関数の型: T -> Unit
  const resolveFunc = new AST.FunctionType(
    resolveType,
    new AST.PrimitiveType("Unit", promiseBlock.line, promiseBlock.column),
    promiseBlock.line,
    promiseBlock.column
  )
  promiseEnv.set("resolve", resolveFunc)

  // reject関数の型: String -> Unit
  const rejectFunc = new AST.FunctionType(
    new AST.PrimitiveType("String", promiseBlock.line, promiseBlock.column),
    new AST.PrimitiveType("Unit", promiseBlock.line, promiseBlock.column),
    promiseBlock.line,
    promiseBlock.column
  )
  promiseEnv.set("reject", rejectFunc)

  // ブロック本体の型を推論
  for (const stmt of promiseBlock.statements) {
    generateConstraintsForStatement(ctx, stmt, promiseEnv)
  }

  // 戻り値式があれば推論
  if (promiseBlock.returnExpression) {
    generateConstraintsForExpression(ctx, promiseBlock.returnExpression, promiseEnv)

    // 型引数が省略されている場合、resolve式から型を推論
    if (
      !promiseBlock.typeArgument &&
      promiseBlock.returnExpression.kind === "ResolveExpression"
    ) {
      const resolveExpr = promiseBlock.returnExpression as AST.ResolveExpression
      const valueType = generateConstraintsForExpression(
        ctx,
        resolveExpr.value,
        promiseEnv
      )
      resolveType = valueType
    }
  }

  // Promise<T>型を返す（() -> Promise<T>のラッパー型として）
  const promiseType = new AST.GenericType(
    "Promise",
    [resolveType],
    promiseBlock.line,
    promiseBlock.column
  )

  // 関数型でラップ（() -> Promise<T>）
  const unitType = new AST.PrimitiveType(
    "Unit",
    promiseBlock.line,
    promiseBlock.column
  )
  return new AST.FunctionType(
    unitType,
    promiseType,
    promiseBlock.line,
    promiseBlock.column
  )
}

/**
 * TryExpression: try expr
 */
export function generateConstraintsForTryExpression(
  ctx: InferenceContext,
  tryExpr: AST.TryExpression,
  env: Map<string, AST.Type>
): AST.Type {
  const innerType = generateConstraintsForExpression(ctx, tryExpr.expression, env)

  // エラー型を決定
  let errorType: AST.Type
  if (tryExpr.errorType) {
    errorType = tryExpr.errorType
  } else {
    // デフォルトはString型
    errorType = new AST.PrimitiveType("String", tryExpr.line, tryExpr.column)
  }

  // Promise型かどうかチェック
  const isPromiseType = checkIsPromiseType(innerType)

  if (isPromiseType) {
    // Promise関数型 (Unit -> Promise<T>) -> Unit -> Promise<Either<L, T>>
    let valueType = innerType

    // Function型の場合、戻り値型(Promise<T>)からT部分を取得
    if (innerType.kind === "FunctionType") {
      const funcType = innerType as AST.FunctionType
      const returnType = funcType.returnType
      if (
        returnType.kind === "GenericType" &&
        (returnType as AST.GenericType).name === "Promise" &&
        (returnType as AST.GenericType).typeArguments.length > 0
      ) {
        valueType = (returnType as AST.GenericType).typeArguments[0]!
      }
    }
    // 直接Promise<T>の場合、T部分を取得
    else if (
      innerType.kind === "GenericType" &&
      (innerType as AST.GenericType).name === "Promise" &&
      (innerType as AST.GenericType).typeArguments.length > 0
    ) {
      valueType = (innerType as AST.GenericType).typeArguments[0]!
    }

    // Either<L, T>を構築
    const eitherType = new AST.GenericType(
      "Either",
      [errorType, valueType],
      tryExpr.line,
      tryExpr.column
    )

    // Promise<Either<L, T>>を構築
    const promiseEitherType = new AST.GenericType(
      "Promise",
      [eitherType],
      tryExpr.line,
      tryExpr.column
    )

    // Unit -> Promise<Either<L, T>>を構築
    return new AST.FunctionType(
      new AST.PrimitiveType("Unit", tryExpr.line, tryExpr.column),
      promiseEitherType,
      tryExpr.line,
      tryExpr.column
    )
  } else {
    // T -> Unit -> Either<L, T>
    const eitherType = new AST.GenericType(
      "Either",
      [errorType, innerType],
      tryExpr.line,
      tryExpr.column
    )

    // Unit -> Either<L, T>を構築
    return new AST.FunctionType(
      new AST.PrimitiveType("Unit", tryExpr.line, tryExpr.column),
      eitherType,
      tryExpr.line,
      tryExpr.column
    )
  }
}

/**
 * Promise型かどうかをチェック
 */
function checkIsPromiseType(type: AST.Type): boolean {
  if (type.kind === "GenericType") {
    const gt = type as AST.GenericType
    return gt.name === "Promise"
  }
  if (type.kind === "FunctionType") {
    const ft = type as AST.FunctionType
    return checkIsPromiseType(ft.returnType)
  }
  return false
}

/**
 * ResolveExpression: resolve value
 */
export function generateConstraintsForResolveExpression(
  ctx: InferenceContext,
  resolveExpr: AST.ResolveExpression,
  env: Map<string, AST.Type>
): AST.Type {
  const valueType = generateConstraintsForExpression(ctx, resolveExpr.value, env)

  // resolve式はPromise<T>を返す関数を返す
  const unitType = new AST.PrimitiveType(
    "Unit",
    resolveExpr.line,
    resolveExpr.column
  )
  const promiseType = new AST.GenericType(
    "Promise",
    [valueType],
    resolveExpr.line,
    resolveExpr.column
  )

  return new AST.FunctionType(
    unitType,
    promiseType,
    resolveExpr.line,
    resolveExpr.column
  )
}

/**
 * RejectExpression: reject message
 */
export function generateConstraintsForRejectExpression(
  ctx: InferenceContext,
  rejectExpr: AST.RejectExpression,
  env: Map<string, AST.Type>
): AST.Type {
  const valueType = generateConstraintsForExpression(ctx, rejectExpr.value, env)

  // rejectの引数はStringであることを制約
  addConstraint(
    ctx,
    new TypeConstraint(
      valueType,
      new AST.PrimitiveType("String", rejectExpr.line, rejectExpr.column),
      rejectExpr.line,
      rejectExpr.column
    )
  )

  // reject式はPromise<T>を返す関数を返す（Tは任意）
  const unitType = new AST.PrimitiveType(
    "Unit",
    rejectExpr.line,
    rejectExpr.column
  )
  const promiseType = new AST.GenericType(
    "Promise",
    [freshTypeVariable(ctx, rejectExpr.line, rejectExpr.column)],
    rejectExpr.line,
    rejectExpr.column
  )

  return new AST.FunctionType(
    unitType,
    promiseType,
    rejectExpr.line,
    rejectExpr.column
  )
}

// Statement dispatcher import (circular dependency handling)
import { generateConstraintsForStatement } from "./statement-dispatcher"
