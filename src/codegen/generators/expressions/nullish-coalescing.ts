/**
 * Nullish Coalescing演算子（??）のコード生成
 */

import type { NullishCoalescingExpression } from "../../../ast"
import { type CodeGenContext, getResolvedType } from "../../context"
import { isEitherType, isMaybeType } from "../../type-utils"
import { generateExpression } from "../dispatcher"

/**
 * Nullish Coalescing演算子をTypeScriptコードに変換
 */
export function generateNullishCoalescing(
  ctx: CodeGenContext,
  expr: NullishCoalescingExpression
): string {
  const left = generateExpression(ctx, expr.left)
  const right = generateExpression(ctx, expr.right)

  const leftType = getResolvedType(ctx, expr.left)

  if (isMaybeType(ctx, leftType)) {
    const rightType = getResolvedType(ctx, expr.right)

    // 右辺もMaybe型の場合: 特別な処理が必要
    if (isMaybeType(ctx, rightType)) {
      // Maybe<T> ?? Maybe<U> の場合、左辺がJustなら左辺の値、そうでなければ右辺の値
      return `(${left}.tag === 'Just' ? ${left}.value : (${right}.tag === 'Just' ? ${right}.value : undefined))`
    }

    // Maybe型の場合: fromMaybe(defaultValue, maybe)
    return `fromMaybe(${right}, ${left})`
  }

  if (isEitherType(ctx, leftType)) {
    const rightType = getResolvedType(ctx, expr.right)

    // 右辺もEither型の場合: 特別な処理が必要
    if (isEitherType(ctx, rightType)) {
      // Either<L, R> ?? Either<L2, R2> の場合、左辺がRightなら左辺の値、そうでなければ右辺の値
      return `(${left}.tag === 'Right' ? ${left}.value : (${right}.tag === 'Right' ? ${right}.value : undefined))`
    }

    // Either型の場合: fromRight(defaultValue, either)
    return `fromRight(${right}, ${left})`
  }

  // その他の場合: TypeScriptのnull合体演算子を使用
  return `(${left} ?? ${right})`
}
