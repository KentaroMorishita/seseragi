/**
 * 関数呼び出し（複数引数）の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import { typeToString as typeToStringUtil } from "../../type-formatter"
import {
  addConstraint,
  addError,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"
import {
  collectPolymorphicTypeVariables,
  instantiatePolymorphicType,
  substituteTypeVariables,
} from "./helpers"

/**
 * 明示的型引数を用いて多相型をインスタンス化
 */
function instantiateWithExplicitTypeArguments(
  ctx: InferenceContext,
  type: AST.Type,
  typeArguments: AST.Type[],
  line: number,
  column: number
): AST.Type {
  // 多相型変数を収集
  const polymorphicVars = collectPolymorphicTypeVariables(type)

  // 型引数の数が一致しない場合はエラー
  if (polymorphicVars.length !== typeArguments.length) {
    addError(
      ctx,
      `Type argument count mismatch. Expected ${polymorphicVars.length} but got ${typeArguments.length}`,
      line,
      column
    )
    return type
  }

  // 多相型変数を明示的型引数で置換するマップを作成
  const substitutionMap = new Map<string, AST.Type>()
  for (let i = 0; i < polymorphicVars.length; i++) {
    const varName = polymorphicVars[i]
    const typeArg = typeArguments[i]
    if (varName && typeArg) {
      substitutionMap.set(varName, typeArg)
    }
  }

  return substituteTypeVariables(type, substitutionMap)
}

/**
 * 関数呼び出しの制約を生成
 * func(arg1, arg2, ...) のような式を処理
 */
export function generateConstraintsForFunctionCall(
  ctx: InferenceContext,
  call: AST.FunctionCall,
  env: Map<string, AST.Type>
): AST.Type {
  // print/putStrLn/show関数の特別処理
  if (call.function.kind === "Identifier") {
    const funcName = (call.function as AST.Identifier).name
    if (
      (funcName === "print" ||
        funcName === "putStrLn" ||
        funcName === "show") &&
      call.arguments.length === 1
    ) {
      const firstArg = call.arguments[0]
      if (firstArg) {
        generateConstraintsForExpression(ctx, firstArg, env)
      }
      return new AST.PrimitiveType("Unit", call.line, call.column)
    }

    // tryRun関数の特別処理
    if (
      (funcName === "tryRun" || funcName === "ssrgTryRun") &&
      call.arguments.length === 1
    ) {
      const firstArg = call.arguments[0]
      if (!firstArg) {
        return freshTypeVariable(ctx, call.line, call.column)
      }

      const argType = generateConstraintsForExpression(ctx, firstArg, env)

      // エラー型を決定（型引数があれば使用、なければString）
      let errorType: AST.Type
      if (call.typeArguments && call.typeArguments.length > 0) {
        const firstTypeArg = call.typeArguments[0]
        errorType = firstTypeArg || new AST.PrimitiveType("String", call.line, call.column)
      } else {
        errorType = new AST.PrimitiveType("String", call.line, call.column)
      }

      // Task<T> から T を取得
      let valueType: AST.Type
      if (
        argType.kind === "GenericType" &&
        (argType as AST.GenericType).name === "Task"
      ) {
        const taskType = argType as AST.GenericType
        const firstTypeArg = taskType.typeArguments[0]
        valueType = firstTypeArg || freshTypeVariable(ctx, call.line, call.column)
      } else {
        valueType = freshTypeVariable(ctx, call.line, call.column)
      }

      // Promise<Either<ErrorType, T>> を返す
      const eitherType = new AST.GenericType(
        "Either",
        [errorType, valueType],
        call.line,
        call.column
      )
      return new AST.GenericType(
        "Promise",
        [eitherType],
        call.line,
        call.column
      )
    }
  }

  // 明示的型引数がある場合と無い場合で処理を分ける
  let funcType: AST.Type
  let resultType: AST.Type

  if (call.typeArguments && call.typeArguments.length > 0) {
    // 明示的型引数がある場合は、環境から直接多相型を取得
    if (call.function.kind === "Identifier") {
      const identifier = call.function as AST.Identifier
      const rawFuncType = env.get(identifier.name)
      if (!rawFuncType) {
        addError(ctx, `Undefined function: ${identifier.name}`, call.line, call.column)
        return freshTypeVariable(ctx, call.line, call.column)
      }

      resultType = instantiateWithExplicitTypeArguments(
        ctx,
        rawFuncType,
        call.typeArguments,
        call.line,
        call.column
      )
      funcType = resultType
    } else {
      // 複雑な式の場合
      funcType = generateConstraintsForExpression(ctx, call.function, env)
      resultType = instantiateWithExplicitTypeArguments(
        ctx,
        funcType,
        call.typeArguments,
        call.line,
        call.column
      )
    }
  } else {
    // 従来の処理
    if (call.function.kind === "Identifier") {
      const identifier = call.function as AST.Identifier
      const rawFuncType = env.get(identifier.name)
      if (rawFuncType) {
        resultType = instantiatePolymorphicType(ctx, rawFuncType, call.line, call.column)
        funcType = resultType
      } else {
        funcType = generateConstraintsForExpression(ctx, call.function, env)
        resultType = instantiatePolymorphicType(ctx, funcType, call.line, call.column)
      }
    } else {
      funcType = generateConstraintsForExpression(ctx, call.function, env)
      resultType = instantiatePolymorphicType(ctx, funcType, call.line, call.column)
    }

    // 引数が0個の場合は、関数がユニット型を取る関数として扱う
    if (call.arguments.length === 0) {
      if (funcType.kind === "FunctionType") {
        const ft = funcType as AST.FunctionType
        const expectedFuncType = new AST.FunctionType(
          new AST.PrimitiveType("Unit", call.line, call.column),
          ft.returnType,
          call.line,
          call.column
        )

        addConstraint(
          ctx,
          new TypeConstraint(
            funcType,
            expectedFuncType,
            call.line,
            call.column,
            `Unit function application`
          )
        )

        return ft.returnType
      }

      // 関数シグネチャが不明な場合のフォールバック
      const result = freshTypeVariable(ctx, call.line, call.column)
      const expectedFuncType = new AST.FunctionType(
        new AST.PrimitiveType("Unit", call.line, call.column),
        result,
        call.line,
        call.column
      )

      addConstraint(
        ctx,
        new TypeConstraint(
          funcType,
          expectedFuncType,
          call.line,
          call.column,
          `Unit function application`
        )
      )

      return result
    }
  }

  // 各引数に対して関数適用の制約を生成
  for (const arg of call.arguments) {
    let expectedParamType: AST.Type = freshTypeVariable(ctx, call.line, call.column)

    // 関数型が既知の場合、パラメータ型を抽出
    if (resultType.kind === "FunctionType") {
      const funcTypeInstance = resultType as AST.FunctionType
      expectedParamType = funcTypeInstance.paramType
    }

    const actualArgType = generateConstraintsForExpression(
      ctx,
      arg,
      env,
      expectedParamType
    )
    const newResultType = freshTypeVariable(ctx, call.line, call.column)

    // 現在の結果型は expectedParamType から newResultType へのFunction型でなければならない
    const expectedFuncType = new AST.FunctionType(
      expectedParamType,
      newResultType,
      call.line,
      call.column
    )

    addConstraint(
      ctx,
      new TypeConstraint(
        resultType,
        expectedFuncType,
        call.line,
        call.column,
        `Function application structure`
      )
    )

    addConstraint(
      ctx,
      new TypeConstraint(
        actualArgType,
        expectedParamType,
        call.line,
        call.column,
        `Function parameter type: ${typeToStringUtil(actualArgType)} ~ ${typeToStringUtil(expectedParamType)}`
      )
    )

    resultType = newResultType
  }

  return resultType
}
