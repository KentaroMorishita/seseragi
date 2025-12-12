/**
 * 関数宣言生成 - Function Declaration Generator
 *
 * カリー化関数（複数パラメータ → ネストしたクロージャ）をサポート
 * PromiseBlock の特別処理、型パラメータ、元の型注釈の優先使用を実装
 */

import type {
  BlockExpression,
  FunctionDeclaration,
  Parameter,
  Type,
} from "../../../ast"
import { FunctionType as FunctionTypeClass, PrimitiveType } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { getIndent, registerFunctionType } from "../../context"
import { sanitizeIdentifier } from "../../helpers"
import { generateType } from "../../type-generators"
import { generateExpression } from "../dispatcher"

/**
 * 関数宣言をTypeScriptコードに変換
 *
 * 対応機能:
 * - カリー化関数（複数パラメータ → f(a)(b)(c) 形式のクロージャ）
 * - PromiseBlock の直接生成（IIFEを避ける）
 * - ジェネリック型パラメータ
 * - 元の型注釈情報の優先使用（originalParameters/originalReturnType）
 *
 * @param ctx - コード生成コンテキスト
 * @param func - 関数宣言AST
 * @returns 生成されたTypeScriptコード
 */
export function generateFunctionDeclaration(
  ctx: CodeGenContext,
  func: FunctionDeclaration
): string {
  const indent = getIndent(ctx)

  // 関数の型を登録（カリー化対応）
  registerFunctionTypeInContext(ctx, func)

  // 元の型注釈情報を優先使用（どれか一つでも存在すれば優先）
  const useOriginalTypes = !!(
    func.originalTypeParameters ||
    func.originalParameters ||
    func.originalReturnType
  )

  // 型パラメータの生成
  const typeParams = generateTypeParameters(func, useOriginalTypes)

  // 現在の関数の型パラメータコンテキストを設定
  ctx.currentFunctionTypeParams =
    (useOriginalTypes ? func.originalTypeParameters : func.typeParameters) || []

  // パラメータと戻り値型を取得（元の型注釈を優先）
  const parameters = useOriginalTypes
    ? func.originalParameters
    : func.parameters
  const funcReturnType = useOriginalTypes
    ? func.originalReturnType
    : func.returnType

  // 関数本体の生成
  const body = generateFunctionBody(ctx, func)

  let result: string

  if (parameters.length > 1) {
    // カリー化関数として生成
    result = generateCurriedFunction(
      ctx,
      func,
      parameters,
      funcReturnType,
      body,
      typeParams,
      indent
    )
  } else {
    // 単一パラメータ関数として生成
    result = generateSingleParamFunction(
      ctx,
      func,
      parameters,
      funcReturnType,
      body,
      typeParams,
      indent
    )
  }

  // コンテキストをクリア
  ctx.currentFunctionTypeParams = []

  return result
}

/**
 * 関数型をコンテキストに登録（カリー化対応）
 */
function registerFunctionTypeInContext(
  ctx: CodeGenContext,
  func: FunctionDeclaration
): void {
  const funcType = new FunctionTypeClass(
    func.parameters[0]?.type || new PrimitiveType("unknown", 0, 0),
    func.returnType,
    0,
    0
  )

  // カリー化された関数の場合、ネストした関数型を構築
  if (func.parameters.length > 1) {
    let currentType: Type = funcType.returnType
    for (let i = func.parameters.length - 1; i > 0; i--) {
      currentType = new FunctionTypeClass(
        func.parameters[i].type,
        currentType,
        0,
        0
      )
    }
    registerFunctionType(
      ctx,
      func.name,
      new FunctionTypeClass(func.parameters[0].type, currentType, 0, 0)
    )
  } else {
    registerFunctionType(ctx, func.name, funcType)
  }
}

/**
 * 型パラメータ文字列を生成
 */
function generateTypeParameters(
  func: FunctionDeclaration,
  useOriginalTypes: boolean
): string {
  const typeParamsArray = useOriginalTypes
    ? func.originalTypeParameters
    : func.typeParameters

  if (!typeParamsArray || typeParamsArray.length === 0) {
    return ""
  }

  return `<${typeParamsArray.map((tp) => tp.name).join(", ")}>`
}

/**
 * 関数本体を生成（PromiseBlock の特別処理含む）
 */
function generateFunctionBody(
  ctx: CodeGenContext,
  func: FunctionDeclaration
): string {
  // 関数本体が単一のPromise ブロックの場合は、IIFEを避けて直接生成
  if (func.body.kind === "BlockExpression") {
    const blockExpr = func.body as BlockExpression
    if (
      blockExpr.statements.length === 0 &&
      blockExpr.returnExpression &&
      blockExpr.returnExpression.kind === "PromiseBlock"
    ) {
      // 単一のPromise ブロックの場合は直接生成
      return generateExpression(ctx, blockExpr.returnExpression)
    }
  }

  // その他の場合は通常通り
  return generateExpression(ctx, func.body)
}

/**
 * カリー化関数を生成（複数パラメータ）
 *
 * 例: f(a: Int, b: Int): Int => function f(a: number): (arg: number) => number { ... }
 */
function generateCurriedFunction(
  _ctx: CodeGenContext,
  func: FunctionDeclaration,
  parameters: Parameter[],
  funcReturnType: Type,
  body: string,
  typeParams: string,
  indent: string
): string {
  // カリー化された戻り値型を構築: B => C => ... => ReturnType
  const returnType = generateType(funcReturnType)
  let curriedReturnType = returnType

  for (let i = parameters.length - 1; i >= 1; i--) {
    const paramType = generateType(parameters[i].type)
    curriedReturnType = `(arg: ${paramType}) => ${curriedReturnType}`
  }

  // パラメータを逆順に処理してネストしたクロージャを作成
  let result = body
  for (let i = parameters.length - 1; i >= 1; i--) {
    const param = parameters[i]
    const paramName = sanitizeIdentifier(param.name)
    const paramType = generateType(param.type)

    result = `function(${paramName}: ${paramType}) {\n${indent}      return ${result};\n${indent}    }`
  }

  // 最初のパラメータで関数を定義
  const firstParam = parameters[0]
  const firstParamName = sanitizeIdentifier(firstParam.name)
  const firstParamType = generateType(firstParam.type)

  return `${indent}function ${sanitizeIdentifier(func.name)}${typeParams}(${firstParamName}: ${firstParamType}): ${curriedReturnType} {\n${indent}  return ${result};\n${indent}}`
}

/**
 * 単一パラメータ関数を生成
 */
function generateSingleParamFunction(
  _ctx: CodeGenContext,
  func: FunctionDeclaration,
  parameters: Parameter[],
  funcReturnType: Type,
  body: string,
  typeParams: string,
  indent: string
): string {
  const params = parameters
    .map((p) => `${sanitizeIdentifier(p.name)}: ${generateType(p.type)}`)
    .join(", ")

  const returnType = generateType(funcReturnType)

  return `${indent}function ${sanitizeIdentifier(func.name)}${typeParams}(${params}): ${returnType} {\n${indent}  return ${body};\n${indent}}`
}
