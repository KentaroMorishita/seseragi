/**
 * メソッド呼び出しの制約生成
 */

import * as AST from "../../../ast"
import { formatType } from "../../type-formatter"
import { TypeConstraint } from "../../constraints"
import {
  addConstraint,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * メソッド呼び出しの制約を生成
 * receiver.method(arg1, arg2, ...) のような式を処理
 */
export function generateConstraintsForMethodCall(
  ctx: InferenceContext,
  call: AST.MethodCall,
  env: Map<string, AST.Type>
): AST.Type {
  // レシーバーの型を推論
  const receiverType = generateConstraintsForExpression(ctx, call.receiver, env)

  // 引数の型を推論
  const argTypes: AST.Type[] = []
  for (const arg of call.arguments) {
    argTypes.push(generateConstraintsForExpression(ctx, arg, env))
  }

  // レシーバーの型名を取得
  let receiverTypeName: string | null = null
  if (receiverType.kind === "StructType") {
    receiverTypeName = (receiverType as AST.StructType).name
  } else if (receiverType.kind === "PrimitiveType") {
    receiverTypeName = (receiverType as AST.PrimitiveType).name
  } else if (receiverType.kind === "TypeVariable") {
    // 型変数の場合、nodeTypeMapから解決を試みる
    for (const [_node, type] of ctx.nodeTypeMap.entries()) {
      if (type === receiverType && type.kind === "StructType") {
        receiverTypeName = (type as AST.StructType).name
        break
      }
    }
  }

  // implブロックからメソッドを検索
  let methodReturnType: AST.Type | null = null
  if (receiverTypeName) {
    const methodKey = `${receiverTypeName}.${call.methodName}`
    const methodInfo = ctx.methodEnvironment.get(methodKey)

    if (methodInfo && methodInfo.kind === "MethodDeclaration") {
      const method = methodInfo as AST.MethodDeclaration

      if (method.returnType) {
        methodReturnType = method.returnType

        // 引数の型チェック（selfを除く）
        const methodParams = method.parameters.filter((p) => !p.isImplicitSelf)
        if (methodParams.length === argTypes.length) {
          for (let i = 0; i < methodParams.length; i++) {
            const argType = argTypes[i]
            const paramType = methodParams[i]?.type
            if (argType && paramType) {
              addConstraint(
                ctx,
                new TypeConstraint(
                  argType,
                  paramType,
                  call.line,
                  call.column,
                  `Method argument ${i} type mismatch`
                )
              )
            }
          }
        }

        // レシーバー型制約を追加
        addConstraint(
          ctx,
          new TypeConstraint(
            receiverType,
            new AST.StructType(receiverTypeName, [], call.line, call.column),
            call.line,
            call.column,
            `Method receiver type mismatch`
          )
        )
      }
    }
  }

  // メソッドが見つからない場合は新しい型変数を使用
  if (!methodReturnType) {
    methodReturnType = freshTypeVariable(ctx, call.line, call.column)

    // カリー化されたFunction型として制約を構築
    let expectedMethodType: AST.Type = methodReturnType

    for (let i = argTypes.length - 1; i >= 0; i--) {
      const argType = argTypes[i]
      if (argType) {
        expectedMethodType = new AST.FunctionType(
          argType,
          expectedMethodType,
          call.line,
          call.column
        )
      }
    }

    expectedMethodType = new AST.FunctionType(
      receiverType,
      expectedMethodType,
      call.line,
      call.column
    )

    addConstraint(
      ctx,
      new TypeConstraint(
        expectedMethodType,
        expectedMethodType,
        call.line,
        call.column,
        `Method call ${call.methodName} on type ${formatType(receiverType)}`
      )
    )
  }

  return methodReturnType
}
