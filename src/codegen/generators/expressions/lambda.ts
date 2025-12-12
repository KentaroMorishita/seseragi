/**
 * ラムダ式（LambdaExpression）のコード生成
 */

import type { LambdaExpression } from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * 型をTypeScript文字列に変換（シンプル版）
 */
function generateType(type: any): string {
  if (!type) return "any"

  switch (type.kind) {
    case "PrimitiveType": {
      // Seseragiの型をTypeScriptの型に変換
      const typeMap: Record<string, string> = {
        Int: "number",
        Float: "number",
        Bool: "boolean",
        String: "string",
        Char: "string",
        Unit: "void",
      }
      return typeMap[type.name] || type.name
    }
    case "FunctionType":
      return `(${generateType(type.paramType)}) => ${generateType(type.returnType)}`
    case "GenericType": {
      const args = type.typeArguments?.map(generateType).join(", ") || ""
      return `${type.name}<${args}>`
    }
    case "RecordType": {
      const fields = type.fields
        ?.map((f: any) => `${f.name}: ${generateType(f.type)}`)
        .join(", ")
      return `{ ${fields} }`
    }
    default:
      return "any"
  }
}

/**
 * ラムダ式をTypeScriptコードに変換
 */
export function generateLambdaExpression(
  ctx: CodeGenContext,
  expr: LambdaExpression
): string {
  const body = generateExpression(ctx, expr.body)

  // 単一パラメータのラムダはアロー関数構文を使用
  if (expr.parameters.length === 1) {
    const param = expr.parameters[0]!
    const paramWithType = `${param.name}: ${generateType(param.type)}`
    return `(${paramWithType}) => ${body}`
  }

  // 複数パラメータのラムダはカリー化
  // \a -> \b -> expr は (a) => (b) => expr になる
  let result = body
  for (let i = expr.parameters.length - 1; i >= 0; i--) {
    const param = expr.parameters[i]!
    const paramWithType = `${param.name}: ${generateType(param.type)}`
    result = `(${paramWithType}) => ${result}`
  }

  return result
}
