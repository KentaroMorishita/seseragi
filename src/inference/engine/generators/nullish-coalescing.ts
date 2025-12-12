/**
 * Nullish合体演算子（??）の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import {
  isEitherType as isEitherTypeUtil,
  isMaybeType as isMaybeTypeUtil,
} from "../../type-inspection"
import {
  addConstraint,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * NullishCoalescing演算子の制約を生成
 * Maybe<T> ?? U => T（MaybeがSomeならその値、NoneならU）
 * Either<L, R> ?? U => R（EitherがRightならその値、LeftならU）
 */
export function generateConstraintsForNullishCoalescing(
  ctx: InferenceContext,
  nullishCoalescing: AST.NullishCoalescingExpression,
  env: Map<string, AST.Type>
): AST.Type {
  const leftType = generateConstraintsForExpression(
    ctx,
    nullishCoalescing.left,
    env
  )
  const rightType = generateConstraintsForExpression(
    ctx,
    nullishCoalescing.right,
    env
  )

  // 型変数の場合
  if (leftType.kind === "TypeVariable") {
    if (rightType.kind === "TypeVariable") {
      // 両方型変数
      const resultTypeVar = freshTypeVariable(
        ctx,
        nullishCoalescing.line,
        nullishCoalescing.column
      )

      const maybeType = new AST.GenericType(
        "Maybe",
        [resultTypeVar],
        nullishCoalescing.line,
        nullishCoalescing.column
      )

      addConstraint(
        ctx,
        new TypeConstraint(
          leftType,
          maybeType,
          nullishCoalescing.line,
          nullishCoalescing.column,
          "Nullish coalescing left operand should be Maybe type"
        )
      )

      addConstraint(
        ctx,
        new TypeConstraint(
          rightType,
          resultTypeVar,
          nullishCoalescing.line,
          nullishCoalescing.column,
          "Nullish coalescing right operand should be compatible with result"
        )
      )

      return resultTypeVar
    }

    // 右辺がMaybe型の場合
    if (isMaybeTypeUtil(rightType)) {
      const rightMaybeInnerType = (rightType as AST.GenericType)
        .typeArguments[0]
      const innerTypeVar = freshTypeVariable(
        ctx,
        nullishCoalescing.line,
        nullishCoalescing.column
      )
      const maybeType = new AST.GenericType(
        "Maybe",
        [innerTypeVar],
        nullishCoalescing.line,
        nullishCoalescing.column
      )

      addConstraint(
        ctx,
        new TypeConstraint(
          leftType,
          maybeType,
          nullishCoalescing.line,
          nullishCoalescing.column,
          "Nullish coalescing left operand should be Maybe type"
        )
      )

      addConstraint(
        ctx,
        new TypeConstraint(
          innerTypeVar,
          rightMaybeInnerType,
          nullishCoalescing.line,
          nullishCoalescing.column,
          "Maybe inner types should be compatible"
        )
      )

      return innerTypeVar
    }

    // 右辺が非Maybe型
    const innerTypeVar = freshTypeVariable(
      ctx,
      nullishCoalescing.line,
      nullishCoalescing.column
    )
    const maybeType = new AST.GenericType(
      "Maybe",
      [innerTypeVar],
      nullishCoalescing.line,
      nullishCoalescing.column
    )

    addConstraint(
      ctx,
      new TypeConstraint(
        leftType,
        maybeType,
        nullishCoalescing.line,
        nullishCoalescing.column,
        "Nullish coalescing left operand should be Maybe type"
      )
    )

    addConstraint(
      ctx,
      new TypeConstraint(
        innerTypeVar,
        rightType,
        nullishCoalescing.line,
        nullishCoalescing.column,
        "Maybe inner type should be compatible with default value"
      )
    )

    return innerTypeVar
  }

  // Maybe型の場合
  if (isMaybeTypeUtil(leftType)) {
    const maybeInnerType = (leftType as AST.GenericType).typeArguments[0]

    if (isMaybeTypeUtil(rightType)) {
      const rightMaybeInnerType = (rightType as AST.GenericType)
        .typeArguments[0]
      addConstraint(
        ctx,
        new TypeConstraint(
          maybeInnerType,
          rightMaybeInnerType,
          nullishCoalescing.line,
          nullishCoalescing.column,
          "Maybe inner types should be compatible"
        )
      )
      return maybeInnerType
    }

    addConstraint(
      ctx,
      new TypeConstraint(
        maybeInnerType,
        rightType,
        nullishCoalescing.line,
        nullishCoalescing.column,
        "Maybe inner type should be compatible with default value"
      )
    )
    return maybeInnerType
  }

  // Either型の場合
  if (isEitherTypeUtil(leftType)) {
    const rightTypeFromEither = (leftType as AST.GenericType).typeArguments[1]

    if (
      rightType.kind === "GenericType" &&
      (rightType as AST.GenericType).name === "Either"
    ) {
      const rightEitherRightType = (rightType as AST.GenericType)
        .typeArguments[1]
      addConstraint(
        ctx,
        new TypeConstraint(
          rightTypeFromEither,
          rightEitherRightType,
          nullishCoalescing.line,
          nullishCoalescing.column,
          "Either Right types should be compatible"
        )
      )
      return rightTypeFromEither
    }

    addConstraint(
      ctx,
      new TypeConstraint(
        rightTypeFromEither,
        rightType,
        nullishCoalescing.line,
        nullishCoalescing.column,
        "Either Right type should be compatible with default value"
      )
    )
    return rightTypeFromEither
  }

  // その他（TypeScript風null合体）
  addConstraint(
    ctx,
    new TypeConstraint(
      leftType,
      rightType,
      nullishCoalescing.line,
      nullishCoalescing.column,
      "?? operands should have compatible types for non-Maybe/Either types"
    )
  )

  return rightType
}
