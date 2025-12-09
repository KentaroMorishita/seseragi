/**
 * ConstructorExpression の制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import { PolymorphicTypeVariable } from "../../type-variables"
import {
  addConstraint,
  addError,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * コンストラクタ式の制約を生成
 */
export function generateConstraintsForConstructorExpression(
  ctx: InferenceContext,
  ctor: AST.ConstructorExpression,
  env: Map<string, AST.Type>
): AST.Type {
  const constructorName = ctor.constructorName

  switch (constructorName) {
    case "Just":
      if (ctor.arguments && ctor.arguments.length > 0) {
        const argType = generateConstraintsForExpression(
          ctx,
          ctor.arguments[0],
          env
        )
        return new AST.GenericType("Maybe", [argType], ctor.line, ctor.column)
      } else if (!ctor.arguments || ctor.arguments.length === 0) {
        // Just without arguments - treat as a curried function
        // Just : 'a -> Maybe<'a>
        const elemType = new PolymorphicTypeVariable("a", ctor.line, ctor.column)
        const maybeType = new AST.GenericType(
          "Maybe",
          [elemType],
          ctor.line,
          ctor.column
        )
        return new AST.FunctionType(elemType, maybeType, ctor.line, ctor.column)
      }
      addError(
        ctx,
        "Just constructor requires exactly one argument",
        ctor.line,
        ctor.column
      )
      return new AST.GenericType(
        "Maybe",
        [freshTypeVariable(ctx, ctor.line, ctor.column)],
        ctor.line,
        ctor.column
      )

    case "Nothing":
      // Nothing doesn't take arguments
      if (ctor.arguments && ctor.arguments.length > 0) {
        addError(
          ctx,
          "Nothing constructor does not take any arguments",
          ctor.line,
          ctor.column
        )
      }
      // Nothing is polymorphic: Maybe<'a>
      return new AST.GenericType(
        "Maybe",
        [new PolymorphicTypeVariable("a", ctor.line, ctor.column)],
        ctor.line,
        ctor.column
      )

    case "Right":
      if (ctor.arguments && ctor.arguments.length > 0) {
        const argType = generateConstraintsForExpression(
          ctx,
          ctor.arguments[0],
          env
        )
        // Right is polymorphic in its left type: Either<'a, argType>
        return new AST.GenericType(
          "Either",
          [new PolymorphicTypeVariable("a", ctor.line, ctor.column), argType],
          ctor.line,
          ctor.column
        )
      } else if (!ctor.arguments || ctor.arguments.length === 0) {
        // Right without arguments - treat as a curried function
        // Right : 'b -> Either<'a, 'b>
        const leftType = new PolymorphicTypeVariable("a", ctor.line, ctor.column)
        const rightType = new PolymorphicTypeVariable("b", ctor.line, ctor.column)
        const eitherType = new AST.GenericType(
          "Either",
          [leftType, rightType],
          ctor.line,
          ctor.column
        )
        return new AST.FunctionType(rightType, eitherType, ctor.line, ctor.column)
      }
      addError(
        ctx,
        "Right constructor requires exactly one argument",
        ctor.line,
        ctor.column
      )
      return new AST.GenericType(
        "Either",
        [
          new PolymorphicTypeVariable("a", ctor.line, ctor.column),
          new PolymorphicTypeVariable("b", ctor.line, ctor.column),
        ],
        ctor.line,
        ctor.column
      )

    case "Left":
      if (ctor.arguments && ctor.arguments.length > 0) {
        const argType = generateConstraintsForExpression(
          ctx,
          ctor.arguments[0],
          env
        )
        // Left is polymorphic in its right type: Either<argType, 'b>
        return new AST.GenericType(
          "Either",
          [argType, new PolymorphicTypeVariable("b", ctor.line, ctor.column)],
          ctor.line,
          ctor.column
        )
      } else if (!ctor.arguments || ctor.arguments.length === 0) {
        // Left without arguments - treat as a curried function
        // Left : 'a -> Either<'a, 'b>
        const leftType = new PolymorphicTypeVariable("a", ctor.line, ctor.column)
        const rightType = new PolymorphicTypeVariable("b", ctor.line, ctor.column)
        const eitherType = new AST.GenericType(
          "Either",
          [leftType, rightType],
          ctor.line,
          ctor.column
        )
        return new AST.FunctionType(leftType, eitherType, ctor.line, ctor.column)
      }
      addError(
        ctx,
        "Left constructor requires exactly one argument",
        ctor.line,
        ctor.column
      )
      return new AST.GenericType(
        "Either",
        [
          new PolymorphicTypeVariable("a", ctor.line, ctor.column),
          new PolymorphicTypeVariable("b", ctor.line, ctor.column),
        ],
        ctor.line,
        ctor.column
      )

    case "Task":
      if (ctor.arguments && ctor.arguments.length > 0) {
        const argType = generateConstraintsForExpression(
          ctx,
          ctor.arguments[0],
          env
        )
        // Task expects (() -> Promise<T>) -> Task<T>
        const typeVar = new PolymorphicTypeVariable("a", ctor.line, ctor.column)
        const promiseType = new AST.GenericType(
          "Promise",
          [typeVar],
          ctor.line,
          ctor.column
        )
        const expectedArgType = new AST.FunctionType(
          new AST.PrimitiveType("Unit", ctor.line, ctor.column),
          promiseType,
          ctor.line,
          ctor.column
        )

        addConstraint(
          ctx,
          new TypeConstraint(
            argType,
            expectedArgType,
            ctor.line,
            ctor.column,
            "Task constructor argument constraint"
          )
        )

        return new AST.GenericType("Task", [typeVar], ctor.line, ctor.column)
      } else if (!ctor.arguments || ctor.arguments.length === 0) {
        // Task without arguments - treat as a curried function
        // Task : (() -> Promise<'a>) -> Task<'a>
        const typeVar = new PolymorphicTypeVariable("a", ctor.line, ctor.column)
        const promiseType = new AST.GenericType(
          "Promise",
          [typeVar],
          ctor.line,
          ctor.column
        )
        const computationType = new AST.FunctionType(
          new AST.PrimitiveType("Unit", ctor.line, ctor.column),
          promiseType,
          ctor.line,
          ctor.column
        )
        const taskType = new AST.GenericType(
          "Task",
          [typeVar],
          ctor.line,
          ctor.column
        )
        return new AST.FunctionType(
          computationType,
          taskType,
          ctor.line,
          ctor.column
        )
      }
      addError(
        ctx,
        "Task constructor requires exactly one argument",
        ctor.line,
        ctor.column
      )
      return new AST.GenericType(
        "Task",
        [new PolymorphicTypeVariable("a", ctor.line, ctor.column)],
        ctor.line,
        ctor.column
      )

    case "Empty":
      // Empty is polymorphic: List<'a>
      return new AST.GenericType(
        "List",
        [new PolymorphicTypeVariable("a", ctor.line, ctor.column)],
        ctor.line,
        ctor.column
      )

    case "Cons":
      if (ctor.arguments && ctor.arguments.length === 2) {
        const headType = generateConstraintsForExpression(
          ctx,
          ctor.arguments[0],
          env
        )
        const tailType = generateConstraintsForExpression(
          ctx,
          ctor.arguments[1],
          env
        )

        // Cons head tail should have type List<headType>
        const expectedTailType = new AST.GenericType(
          "List",
          [headType],
          ctor.line,
          ctor.column
        )

        // Add constraint that tail must be List<headType>
        addConstraint(
          ctx,
          new TypeConstraint(
            tailType,
            expectedTailType,
            ctor.line,
            ctor.column,
            "Cons tail type"
          )
        )

        return expectedTailType
      } else if (!ctor.arguments || ctor.arguments.length === 0) {
        // Cons without arguments - treat as a curried function
        // Cons : 'a -> List<'a> -> List<'a>
        const elemType = new PolymorphicTypeVariable("a", ctor.line, ctor.column)
        const listType = new AST.GenericType(
          "List",
          [elemType],
          ctor.line,
          ctor.column
        )
        return new AST.FunctionType(
          elemType,
          new AST.FunctionType(listType, listType, ctor.line, ctor.column),
          ctor.line,
          ctor.column
        )
      }
      addError(
        ctx,
        "Cons constructor requires exactly two arguments (head and tail)",
        ctor.line,
        ctor.column
      )
      return new AST.GenericType(
        "List",
        [new PolymorphicTypeVariable("a", ctor.line, ctor.column)],
        ctor.line,
        ctor.column
      )

    default: {
      // Check if this is an ADT constructor from the environment
      const constructorType = env.get(constructorName)
      if (constructorType) {
        return applyConstructor(ctx, constructorType, ctor, env)
      }

      addError(ctx, `Unknown constructor: ${constructorName}`, ctor.line, ctor.column)
      return freshTypeVariable(ctx, ctor.line, ctor.column)
    }
  }
}

/**
 * ADTコンストラクタを適用
 */
function applyConstructor(
  ctx: InferenceContext,
  constructorType: AST.Type,
  ctor: AST.ConstructorExpression,
  env: Map<string, AST.Type>
): AST.Type {
  // Extract parameter types and result type from constructor function type
  let currentType = constructorType
  const expectedParamTypes: AST.Type[] = []

  // Traverse function type to get parameter types
  while (currentType instanceof AST.FunctionType) {
    expectedParamTypes.push(currentType.paramType)
    currentType = currentType.returnType
  }

  // The final type should be the ADT type
  const resultType = currentType

  // Check argument count
  if (ctor.arguments.length !== expectedParamTypes.length) {
    addError(
      ctx,
      `Constructor ${ctor.constructorName} expects ${expectedParamTypes.length} arguments, but got ${ctor.arguments.length}`,
      ctor.line,
      ctor.column
    )
    return resultType
  }

  // Type check each argument
  for (let i = 0; i < ctor.arguments.length; i++) {
    const argType = generateConstraintsForExpression(ctx, ctor.arguments[i], env)

    // Add constraint that argument type matches expected parameter type
    addConstraint(
      ctx,
      new TypeConstraint(
        argType,
        expectedParamTypes[i],
        ctor.arguments[i].line,
        ctor.arguments[i].column,
        `Constructor ${ctor.constructorName} argument ${i + 1}`
      )
    )
  }

  return resultType
}
