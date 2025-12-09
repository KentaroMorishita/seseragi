/**
 * 組み込み関数呼び出しの制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import {
  addConstraint,
  addError,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * 組み込み関数呼び出しの制約を生成
 */
export function generateConstraintsForBuiltinFunctionCall(
  ctx: InferenceContext,
  call: AST.BuiltinFunctionCall,
  env: Map<string, AST.Type>
): AST.Type {
  switch (call.functionName) {
    case "print":
    case "putStrLn":
    case "show": {
      // Type: 'a -> Unit (polymorphic)
      const firstArg = call.arguments[0]
      if (call.arguments.length === 1 && firstArg) {
        generateConstraintsForExpression(ctx, firstArg, env)
      }
      return new AST.PrimitiveType("Unit", call.line, call.column)
    }

    case "toString": {
      // Type: 'a -> String (polymorphic)
      const firstArg = call.arguments[0]
      if (call.arguments.length === 1 && firstArg) {
        generateConstraintsForExpression(ctx, firstArg, env)
      }
      return new AST.PrimitiveType("String", call.line, call.column)
    }

    case "head": {
      // Type: List<T> -> Maybe<T>
      const firstArg = call.arguments[0]
      if (call.arguments.length === 1 && firstArg) {
        const listType = generateConstraintsForExpression(ctx, firstArg, env)
        const elementType = freshTypeVariable(ctx, call.line, call.column)

        const expectedListType = new AST.GenericType(
          "List",
          [elementType],
          call.line,
          call.column
        )
        addConstraint(
          ctx,
          new TypeConstraint(
            listType,
            expectedListType,
            call.line,
            call.column,
            "head function requires List<T> argument"
          )
        )

        return new AST.GenericType(
          "Maybe",
          [elementType],
          call.line,
          call.column
        )
      }
      addError(ctx, "head function requires exactly one argument", call.line, call.column)
      return freshTypeVariable(ctx, call.line, call.column)
    }

    case "tail": {
      // Type: List<T> -> List<T>
      const firstArg = call.arguments[0]
      if (call.arguments.length === 1 && firstArg) {
        const listType = generateConstraintsForExpression(ctx, firstArg, env)
        const elementType = freshTypeVariable(ctx, call.line, call.column)

        const expectedListType = new AST.GenericType(
          "List",
          [elementType],
          call.line,
          call.column
        )
        addConstraint(
          ctx,
          new TypeConstraint(
            listType,
            expectedListType,
            call.line,
            call.column,
            "tail function requires List<T> argument"
          )
        )

        return expectedListType
      }
      addError(ctx, "tail function requires exactly one argument", call.line, call.column)
      return freshTypeVariable(ctx, call.line, call.column)
    }

    case "toInt": {
      // Type: 'a -> Int
      const firstArg = call.arguments[0]
      if (call.arguments.length === 1 && firstArg) {
        generateConstraintsForExpression(ctx, firstArg, env)
        return new AST.PrimitiveType("Int", call.line, call.column)
      }
      addError(ctx, "toInt function requires exactly one argument", call.line, call.column)
      return freshTypeVariable(ctx, call.line, call.column)
    }

    case "toFloat": {
      // Type: 'a -> Float
      const firstArg = call.arguments[0]
      if (call.arguments.length === 1 && firstArg) {
        generateConstraintsForExpression(ctx, firstArg, env)
        return new AST.PrimitiveType("Float", call.line, call.column)
      }
      addError(ctx, "toFloat function requires exactly one argument", call.line, call.column)
      return freshTypeVariable(ctx, call.line, call.column)
    }

    case "typeof": {
      // Type: 'a -> String (polymorphic)
      const firstArg = call.arguments[0]
      if (call.arguments.length === 1 && firstArg) {
        generateConstraintsForExpression(ctx, firstArg, env)
        return new AST.PrimitiveType("String", call.line, call.column)
      }
      addError(ctx, "typeof function requires exactly one argument", call.line, call.column)
      return freshTypeVariable(ctx, call.line, call.column)
    }

    case "subscribe": {
      // Type: Signal<T> -> (T -> Unit) -> String
      const firstArg = call.arguments[0]
      const secondArg = call.arguments[1]
      if (call.arguments.length === 2 && firstArg && secondArg) {
        const signalType = generateConstraintsForExpression(ctx, firstArg, env)
        const observerType = generateConstraintsForExpression(ctx, secondArg, env)

        const valueType = freshTypeVariable(ctx, call.line, call.column)
        const expectedSignalType = new AST.GenericType(
          "Signal",
          [valueType],
          call.line,
          call.column
        )
        const expectedObserverType = new AST.FunctionType(
          valueType,
          new AST.PrimitiveType("Unit", call.line, call.column),
          call.line,
          call.column
        )

        addConstraint(
          ctx,
          new TypeConstraint(
            signalType,
            expectedSignalType,
            call.line,
            call.column,
            "subscribe requires Signal<T> as first argument"
          )
        )
        addConstraint(
          ctx,
          new TypeConstraint(
            observerType,
            expectedObserverType,
            call.line,
            call.column,
            "subscribe requires (T -> Unit) observer function as second argument"
          )
        )

        return new AST.PrimitiveType("String", call.line, call.column)
      }
      addError(
        ctx,
        "subscribe function requires exactly two arguments: signal and observer",
        call.line,
        call.column
      )
      return freshTypeVariable(ctx, call.line, call.column)
    }

    case "unsubscribe": {
      // Type: String -> Unit
      const firstArg = call.arguments[0]
      if (call.arguments.length === 1 && firstArg) {
        const keyType = generateConstraintsForExpression(ctx, firstArg, env)
        addConstraint(
          ctx,
          new TypeConstraint(
            keyType,
            new AST.PrimitiveType("String", call.line, call.column),
            call.line,
            call.column,
            "unsubscribe requires String subscription key"
          )
        )
        return new AST.PrimitiveType("Unit", call.line, call.column)
      }
      addError(
        ctx,
        "unsubscribe function requires exactly one argument: subscription key",
        call.line,
        call.column
      )
      return freshTypeVariable(ctx, call.line, call.column)
    }

    case "detach": {
      // Type: Signal<T> -> Unit
      const firstArg = call.arguments[0]
      if (call.arguments.length === 1 && firstArg) {
        const signalType = generateConstraintsForExpression(ctx, firstArg, env)
        const valueType = freshTypeVariable(ctx, call.line, call.column)
        const expectedSignalType = new AST.GenericType(
          "Signal",
          [valueType],
          call.line,
          call.column
        )

        addConstraint(
          ctx,
          new TypeConstraint(
            signalType,
            expectedSignalType,
            call.line,
            call.column,
            "detach requires Signal<T> argument"
          )
        )

        return new AST.PrimitiveType("Unit", call.line, call.column)
      }
      addError(
        ctx,
        "detach function requires exactly one argument: signal",
        call.line,
        call.column
      )
      return freshTypeVariable(ctx, call.line, call.column)
    }

    default:
      addError(
        ctx,
        `Unknown builtin function: ${call.functionName}`,
        call.line,
        call.column
      )
      return freshTypeVariable(ctx, call.line, call.column)
  }
}
