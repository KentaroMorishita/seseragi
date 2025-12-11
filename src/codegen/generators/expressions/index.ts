/**
 * Expression generators - 式生成モジュール
 */

// 基本式
export { generateLiteral } from "./literal"
export { generateIdentifier } from "./identifier"

// 演算子
export { generateUnaryOperation } from "./unary-operation"

// 条件式
export {
  generateConditionalExpression,
  generateTernaryExpression,
} from "./conditional"

// テンプレート式
export { generateTemplateExpression } from "./template"

// TODO: 以下のジェネレーターを追加予定
// export { generateBinaryOperation } from "./binary-operation"
// export { generateFunctionCall } from "./function-call"
// export { generateMethodCall } from "./method-call"
// export { generateFunctionApplication } from "./function-application"
// export { generateBuiltinFunctionCall } from "./builtin-function-call"
// export { generateMatchExpression } from "./match"
// export { generatePipeline } from "./pipeline"
// export { generateLambdaExpression } from "./lambda"
// export { generateBlockExpression } from "./block"
// export { generateRecordExpression } from "./record"
// export { generateArrayLiteral, generateArrayAccess } from "./array"
// export { generateListSugar, generateConsExpression, generateListComprehension } from "./list"
// export { generateTupleExpression } from "./tuple"
// export { generateStructExpression } from "./struct"
// export { generateFunctorMap, generateApplicativeApply, generateMonadBind } from "./monad-operations"
// export { generatePromiseBlock, generateResolveExpression, generateRejectExpression } from "./promise"
// export { generateSignalExpression, generateAssignmentExpression } from "./signal"
