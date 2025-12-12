/**
 * Expression Dispatcher - 式生成のルーター
 *
 * 各式の種類に応じて適切なジェネレーターに振り分ける
 */

import type { Expression, Identifier, Literal } from "../../ast"
import type { CodeGenContext } from "../context"
import { sanitizeIdentifier } from "../helpers"
import {
  generateArrayAccess,
  generateArrayLiteral,
  generateRangeLiteral,
} from "./expressions/array"
import { generateBinaryOperation } from "./expressions/binary-operation"
import { generateBlockExpression } from "./expressions/block"
import { generateBuiltinFunctionCall } from "./expressions/builtin-function-call"
import {
  generateConditionalExpression,
  generateTernaryExpression,
} from "./expressions/conditional"
import { generateConstructorExpression } from "./expressions/constructor"
import { generateFunctionApplication } from "./expressions/function-application"
import { generateFunctionCall } from "./expressions/function-call"
import { generateLambdaExpression } from "./expressions/lambda"
import { generateConsExpression, generateListSugar } from "./expressions/list"
import {
  generateListComprehension,
  generateListComprehensionSugar,
} from "./expressions/list-comprehension"
// Expression generators
import { generateLiteral } from "./expressions/literal"
import { generateMatchExpression } from "./expressions/match"
import { generateMethodCall } from "./expressions/method-call"
import { generateNullishCoalescing } from "./expressions/nullish-coalescing"
import {
  generateApplicativeApply,
  generateFoldMonoid,
  generateFunctionApplicationOperator,
  generateFunctorMap,
  generateMonadBind,
  generatePipeline,
  generateReversePipe,
} from "./expressions/pipeline"
import {
  generatePromiseBlock,
  generateRejectExpression,
  generateResolveExpression,
  generateTryExpression,
} from "./expressions/promise"
import {
  generateRecordAccess,
  generateRecordExpression,
} from "./expressions/record"
import {
  generateAssignmentExpression,
  generateSignalExpression,
} from "./expressions/signal"
import {
  generateSpreadExpression,
  generateStructExpression,
} from "./expressions/struct"
import { generateTemplateExpression } from "./expressions/template"
import { generateTupleExpression } from "./expressions/tuple"
import {
  generateIsExpression,
  generateTypeAssertion,
} from "./expressions/type-operations"
import { generateUnaryOperation } from "./expressions/unary-operation"

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

    // 配列・リスト・タプル
    case "ArrayLiteral":
      return generateArrayLiteral(ctx, expr as any)

    case "ArrayAccess":
      return generateArrayAccess(ctx, expr as any)

    case "RangeLiteral":
      return generateRangeLiteral(ctx, expr as any)

    case "TupleExpression":
      return generateTupleExpression(ctx, expr as any)

    case "ListSugar":
      return generateListSugar(ctx, expr as any)

    case "ConsExpression":
      return generateConsExpression(ctx, expr as any)

    case "ListComprehension":
      return generateListComprehension(ctx, expr as any)

    case "ListComprehensionSugar":
      return generateListComprehensionSugar(ctx, expr as any)

    // 二項演算
    case "BinaryOperation":
      return generateBinaryOperation(ctx, expr as any)

    case "NullishCoalescingExpression":
      return generateNullishCoalescing(ctx, expr as any)

    // 関数呼び出し
    case "FunctionCall":
      return generateFunctionCall(ctx, expr as any)

    case "MethodCall":
      return generateMethodCall(ctx, expr as any)

    case "FunctionApplication":
      return generateFunctionApplication(ctx, expr as any)

    case "BuiltinFunctionCall":
      return generateBuiltinFunctionCall(ctx, expr as any)

    // パイプライン・モナド演算子
    case "Pipeline":
      return generatePipeline(ctx, expr as any)

    case "ReversePipe":
      return generateReversePipe(ctx, expr as any)

    case "FunctorMap":
      return generateFunctorMap(ctx, expr as any)

    case "ApplicativeApply":
      return generateApplicativeApply(ctx, expr as any)

    case "MonadBind":
      return generateMonadBind(ctx, expr as any)

    case "FoldMonoid":
      return generateFoldMonoid(ctx, expr as any)

    case "FunctionApplicationOperator":
      return generateFunctionApplicationOperator(ctx, expr as any)

    // コンストラクタ
    case "ConstructorExpression":
      return generateConstructorExpression(ctx, expr as any)

    // ブロック・ラムダ
    case "BlockExpression":
      return generateBlockExpression(ctx, expr as any)

    case "LambdaExpression":
      return generateLambdaExpression(ctx, expr as any)

    // レコード
    case "RecordExpression":
      return generateRecordExpression(ctx, expr as any)

    case "RecordAccess":
      return generateRecordAccess(ctx, expr as any)

    // 構造体・スプレッド
    case "StructExpression":
      return generateStructExpression(ctx, expr as any)

    case "SpreadExpression":
      return generateSpreadExpression(ctx, expr as any)

    // シグナル
    case "SignalExpression":
      return generateSignalExpression(ctx, expr as any)

    case "AssignmentExpression":
      return generateAssignmentExpression(ctx, expr as any)

    // 型操作
    case "TypeAssertion":
      return generateTypeAssertion(ctx, expr as any)

    case "IsExpression":
      return generateIsExpression(ctx, expr as any)

    // Promise関連
    case "PromiseBlock":
      return generatePromiseBlock(ctx, expr as any)

    case "ResolveExpression":
      return generateResolveExpression(ctx, expr as any)

    case "RejectExpression":
      return generateRejectExpression(ctx, expr as any)

    case "TryExpression":
      return generateTryExpression(ctx, expr as any)

    // パターンマッチ
    case "MatchExpression":
      return generateMatchExpression(ctx, expr as any)

    default:
      return `/* Unsupported expression: ${(expr as any).kind} */`
  }
}
