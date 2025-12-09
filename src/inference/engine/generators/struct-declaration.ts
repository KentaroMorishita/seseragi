/**
 * 構造体宣言とimplブロックの制約生成
 */

import * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import {
  addConstraint,
  addError,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { resolveTypeAlias } from "../type-alias-resolver"
import { generateConstraintsForExpression } from "./dispatcher"
import { generalize } from "./helpers"

/**
 * 構造体宣言の制約を生成
 * struct Point { x: Float, y: Float } のような宣言を処理
 */
export function generateConstraintsForStructDeclaration(
  ctx: InferenceContext,
  structDecl: AST.StructDeclaration,
  env: Map<string, AST.Type>
): void {
  // 構造体型を作成
  const structType = new AST.StructType(
    structDecl.name,
    structDecl.fields,
    structDecl.line,
    structDecl.column
  )

  // 環境に構造体型を登録
  env.set(structDecl.name, structType)

  // コンテキストにも登録
  ctx.structTypes.set(structDecl.name, structType)

  // nodeTypeMapにも登録
  ctx.nodeTypeMap.set(structDecl, structType)
}

/**
 * implブロックの制約を生成
 * impl Point { ... } のようなブロックを処理
 */
export function generateConstraintsForImplBlock(
  ctx: InferenceContext,
  implBlock: AST.ImplBlock,
  env: Map<string, AST.Type>
): void {
  // impl ブロックの型名が存在するかチェック
  const implType = env.get(implBlock.typeName)
  if (!implType) {
    addError(
      ctx,
      `Unknown type for impl block: ${implBlock.typeName}`,
      implBlock.line,
      implBlock.column
    )
    return
  }

  // メソッドの制約を生成
  for (const method of implBlock.methods) {
    generateConstraintsForMethodDeclaration(ctx, method, env, implType)
  }

  // 演算子の制約を生成
  for (const operator of implBlock.operators) {
    generateConstraintsForOperatorDeclaration(ctx, operator, env, implType)
  }

  // モノイドの制約を生成
  if (implBlock.monoid) {
    generateConstraintsForMonoidDeclaration(ctx, implBlock.monoid, env, implType)
  }
}

/**
 * メソッド宣言の制約を生成
 */
export function generateConstraintsForMethodDeclaration(
  ctx: InferenceContext,
  method: AST.MethodDeclaration,
  env: Map<string, AST.Type>,
  implType: AST.Type
): void {
  // メソッドを環境に登録
  if (implType.kind === "StructType") {
    const structType = implType as AST.StructType
    const methodKey = `${structType.name}.${method.name}`
    ctx.methodEnvironment.set(methodKey, method)
  }

  // パラメータの型を解決
  const resolvedParameters = method.parameters.map((param) => ({
    ...param,
    type: resolveParameterType(ctx, param, implType, env),
  }))

  // 関数型を構築
  const functionType = buildFunctionType(ctx, resolvedParameters, method.returnType)

  // 環境に追加
  env.set(`${method.name}`, functionType)
  ctx.nodeTypeMap.set(method, functionType)

  // メソッド本体用の環境を作成
  const methodEnv = createMethodEnvironment(env, implType)
  addMethodParametersToEnvironment(ctx, method, methodEnv, implType, env)

  // メソッド本体の型を推論
  const bodyType = generateConstraintsForExpression(ctx, method.body, methodEnv)

  // 制約を追加
  addConstraint(
    ctx,
    new TypeConstraint(
      bodyType,
      method.returnType,
      method.line,
      method.column,
      `Method ${method.name} body type`
    )
  )
}

/**
 * 演算子宣言の制約を生成
 */
export function generateConstraintsForOperatorDeclaration(
  ctx: InferenceContext,
  operator: AST.OperatorDeclaration,
  env: Map<string, AST.Type>,
  implType: AST.Type
): void {
  // パラメータの型を解決
  const resolvedParameters = operator.parameters.map((param) => ({
    ...param,
    type: resolveParameterType(ctx, param, implType, env),
  }))

  // 関数型を構築
  const functionType = buildFunctionType(ctx, resolvedParameters, operator.returnType)

  // 一般化して環境に追加
  const generalizedOperatorType = generalize(functionType, env)
  env.set(`${operator.operator}`, generalizedOperatorType)
  ctx.nodeTypeMap.set(operator, generalizedOperatorType)

  // 演算子本体用の環境を作成
  const operatorEnv = createMethodEnvironment(env, implType)
  addParametersToEnvironment(ctx, operator.parameters, operatorEnv, implType, env)

  // 演算子本体の型を推論
  const bodyType = generateConstraintsForExpression(ctx, operator.body, operatorEnv)

  // 制約を追加
  addConstraint(
    ctx,
    new TypeConstraint(
      bodyType,
      operator.returnType,
      operator.line,
      operator.column,
      `Operator ${operator.operator} body type`
    )
  )
}

/**
 * モノイド宣言の制約を生成
 */
function generateConstraintsForMonoidDeclaration(
  ctx: InferenceContext,
  monoid: AST.MonoidDeclaration,
  env: Map<string, AST.Type>,
  implType: AST.Type
): void {
  // empty（単位元）の型チェック
  const emptyType = generateConstraintsForExpression(ctx, monoid.empty, env)
  addConstraint(
    ctx,
    new TypeConstraint(
      emptyType,
      implType,
      monoid.line,
      monoid.column,
      `Monoid empty must be of type ${implType.kind === "StructType" ? (implType as AST.StructType).name : "impl type"}`
    )
  )

  // combine（結合演算）の型チェック
  const combineEnv = new Map(env)
  combineEnv.set("self", implType)
  combineEnv.set("other", implType)

  const combineType = generateConstraintsForExpression(ctx, monoid.combine, combineEnv)
  addConstraint(
    ctx,
    new TypeConstraint(
      combineType,
      implType,
      monoid.line,
      monoid.column,
      "Monoid combine must return impl type"
    )
  )
}

// ============================================================
// ヘルパー関数
// ============================================================

/**
 * パラメータの型を解決
 */
function resolveParameterType(
  ctx: InferenceContext,
  param: AST.Parameter,
  implType: AST.Type,
  env: Map<string, AST.Type>
): AST.Type {
  if (param.isImplicitSelf || param.isImplicitOther) {
    return implType
  }

  const resolvedType = resolveTypeAlias(ctx, param.type)
  return resolveStructTypeFromEnvironment(resolvedType, env)
}

/**
 * 環境から構造体型を解決
 */
function resolveStructTypeFromEnvironment(
  resolvedType: AST.Type,
  env: Map<string, AST.Type>
): AST.Type {
  if (resolvedType.kind === "PrimitiveType") {
    const structTypeFromEnv = env.get((resolvedType as AST.PrimitiveType).name)
    if (structTypeFromEnv?.kind === "StructType") {
      return structTypeFromEnv
    }
  }
  return resolvedType
}

/**
 * 関数型を構築（カリー化形式）
 */
function buildFunctionType(
  ctx: InferenceContext,
  parameters: AST.Parameter[],
  returnType: AST.Type
): AST.Type {
  let funcType: AST.Type = returnType

  // 引数なしの場合は Unit -> ReturnType
  if (parameters.length === 0) {
    const unitType = new AST.PrimitiveType("Unit", returnType.line, returnType.column)
    return new AST.FunctionType(unitType, funcType, returnType.line, returnType.column)
  }

  // 後ろから前にカリー化関数型を構築
  for (let i = parameters.length - 1; i >= 0; i--) {
    const param = parameters[i]
    if (param) {
      funcType = new AST.FunctionType(
        param.type,
        funcType,
        param.line,
        param.column
      )
    }
  }

  return funcType
}

/**
 * メソッド環境を作成
 */
function createMethodEnvironment(
  env: Map<string, AST.Type>,
  implType: AST.Type
): Map<string, AST.Type> {
  const methodEnv = new Map(env)

  if (implType.kind === "StructType") {
    const structType = implType as AST.StructType
    methodEnv.set(structType.name, implType)
  }

  // 構造体型を環境にコピー
  for (const [key, value] of env.entries()) {
    if (value.kind === "StructType") {
      methodEnv.set(key, value)
    }
  }

  return methodEnv
}

/**
 * メソッドパラメータを環境に追加
 */
function addMethodParametersToEnvironment(
  ctx: InferenceContext,
  method: AST.MethodDeclaration,
  methodEnv: Map<string, AST.Type>,
  implType: AST.Type,
  env: Map<string, AST.Type>
): void {
  for (const param of method.parameters) {
    const resolvedType = resolveParameterType(ctx, param, implType, env)
    methodEnv.set(param.name, resolvedType)
  }
}

/**
 * 演算子パラメータを環境に追加
 */
function addParametersToEnvironment(
  ctx: InferenceContext,
  parameters: AST.Parameter[],
  operatorEnv: Map<string, AST.Type>,
  implType: AST.Type,
  env: Map<string, AST.Type>
): void {
  for (const param of parameters) {
    const resolvedType = resolveParameterType(ctx, param, implType, env)
    operatorEnv.set(param.name, resolvedType)
    ctx.nodeTypeMap.set(param, resolvedType)
  }
}
