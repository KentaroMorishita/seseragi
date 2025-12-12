/**
 * パイプライン演算子のコード生成
 */

import type {
  ApplicativeApply,
  FoldMonoid,
  FunctionApplicationOperator,
  FunctorMap,
  Identifier,
  MonadBind,
  Pipeline,
  ReversePipe,
  Type,
} from "../../../ast"
import { LambdaExpression } from "../../../ast"
import { type CodeGenContext, getResolvedType } from "../../context"
import {
  isArrayType,
  isEitherType,
  isListType,
  isMaybeType,
  isSignalType,
  isTaskType,
  isTupleType,
} from "../../type-utils"
import { generateExpression } from "../dispatcher"

/**
 * パイプライン演算子（|>）をTypeScriptコードに変換
 */
export function generatePipeline(
  ctx: CodeGenContext,
  pipeline: Pipeline
): string {
  const left = generateExpression(ctx, pipeline.left)
  const right = generateExpression(ctx, pipeline.right)

  return `pipe(${left}, ${right})`
}

/**
 * 逆パイプ演算子（<|）をTypeScriptコードに変換
 */
export function generateReversePipe(
  ctx: CodeGenContext,
  reversePipe: ReversePipe
): string {
  // ビルトイン関数の場合は直接変換
  if (reversePipe.left.kind === "Identifier") {
    const identifier = reversePipe.left as Identifier
    const right = generateExpression(ctx, reversePipe.right)

    switch (identifier.name) {
      case "print":
        return `ssrgPrint(${right})`
      case "putStrLn":
        return `ssrgPutStrLn(${right})`
      case "toString":
        return `ssrgToString(${right})`
      case "toInt":
        return `ssrgToInt(${right})`
      case "toFloat":
        return `ssrgToFloat(${right})`
      case "show":
        return `ssrgShow(${right})`
      case "head":
        return `headList(${right})`
      case "tail":
        return `tailList(${right})`
    }
  }

  const left = generateExpression(ctx, reversePipe.left)
  const right = generateExpression(ctx, reversePipe.right)

  return `reversePipe(${left}, ${right})`
}

/**
 * ファンクターマップ（<$>）をTypeScriptコードに変換
 */
export function generateFunctorMap(
  ctx: CodeGenContext,
  map: FunctorMap
): string {
  const func = generateExpression(ctx, map.left)
  const value = generateExpression(ctx, map.right)
  const valueType = getResolvedType(ctx, map.right)

  // 型変数の場合はエラー
  if (valueType?.kind === "TypeVariable") {
    throw new Error(
      `Unknown type for functor map: Type variable not resolved - ${JSON.stringify(valueType)}`
    )
  }

  if (isSignalType(ctx, valueType)) return `mapSignal(${value}, ${func})`
  if (isTaskType(ctx, valueType)) return `mapTask(${func}, ${value})`
  if (isArrayType(ctx, valueType)) return `mapArray(${value}, ${func})`
  if (isListType(ctx, valueType)) return `mapList(${value}, ${func})`
  if (isEitherType(ctx, valueType)) return `mapEither(${value}, ${func})`
  if (isMaybeType(ctx, valueType)) return `mapMaybe(${value}, ${func})`
  if (isTupleType(ctx, valueType)) {
    return `{ tag: 'Tuple', elements: mapArray((${value}).elements, ${func}) }`
  }

  throw new Error(`Unknown type for functor map: ${JSON.stringify(valueType)}`)
}

/**
 * アプリカティブ適用（<*>）をTypeScriptコードに変換
 */
export function generateApplicativeApply(
  ctx: CodeGenContext,
  apply: ApplicativeApply
): string {
  const funcContainer = generateExpression(ctx, apply.left)
  const valueContainer = generateExpression(ctx, apply.right)
  const leftType = getResolvedType(ctx, apply.left)
  const rightType = getResolvedType(ctx, apply.right)

  const isTypeVariableLeft = leftType?.kind === "TypeVariable"
  const isTypeVariableRight = rightType?.kind === "TypeVariable"

  const typeAppliers: Array<
    [(ctx: CodeGenContext, type: Type | undefined) => boolean, string]
  > = [
    [isSignalType, "applySignal"],
    [isTaskType, "applyTask"],
    [isArrayType, "applyArray"],
    [isListType, "applyList"],
    [isEitherType, "applyEither"],
    [isMaybeType, "applyMaybe"],
  ]

  for (const [checker, applyFunc] of typeAppliers) {
    if (
      (leftType &&
        rightType &&
        checker(ctx, leftType) &&
        checker(ctx, rightType)) ||
      (isTypeVariableLeft && rightType && checker(ctx, rightType)) ||
      (leftType && isTypeVariableRight && checker(ctx, leftType))
    ) {
      return `${applyFunc}(${funcContainer}, ${valueContainer})`
    }
  }

  throw new Error(
    `Unknown types for applicative apply: left=${JSON.stringify(leftType)}, right=${JSON.stringify(rightType)}`
  )
}

/**
 * モナドバインド（>>=）をTypeScriptコードに変換
 */
export function generateMonadBind(
  ctx: CodeGenContext,
  bind: MonadBind
): string {
  const monadValue = generateExpression(ctx, bind.left)
  const bindFunc = generateExpression(ctx, bind.right)
  const monadType = getResolvedType(ctx, bind.left)

  // 型変数の場合はエラー
  if (monadType?.kind === "TypeVariable") {
    throw new Error(
      `Unknown type for monad bind: Type variable not resolved - ${JSON.stringify(monadType)}`
    )
  }

  if (isTaskType(ctx, monadType)) return `bindTask(${monadValue}, ${bindFunc})`
  if (isArrayType(ctx, monadType))
    return `bindArray(${monadValue}, ${bindFunc})`
  if (isListType(ctx, monadType)) return `bindList(${monadValue}, ${bindFunc})`
  if (isEitherType(ctx, monadType))
    return `bindEither(${monadValue}, ${bindFunc})`
  if (isMaybeType(ctx, monadType))
    return `bindMaybe(${monadValue}, ${bindFunc})`
  if (isSignalType(ctx, monadType))
    return `bindSignal(${monadValue}, ${bindFunc})`
  if (isTupleType(ctx, monadType)) {
    return `{ tag: 'Tuple', elements: bindArray((${monadValue}).elements, ${bindFunc}) }`
  }

  throw new Error(`Unknown type for monad bind: ${JSON.stringify(monadType)}`)
}

/**
 * 畳み込みモノイド（<>）をTypeScriptコードに変換
 */
export function generateFoldMonoid(
  ctx: CodeGenContext,
  fold: FoldMonoid
): string {
  const left = generateExpression(ctx, fold.left)
  const right = generateExpression(ctx, fold.right)

  return `foldMonoid(${left}, /* empty */, ${right})`
}

/**
 * 関数適用演算子（$）をTypeScriptコードに変換
 */
export function generateFunctionApplicationOperator(
  ctx: CodeGenContext,
  app: FunctionApplicationOperator
): string {
  let left = generateExpression(ctx, app.left)
  const right = generateExpression(ctx, app.right)

  // 左辺がラムダ式の場合、適用のために括弧で包む
  if (app.left instanceof LambdaExpression) {
    left = `(${left})`
  }

  // ビルトイン関数の特別処理
  if (app.left.kind === "Identifier") {
    const identifier = app.left as Identifier
    const builtinFunctions: Record<string, string> = {
      print: "ssrgPrint",
      putStrLn: "ssrgPutStrLn",
      toString: "ssrgToString",
      toInt: "ssrgToInt",
      toFloat: "ssrgToFloat",
      show: "ssrgShow",
      head: "headList",
      tail: "tailList",
    }

    const builtinFunc = builtinFunctions[identifier.name]
    if (builtinFunc) {
      return `${builtinFunc}(${right})`
    }
  }

  return `${left}(${right})`
}
