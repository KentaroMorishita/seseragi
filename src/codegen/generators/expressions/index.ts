/**
 * Expression generators - 式生成モジュール
 */

// 配列
export {
  generateArrayAccess,
  generateArrayLiteral,
  generateRangeLiteral,
} from "./array"
export { generateBinaryOperation } from "./binary-operation"
// ブロック式
export { generateBlockExpression } from "./block"
export { generateBuiltinFunctionCall } from "./builtin-function-call"
// 条件式
export {
  generateConditionalExpression,
  generateTernaryExpression,
} from "./conditional"
// コンストラクタ
export { generateConstructorExpression } from "./constructor"
export { generateFunctionApplication } from "./function-application"
// 関数呼び出し
export { generateFunctionCall } from "./function-call"
export { generateIdentifier } from "./identifier"
// ラムダ式
export { generateLambdaExpression } from "./lambda"
// リスト
export { generateConsExpression, generateListSugar } from "./list"
export {
  generateListComprehension,
  generateListComprehensionSugar,
} from "./list-comprehension"
// 基本式
export { generateLiteral } from "./literal"
// パターンマッチ
export { generateMatchExpression } from "./match"
export { generateMethodCall } from "./method-call"
export { generateNullishCoalescing } from "./nullish-coalescing"
// パイプライン・モナド演算子
export {
  generateApplicativeApply,
  generateFoldMonoid,
  generateFunctionApplicationOperator,
  generateFunctorMap,
  generateMonadBind,
  generatePipeline,
  generateReversePipe,
} from "./pipeline"
// Promise
export {
  generatePromiseBlock,
  generateRejectExpression,
  generateResolveExpression,
  generateTryExpression,
} from "./promise"
// レコード
export { generateRecordAccess, generateRecordExpression } from "./record"
// シグナル
export {
  generateAssignmentExpression,
  generateSignalExpression,
} from "./signal"
// 構造体・スプレッド
export { generateSpreadExpression, generateStructExpression } from "./struct"
// テンプレート式
export { generateTemplateExpression } from "./template"
// タプル
export { generateTupleExpression } from "./tuple"
// 型操作
export { generateIsExpression, generateTypeAssertion } from "./type-operations"
// 演算子
export { generateUnaryOperation } from "./unary-operation"
