/**
 * Expression Dispatcher - 式生成のルーター
 *
 * 各式の種類に応じて適切なジェネレーターに振り分ける
 */

import type { Expression, Identifier, Literal } from "../../ast"
import type { CodeGenContext } from "../context"
import { sanitizeIdentifier } from "../helpers"

// Expression generators
import { generateLiteral } from "./expressions/literal"
import {
  generateConditionalExpression,
  generateTernaryExpression,
} from "./expressions/conditional"
import { generateUnaryOperation } from "./expressions/unary-operation"
import { generateTemplateExpression } from "./expressions/template"

/**
 * 式をTypeScriptコードに変換
 *
 * 式の種類に応じて適切なジェネレーターに振り分ける
 */
export function generateExpression(
  ctx: CodeGenContext,
  expr: Expression
): string {
  switch (expr.kind) {
    case "Literal":
      return generateLiteral(ctx, expr as Literal)

    case "Identifier":
      return sanitizeIdentifier((expr as Identifier).name)

    case "TemplateExpression":
      return generateTemplateExpression(ctx, expr as any)

    case "UnaryOperation":
      return generateUnaryOperation(ctx, expr as any)

    case "ConditionalExpression":
      return generateConditionalExpression(ctx, expr as any)

    case "TernaryExpression":
      return generateTernaryExpression(ctx, expr as any)

    // ===================================================================
    // TODO: 以下の式は段階的に移行する
    // 現在は旧CodeGeneratorクラスへフォールバック
    // ===================================================================

    case "BinaryOperation":
    case "NullishCoalescingExpression":
    case "FunctionCall":
    case "MethodCall":
    case "FunctionApplication":
    case "BuiltinFunctionCall":
    case "MatchExpression":
    case "Pipeline":
    case "ReversePipe":
    case "FunctorMap":
    case "ApplicativeApply":
    case "MonadBind":
    case "FoldMonoid":
    case "FunctionApplicationOperator":
    case "ConstructorExpression":
    case "BlockExpression":
    case "LambdaExpression":
    case "RecordExpression":
    case "RecordAccess":
    case "ArrayLiteral":
    case "ArrayAccess":
    case "ListSugar":
    case "ConsExpression":
    case "RangeLiteral":
    case "ListComprehension":
    case "ListComprehensionSugar":
    case "TupleExpression":
    case "StructExpression":
    case "SpreadExpression":
    case "TypeAssertion":
    case "IsExpression":
    case "PromiseBlock":
    case "ResolveExpression":
    case "RejectExpression":
    case "TryExpression":
    case "SignalExpression":
    case "AssignmentExpression":
      // 旧実装へのフォールバック
      // これらは段階的に新モジュールに移行する
      return fallbackToLegacy(ctx, expr)

    default:
      return `/* Unsupported expression: ${(expr as any).kind} */`
  }
}

/**
 * 旧CodeGeneratorクラスへのフォールバック
 *
 * 新モジュールに移行されていない式は旧実装を使用する
 * このフォールバックは段階的に削除される
 */
function fallbackToLegacy(ctx: CodeGenContext, expr: Expression): string {
  // 旧実装へのフォールバック
  // CodeGeneratorインスタンスがctxに存在する場合はそれを使用
  if ((ctx as any).legacyGenerator) {
    return (ctx as any).legacyGenerator.generateExpression(expr)
  }

  // フォールバックが利用できない場合はエラーコメント
  return `/* Expression requires legacy generator: ${(expr as any).kind} */`
}
