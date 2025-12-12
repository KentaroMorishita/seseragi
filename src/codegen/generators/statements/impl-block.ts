/**
 * Impl Block Generator - implブロックの生成
 */

import type {
  ImplBlock,
  MethodDeclaration,
  MonoidDeclaration,
  OperatorDeclaration,
} from "../../../ast"
import type { CodeGenContext } from "../../context"
import {
  enterStructContext,
  exitStructContext,
  getIndent,
  registerStructMethod,
  registerStructOperator,
} from "../../context"
import { operatorToMethodName, sanitizeIdentifier } from "../../helpers"
import { generateType } from "../../type-generators"
import { generateExpression } from "../dispatcher"

/**
 * implブロックをTypeScriptコードに変換
 *
 * implブロック内のメソッドと演算子を生成し、
 * structMethods/structOperatorsのディスパッチテーブルに登録する。
 *
 * 構造:
 * - メソッド: グローバル関数として生成（ファイルハッシュベースの一意な名前）
 * - 演算子: グローバル関数として生成（演算子名を安全な識別子に変換）
 * - モノイド: identity定数と演算子の組み合わせ
 *
 * 例:
 * ```seseragi
 * impl Point {
 *   fn distance(self: Point, other: Point): Float { ... }
 *   op +(self: Point, other: Point): Point { ... }
 * }
 * ```
 * ↓
 * ```typescript
 * // Point implementation
 * function __ssrg_Point_<hash>_distance(self: Point, other: Point): number {
 *   return ...;
 * }
 * function __ssrg_Point_<hash>_op_add(self: Point, other: Point): Point {
 *   return ...;
 * }
 * ```
 */
export function generateImplBlock(
  ctx: CodeGenContext,
  implBlock: ImplBlock
): string {
  const indent = getIndent(ctx)
  const lines: string[] = []

  // 構造体コンテキストを設定
  enterStructContext(ctx, implBlock.typeName)

  // implブロックコメント
  lines.push(`${indent}// ${implBlock.typeName} implementation`)

  // メソッドの生成と登録
  for (const method of implBlock.methods) {
    const methodCode = generateMethodDeclaration(ctx, method)
    lines.push(methodCode)
    registerStructMethod(ctx, implBlock.typeName, method.name)
  }

  // 演算子の生成と登録
  for (const operator of implBlock.operators) {
    const operatorCode = generateOperatorDeclaration(ctx, operator)
    lines.push(operatorCode)
    registerStructOperator(ctx, implBlock.typeName, operator.operator)
  }

  // モノイドの生成
  if (implBlock.monoid) {
    const monoidCode = generateMonoidDeclaration(ctx, implBlock.monoid)
    lines.push(monoidCode)
  }

  // コンテキストを復元
  exitStructContext(ctx)

  return lines.join("\n")
}

/**
 * メソッド宣言を生成
 *
 * 構造体名とファイルハッシュベースの一意な名前を生成する。
 * 形式: __ssrg_<StructName>_<filePrefix>_<methodName>
 */
function generateMethodDeclaration(
  ctx: CodeGenContext,
  method: MethodDeclaration
): string {
  const indent = getIndent(ctx)
  const params = method.parameters
    .map((p) => `${sanitizeIdentifier(p.name)}: ${generateType(p.type)}`)
    .join(", ")
  const returnType = generateType(method.returnType)
  const body = generateExpression(ctx, method.body)

  // 構造体名とファイルハッシュベースの一意な名前を生成
  const structPrefix = ctx.currentStructContext
    ? `${ctx.currentStructContext}_`
    : ""
  const methodName = `__ssrg_${structPrefix}${ctx.filePrefix}_${method.name}`

  return `${indent}function ${methodName}(${params}): ${returnType} {
${indent}  return ${body};
${indent}}`
}

/**
 * 演算子宣言を生成
 *
 * 演算子名を安全な識別子に変換して関数として生成する。
 * 形式: __ssrg_<StructName>_<filePrefix>_op_<operatorName>
 */
function generateOperatorDeclaration(
  ctx: CodeGenContext,
  operator: OperatorDeclaration
): string {
  const indent = getIndent(ctx)
  const params = operator.parameters
    .map((p) => `${sanitizeIdentifier(p.name)}: ${generateType(p.type)}`)
    .join(", ")
  const returnType = generateType(operator.returnType)
  const body = generateExpression(ctx, operator.body)

  // 演算子名を安全な識別子に変換
  const opMethodName = operatorToMethodName(operator.operator)
  const structPrefix = ctx.currentStructContext
    ? `${ctx.currentStructContext}_`
    : ""
  const operatorName = `__ssrg_${structPrefix}${ctx.filePrefix}_op_${opMethodName}`

  return `${indent}function ${operatorName}(${params}): ${returnType} {
${indent}  return ${body};
${indent}}`
}

/**
 * モノイド宣言を生成
 *
 * identityとoperatorの組み合わせ。
 * - identity: エクスポートされた定数
 * - operator: 演算子宣言として生成
 */
function generateMonoidDeclaration(
  ctx: CodeGenContext,
  monoid: MonoidDeclaration
): string {
  const indent = getIndent(ctx)
  const lines: string[] = []

  lines.push(`${indent}// Monoid implementation`)
  lines.push(
    `${indent}export const identity = ${generateExpression(ctx, monoid.identity)};`
  )
  lines.push(generateOperatorDeclaration(ctx, monoid.operator))

  return lines.join("\n")
}
