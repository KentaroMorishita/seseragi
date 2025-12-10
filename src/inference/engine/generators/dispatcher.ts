/**
 * 制約生成ディスパッチャ
 *
 * 式の種類に応じて適切なgeneratorを呼び出す
 */

import type * as AST from "../../../ast"
import { TypeConstraint } from "../../constraints"
import {
  addConstraint,
  addError,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForArrayAccess } from "./array-access"
import { generateConstraintsForArrayLiteral } from "./array"
import { generateConstraintsForBinaryOperation } from "./binary-operation"
import { generateConstraintsForBlockExpression } from "./block"
import { generateConstraintsForConditional } from "./conditional"
import { generateConstraintsForFunctionApplication } from "./function-application"
import { generateConstraintsForFunctionCall } from "./function-call"
import { generateConstraintsForIdentifier } from "./identifier"
import { generateConstraintsForMatchExpression } from "./match"
import { generateConstraintsForLambdaExpression } from "./lambda"
import { generateConstraintsForRecordExpression } from "./record-expression"
import { generateConstraintsForMethodCall } from "./method-call"
import { generateConstraintsForListSugar } from "./list-sugar"
import { generateConstraintsForLiteral } from "./literal"
import { generateConstraintsForPipeline } from "./pipeline"
import { generateConstraintsForRecordAccess } from "./record-access"
import { generateConstraintsForStructExpression } from "./struct-expression"
import { generateConstraintsForTupleExpression } from "./tuple"
import { generateConstraintsForUnaryOperation } from "./unary-operation"
import { generateConstraintsForBuiltinFunctionCall } from "./builtin-function-call"
import { generateConstraintsForTernaryExpression } from "./ternary"
import { generateConstraintsForNullishCoalescing } from "./nullish-coalescing"
import { generateConstraintsForIsExpression } from "./is-expression"
import { generateConstraintsForFunctorMap } from "./functor-map"
import { generateConstraintsForApplicativeApply } from "./applicative-apply"
import { generateConstraintsForMonadBind } from "./monad-bind"
import { generateConstraintsForFunctionApplicationOperator } from "./function-application-operator"
import { generateConstraintsForConstructorExpression } from "./constructor-expression"
import { generateConstraintsForSignalExpression } from "./signal-expression"
import { generateConstraintsForAssignmentExpression } from "./assignment-expression"
import { generateConstraintsForConsExpression } from "./cons-expression"
import { generateConstraintsForRangeLiteral } from "./range-literal"
import {
  generateConstraintsForListComprehension,
  generateConstraintsForListComprehensionSugar,
} from "./list-comprehension"
import { generateConstraintsForSpreadExpression } from "./spread-expression"
import {
  generateConstraintsForPromiseBlock,
  generateConstraintsForTryExpression,
  generateConstraintsForResolveExpression,
  generateConstraintsForRejectExpression,
} from "./promise"
import { generateConstraintsForTemplateExpression } from "./template-expression"
import { generateConstraintsForTypeAssertion } from "./type-assertion"

/**
 * 式に対する制約を生成し、その式の型を返す
 */
export function generateConstraintsForExpression(
  ctx: InferenceContext,
  expr: AST.Expression,
  env: Map<string, AST.Type>,
  expectedType?: AST.Type
): AST.Type {
  let resultType: AST.Type

  switch (expr.kind) {
    case "Literal":
      resultType = generateConstraintsForLiteral(ctx, expr as AST.Literal)
      break

    case "TemplateExpression":
      resultType = generateConstraintsForTemplateExpression(
        ctx,
        expr as AST.TemplateExpression,
        env
      )
      break

    case "Identifier":
      resultType = generateConstraintsForIdentifier(
        ctx,
        expr as AST.Identifier,
        env
      )
      break

    case "BinaryOperation":
      resultType = generateConstraintsForBinaryOperation(
        ctx,
        expr as AST.BinaryOperation,
        env
      )
      break

    case "UnaryOperation":
      resultType = generateConstraintsForUnaryOperation(
        ctx,
        expr as AST.UnaryOperation,
        env
      )
      break

    case "ConditionalExpression":
      resultType = generateConstraintsForConditional(
        ctx,
        expr as AST.ConditionalExpression,
        env
      )
      break

    case "LambdaExpression":
      resultType = generateConstraintsForLambdaExpression(
        ctx,
        expr as AST.LambdaExpression,
        env
      )
      break

    case "TupleExpression":
      resultType = generateConstraintsForTupleExpression(
        ctx,
        expr as AST.TupleExpression,
        env
      )
      break

    case "ArrayLiteral":
      resultType = generateConstraintsForArrayLiteral(
        ctx,
        expr as AST.ArrayLiteral,
        env
      )
      break

    case "RecordAccess":
      resultType = generateConstraintsForRecordAccess(
        ctx,
        expr as AST.RecordAccess,
        env
      )
      break

    case "ArrayAccess":
      resultType = generateConstraintsForArrayAccess(
        ctx,
        expr as AST.ArrayAccess,
        env
      )
      break

    case "StructExpression":
      resultType = generateConstraintsForStructExpression(
        ctx,
        expr as AST.StructExpression,
        env
      )
      break

    case "ListSugar":
      resultType = generateConstraintsForListSugar(
        ctx,
        expr as AST.ListSugar,
        env
      )
      break

    case "Pipeline":
      resultType = generateConstraintsForPipeline(
        ctx,
        expr as AST.Pipeline,
        env
      )
      break

    case "BlockExpression":
      resultType = generateConstraintsForBlockExpression(
        ctx,
        expr as AST.BlockExpression,
        env
      )
      break

    case "FunctionApplication":
      resultType = generateConstraintsForFunctionApplication(
        ctx,
        expr as AST.FunctionApplication,
        env
      )
      break

    case "FunctionCall":
      resultType = generateConstraintsForFunctionCall(
        ctx,
        expr as AST.FunctionCall,
        env
      )
      break

    case "MatchExpression":
      resultType = generateConstraintsForMatchExpression(
        ctx,
        expr as AST.MatchExpression,
        env
      )
      break

    case "RecordExpression":
      resultType = generateConstraintsForRecordExpression(
        ctx,
        expr as AST.RecordExpression,
        env,
        expectedType
      )
      break

    case "MethodCall":
      resultType = generateConstraintsForMethodCall(
        ctx,
        expr as AST.MethodCall,
        env
      )
      break

    case "BuiltinFunctionCall":
      resultType = generateConstraintsForBuiltinFunctionCall(
        ctx,
        expr as AST.BuiltinFunctionCall,
        env
      )
      break

    case "TernaryExpression":
      resultType = generateConstraintsForTernaryExpression(
        ctx,
        expr as AST.TernaryExpression,
        env,
        expectedType
      )
      break

    case "NullishCoalescingExpression":
      resultType = generateConstraintsForNullishCoalescing(
        ctx,
        expr as AST.NullishCoalescingExpression,
        env
      )
      break

    case "IsExpression":
      resultType = generateConstraintsForIsExpression(
        ctx,
        expr as AST.IsExpression,
        env
      )
      break

    case "TypeAssertion":
      resultType = generateConstraintsForTypeAssertion(
        ctx,
        expr as AST.TypeAssertion,
        env
      )
      break

    case "FunctorMap":
      resultType = generateConstraintsForFunctorMap(
        ctx,
        expr as AST.FunctorMap,
        env
      )
      break

    case "ApplicativeApply":
      resultType = generateConstraintsForApplicativeApply(
        ctx,
        expr as AST.ApplicativeApply,
        env
      )
      break

    case "MonadBind":
      resultType = generateConstraintsForMonadBind(
        ctx,
        expr as AST.MonadBind,
        env
      )
      break

    case "FunctionApplicationOperator":
      resultType = generateConstraintsForFunctionApplicationOperator(
        ctx,
        expr as AST.FunctionApplicationOperator,
        env
      )
      break

    case "ConstructorExpression":
      resultType = generateConstraintsForConstructorExpression(
        ctx,
        expr as AST.ConstructorExpression,
        env
      )
      break

    case "SignalExpression":
      resultType = generateConstraintsForSignalExpression(
        ctx,
        expr as AST.SignalExpression,
        env
      )
      break

    case "AssignmentExpression":
      resultType = generateConstraintsForAssignmentExpression(
        ctx,
        expr as AST.AssignmentExpression,
        env
      )
      break

    case "ConsExpression":
      resultType = generateConstraintsForConsExpression(
        ctx,
        expr as AST.ConsExpression,
        env
      )
      break

    case "RangeLiteral":
      resultType = generateConstraintsForRangeLiteral(
        ctx,
        expr as AST.RangeLiteral,
        env
      )
      break

    case "ListComprehension":
      resultType = generateConstraintsForListComprehension(
        ctx,
        expr as AST.ListComprehension,
        env
      )
      break

    case "ListComprehensionSugar":
      resultType = generateConstraintsForListComprehensionSugar(
        ctx,
        expr as AST.ListComprehensionSugar,
        env
      )
      break

    case "SpreadExpression":
      resultType = generateConstraintsForSpreadExpression(
        ctx,
        expr as AST.SpreadExpression,
        env
      )
      break

    case "PromiseBlock":
      resultType = generateConstraintsForPromiseBlock(
        ctx,
        expr as AST.PromiseBlock,
        env
      )
      break

    case "TryExpression":
      resultType = generateConstraintsForTryExpression(
        ctx,
        expr as AST.TryExpression,
        env
      )
      break

    case "ResolveExpression":
      resultType = generateConstraintsForResolveExpression(
        ctx,
        expr as AST.ResolveExpression,
        env
      )
      break

    case "RejectExpression":
      resultType = generateConstraintsForRejectExpression(
        ctx,
        expr as AST.RejectExpression,
        env
      )
      break

    default:
      // 未実装の式タイプ
      addError(
        ctx,
        `[Engine] Unsupported expression type: ${expr.kind}`,
        expr.line,
        expr.column
      )
      // フォールバック: 新しい型変数を返す
      resultType = freshTypeVariable(ctx, expr.line, expr.column)
  }

  // ノードタイプマップに記録
  ctx.nodeTypeMap.set(expr, resultType)

  // 期待される型が指定されていれば制約を追加
  if (expectedType) {
    addConstraint(
      ctx,
      new TypeConstraint(resultType, expectedType, expr.line, expr.column)
    )
  }

  return resultType
}
