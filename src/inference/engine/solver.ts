/**
 * 制約ソルバー（Constraint Solver）
 *
 * 収集された型制約を解決し、型代入（TypeSubstitution）を生成する
 */

import * as AST from "../../ast"
import {
  ApplicativeApplyConstraint,
  ArrayAccessConstraint,
  FunctorMapConstraint,
  TypeConstraint,
} from "../constraints"
import { TypeInferenceError } from "../errors"
import { TypeSubstitution } from "../substitution"
import { typeToString } from "../type-formatter"
import { TypeVariable } from "../type-variables"
import { addError, freshTypeVariable, type InferenceContext } from "./context"
import { unifyOrThrow } from "./unifier"

/**
 * 制約ソルバーの結果
 */
export interface SolveResult {
  substitution: TypeSubstitution
  errors: TypeInferenceError[]
}

/**
 * すべての制約を解決する
 */
export function solveConstraints(ctx: InferenceContext): TypeSubstitution {
  let substitution = new TypeSubstitution()

  for (const constraint of ctx.constraints) {
    try {
      if (constraint instanceof ArrayAccessConstraint) {
        // ArrayAccessConstraintの特別な処理
        const constraintSub = solveArrayAccessConstraint(
          ctx,
          constraint,
          substitution
        )
        substitution = substitution.compose(constraintSub)
      } else if (constraint instanceof FunctorMapConstraint) {
        // FunctorMapConstraintの特別な処理
        const constraintSub = solveFunctorMapConstraint(
          ctx,
          constraint,
          substitution
        )
        substitution = substitution.compose(constraintSub)
      } else if (constraint instanceof ApplicativeApplyConstraint) {
        // ApplicativeApplyConstraintの特別な処理
        const constraintSub = solveApplicativeApplyConstraint(
          ctx,
          constraint,
          substitution
        )
        substitution = substitution.compose(constraintSub)
      } else {
        // 通常のTypeConstraint処理
        const constraintSub = unifyOrThrow(
          ctx,
          substitution.apply(constraint.type1),
          substitution.apply(constraint.type2)
        )
        substitution = substitution.compose(constraintSub)
      }
    } catch (error) {
      addError(
        ctx,
        `Cannot unify types: ${error}`,
        constraint.line,
        constraint.column,
        constraint.context
      )
    }
  }

  return substitution
}

/**
 * 部分的制約解決：指定した制約のみを解決
 */
export function solveConstraintsPartial(
  ctx: InferenceContext,
  constraintsToSolve: (
    | TypeConstraint
    | ArrayAccessConstraint
    | FunctorMapConstraint
    | ApplicativeApplyConstraint
  )[]
): TypeSubstitution {
  let substitution = new TypeSubstitution()

  for (const constraint of constraintsToSolve) {
    try {
      if (constraint instanceof ArrayAccessConstraint) {
        const constraintSub = solveArrayAccessConstraint(
          ctx,
          constraint,
          substitution
        )
        substitution = substitution.compose(constraintSub)
      } else if (constraint instanceof FunctorMapConstraint) {
        const constraintSub = solveFunctorMapConstraint(
          ctx,
          constraint,
          substitution
        )
        substitution = substitution.compose(constraintSub)
      } else if (constraint instanceof ApplicativeApplyConstraint) {
        const constraintSub = solveApplicativeApplyConstraint(
          ctx,
          constraint,
          substitution
        )
        substitution = substitution.compose(constraintSub)
      } else {
        const constraintSub = unifyOrThrow(
          ctx,
          substitution.apply(constraint.type1),
          substitution.apply(constraint.type2)
        )
        substitution = substitution.compose(constraintSub)
      }
    } catch {
      // 部分解決では一部エラーを無視
    }
  }

  return substitution
}

/**
 * ArrayAccess制約の特別な解決
 * array[index] -> Maybe<elementType>
 */
function solveArrayAccessConstraint(
  ctx: InferenceContext,
  constraint: ArrayAccessConstraint,
  currentSubstitution: TypeSubstitution
): TypeSubstitution {
  const arrayType = currentSubstitution.apply(constraint.arrayType)
  const resultType = constraint.resultType

  // Array<T>の場合
  if (arrayType.kind === "GenericType") {
    const gt = arrayType as AST.GenericType
    if (gt.name === "Array" && gt.typeArguments.length === 1) {
      // Array<T>[index] -> Maybe<T>
      const maybeType = new AST.GenericType(
        "Maybe",
        [gt.typeArguments[0]!],
        constraint.line,
        constraint.column
      )
      return unifyOrThrow(ctx, resultType, maybeType)
    }
    if (gt.name === "List" && gt.typeArguments.length === 1) {
      // List<T>[index] -> Maybe<T>
      const maybeType = new AST.GenericType(
        "Maybe",
        [gt.typeArguments[0]!],
        constraint.line,
        constraint.column
      )
      return unifyOrThrow(ctx, resultType, maybeType)
    }
  }

  // Tuple型の場合
  if (arrayType.kind === "TupleType") {
    const tt = arrayType as AST.TupleType
    if (tt.elementTypes.length > 0) {
      // タプルアクセスの場合、結果型を任意の型変数とする
      const unionType = freshTypeVariable(ctx, constraint.line, constraint.column)
      return unifyOrThrow(ctx, resultType, unionType)
    }
  }

  // 型変数の場合、Array<T>として推論
  if (arrayType.kind === "TypeVariable") {
    const elementType = freshTypeVariable(ctx, constraint.line, constraint.column)

    // Array<T>として推論
    const arrayGenericType = new AST.GenericType(
      "Array",
      [elementType],
      constraint.line,
      constraint.column
    )

    // 結果はMaybe<T>
    const maybeType = new AST.GenericType(
      "Maybe",
      [elementType],
      constraint.line,
      constraint.column
    )

    const sub1 = unifyOrThrow(ctx, arrayType, arrayGenericType)
    const sub2 = unifyOrThrow(ctx, resultType, maybeType)
    return sub1.compose(sub2)
  }

  throw new Error(
    `Array access requires Array<T>, List<T> or Tuple type, got ${typeToString(arrayType)}`
  )
}

/**
 * FunctorMap制約の特別な解決
 * container.fmap(f) -> container'
 */
function solveFunctorMapConstraint(
  ctx: InferenceContext,
  constraint: FunctorMapConstraint,
  currentSubstitution: TypeSubstitution
): TypeSubstitution {
  const containerType = currentSubstitution.apply(constraint.containerType)
  const inputType = currentSubstitution.apply(constraint.inputType)
  const outputType = currentSubstitution.apply(constraint.outputType)
  const resultType = constraint.resultType

  if (containerType.kind === "GenericType") {
    const gt = containerType as AST.GenericType

    if (gt.name === "Maybe" && gt.typeArguments.length === 1) {
      // Maybe<a> case
      const sub1 = unifyOrThrow(ctx, gt.typeArguments[0]!, inputType)
      const maybeResultType = new AST.GenericType(
        "Maybe",
        [outputType],
        constraint.line,
        constraint.column
      )
      const sub2 = unifyOrThrow(ctx, resultType, maybeResultType)
      return sub1.compose(sub2)
    } else if (gt.name === "Either" && gt.typeArguments.length === 2) {
      // Either<e, a> case
      const errorType = gt.typeArguments[0]!
      const sub1 = unifyOrThrow(ctx, gt.typeArguments[1]!, inputType)
      const eitherResultType = new AST.GenericType(
        "Either",
        [errorType, outputType],
        constraint.line,
        constraint.column
      )
      const sub2 = unifyOrThrow(ctx, resultType, eitherResultType)
      return sub1.compose(sub2)
    } else if (gt.name === "List" && gt.typeArguments.length === 1) {
      // List<a> case
      const sub1 = unifyOrThrow(ctx, gt.typeArguments[0]!, inputType)
      const listResultType = new AST.GenericType(
        "List",
        [outputType],
        constraint.line,
        constraint.column
      )
      const sub2 = unifyOrThrow(ctx, resultType, listResultType)
      return sub1.compose(sub2)
    } else if (gt.name === "Task" && gt.typeArguments.length === 1) {
      // Task<a> case
      const sub1 = unifyOrThrow(ctx, gt.typeArguments[0]!, inputType)
      const taskResultType = new AST.GenericType(
        "Task",
        [outputType],
        constraint.line,
        constraint.column
      )
      const sub2 = unifyOrThrow(ctx, resultType, taskResultType)
      return sub1.compose(sub2)
    } else if (gt.typeArguments.length > 0) {
      // Generic functor case
      const sub1 = unifyOrThrow(
        ctx,
        gt.typeArguments[gt.typeArguments.length - 1]!,
        inputType
      )
      const newArgs = [...gt.typeArguments]
      newArgs[newArgs.length - 1] = outputType
      const genericResultType = new AST.GenericType(
        gt.name,
        newArgs,
        constraint.line,
        constraint.column
      )
      const sub2 = unifyOrThrow(ctx, resultType, genericResultType)
      return sub1.compose(sub2)
    }
  }

  // まだ解決されていない型変数の場合
  if (containerType.kind === "TypeVariable") {
    return new TypeSubstitution()
  }

  throw new Error(
    `FunctorMap requires a Functor type (Maybe, Either, List, Task, etc.), got ${typeToString(containerType)}`
  )
}

/**
 * ApplicativeApply制約の特別な解決
 * funcContainer <*> valueContainer -> resultContainer
 */
function solveApplicativeApplyConstraint(
  ctx: InferenceContext,
  constraint: ApplicativeApplyConstraint,
  currentSubstitution: TypeSubstitution
): TypeSubstitution {
  const funcContainerType = currentSubstitution.apply(constraint.funcContainerType)
  const valueContainerType = currentSubstitution.apply(constraint.valueContainerType)
  const inputType = currentSubstitution.apply(constraint.inputType)
  const outputType = currentSubstitution.apply(constraint.outputType)
  const resultType = constraint.resultType

  if (funcContainerType.kind === "GenericType") {
    const funcGt = funcContainerType as AST.GenericType

    if (funcGt.name === "Maybe" && funcGt.typeArguments.length === 1) {
      // Maybe<(a -> b)> <*> Maybe<a> -> Maybe<b>
      const funcType = new AST.FunctionType(
        inputType,
        outputType,
        constraint.line,
        constraint.column
      )
      const sub1 = unifyOrThrow(ctx, funcGt.typeArguments[0]!, funcType)

      // valueContainerTypeをMaybe<inputType>に統一
      const expectedValueType = new AST.GenericType(
        "Maybe",
        [inputType],
        constraint.line,
        constraint.column
      )
      const sub2 = unifyOrThrow(ctx, valueContainerType, expectedValueType)

      // 結果をMaybe<outputType>に統一
      const maybeResultType = new AST.GenericType(
        "Maybe",
        [outputType],
        constraint.line,
        constraint.column
      )
      const sub3 = unifyOrThrow(ctx, resultType, maybeResultType)
      return sub1.compose(sub2).compose(sub3)
    } else if (funcGt.name === "Either" && funcGt.typeArguments.length === 2) {
      // Either<e, (a -> b)> <*> Either<e, a> -> Either<e, b>
      const errorType = funcGt.typeArguments[0]!
      const funcType = new AST.FunctionType(
        inputType,
        outputType,
        constraint.line,
        constraint.column
      )
      const sub1 = unifyOrThrow(ctx, funcGt.typeArguments[1]!, funcType)

      // valueContainerTypeをEither<e, inputType>に統一
      const expectedValueType = new AST.GenericType(
        "Either",
        [errorType, inputType],
        constraint.line,
        constraint.column
      )
      const sub2 = unifyOrThrow(ctx, valueContainerType, expectedValueType)

      // 結果をEither<e, outputType>に統一
      const eitherResultType = new AST.GenericType(
        "Either",
        [errorType, outputType],
        constraint.line,
        constraint.column
      )
      const sub3 = unifyOrThrow(ctx, resultType, eitherResultType)
      return sub1.compose(sub2).compose(sub3)
    } else if (funcGt.name === "List" && funcGt.typeArguments.length === 1) {
      // List<(a -> b)> <*> List<a> -> List<b>
      const funcType = new AST.FunctionType(
        inputType,
        outputType,
        constraint.line,
        constraint.column
      )
      const sub1 = unifyOrThrow(ctx, funcGt.typeArguments[0]!, funcType)

      // valueContainerTypeをList<inputType>に統一
      const expectedValueType = new AST.GenericType(
        "List",
        [inputType],
        constraint.line,
        constraint.column
      )
      const sub2 = unifyOrThrow(ctx, valueContainerType, expectedValueType)

      // 結果をList<outputType>に統一
      const listResultType = new AST.GenericType(
        "List",
        [outputType],
        constraint.line,
        constraint.column
      )
      const sub3 = unifyOrThrow(ctx, resultType, listResultType)
      return sub1.compose(sub2).compose(sub3)
    } else if (funcGt.name === "Task" && funcGt.typeArguments.length === 1) {
      // Task<(a -> b)> <*> Task<a> -> Task<b>
      const funcType = new AST.FunctionType(
        inputType,
        outputType,
        constraint.line,
        constraint.column
      )
      const sub1 = unifyOrThrow(ctx, funcGt.typeArguments[0]!, funcType)

      // valueContainerTypeをTask<inputType>に統一
      const expectedValueType = new AST.GenericType(
        "Task",
        [inputType],
        constraint.line,
        constraint.column
      )
      const sub2 = unifyOrThrow(ctx, valueContainerType, expectedValueType)

      // 結果をTask<outputType>に統一
      const taskResultType = new AST.GenericType(
        "Task",
        [outputType],
        constraint.line,
        constraint.column
      )
      const sub3 = unifyOrThrow(ctx, resultType, taskResultType)
      return sub1.compose(sub2).compose(sub3)
    } else if (funcGt.typeArguments.length > 0) {
      // Generic applicative case
      const funcType = new AST.FunctionType(
        inputType,
        outputType,
        constraint.line,
        constraint.column
      )
      const sub1 = unifyOrThrow(
        ctx,
        funcGt.typeArguments[funcGt.typeArguments.length - 1]!,
        funcType
      )

      // valueContainerTypeを同じコンテナ型に統一
      const expectedValueArgs = [...funcGt.typeArguments]
      expectedValueArgs[expectedValueArgs.length - 1] = inputType
      const expectedValueType = new AST.GenericType(
        funcGt.name,
        expectedValueArgs,
        constraint.line,
        constraint.column
      )
      const sub2 = unifyOrThrow(ctx, valueContainerType, expectedValueType)

      // 結果を同じコンテナ型に統一
      const resultArgs = [...funcGt.typeArguments]
      resultArgs[resultArgs.length - 1] = outputType
      const genericResultType = new AST.GenericType(
        funcGt.name,
        resultArgs,
        constraint.line,
        constraint.column
      )
      const sub3 = unifyOrThrow(ctx, resultType, genericResultType)
      return sub1.compose(sub2).compose(sub3)
    }
  }

  // まだ解決されていない型変数の場合
  if (funcContainerType.kind === "TypeVariable") {
    return new TypeSubstitution()
  }

  throw new Error(
    `ApplicativeApply requires an Applicative type (Maybe, Either, List, Task, etc.), got ${typeToString(funcContainerType)}`
  )
}
