/**
 * 制約生成モジュール
 *
 * 式・文の種類ごとに制約を生成する関数群
 */

export { generateConstraintsForApplicativeApply } from "./applicative-apply"
export { generateConstraintsForArrayLiteral } from "./array"
export { generateConstraintsForArrayAccess } from "./array-access"
export { generateConstraintsForAssignmentExpression } from "./assignment-expression"
export { generateConstraintsForBinaryOperation } from "./binary-operation"
export { generateConstraintsForBlockExpression } from "./block"
export { generateConstraintsForBuiltinFunctionCall } from "./builtin-function-call"
export { generateConstraintsForConditional } from "./conditional"
export { generateConstraintsForConsExpression } from "./cons-expression"
export { generateConstraintsForConstructorExpression } from "./constructor-expression"
// メインディスパッチャ
export { generateConstraintsForExpression } from "./dispatcher"
export { generateConstraintsForFunctionApplication } from "./function-application"
export { generateConstraintsForFunctionApplicationOperator } from "./function-application-operator"
export { generateConstraintsForFunctionCall } from "./function-call"
export { generateConstraintsForFunctionDeclaration } from "./function-declaration"
export { generateConstraintsForFunctorMap } from "./functor-map"
// ヘルパー
export { generalize, instantiatePolymorphicType } from "./helpers"
export { generateConstraintsForIdentifier } from "./identifier"
export { generateConstraintsForImportDeclaration } from "./import-declaration"
export { generateConstraintsForIsExpression } from "./is-expression"
export { generateConstraintsForLambdaExpression } from "./lambda"
export {
  generateConstraintsForListComprehension,
  generateConstraintsForListComprehensionSugar,
} from "./list-comprehension"
export { generateConstraintsForListSugar } from "./list-sugar"
// 個別のgenerator
export { generateConstraintsForLiteral } from "./literal"
export { generateConstraintsForMatchExpression } from "./match"
export { generateConstraintsForMethodCall } from "./method-call"
export { generateConstraintsForMonadBind } from "./monad-bind"
export { generateConstraintsForNullishCoalescing } from "./nullish-coalescing"
export { generateConstraintsForPattern } from "./pattern"
export { generateConstraintsForPipeline } from "./pipeline"
export { generateConstraintsForRangeLiteral } from "./range-literal"
export { generateConstraintsForRecordAccess } from "./record-access"
export { generateConstraintsForRecordDestructuring } from "./record-destructuring"
export { generateConstraintsForRecordExpression } from "./record-expression"
export { generateConstraintsForSignalExpression } from "./signal-expression"
export { generateConstraintsForSpreadExpression } from "./spread-expression"
export { generateConstraintsForStatement } from "./statement-dispatcher"
export {
  generateConstraintsForImplBlock,
  generateConstraintsForStructDeclaration,
} from "./struct-declaration"
export { generateConstraintsForStructDestructuring } from "./struct-destructuring"
export { generateConstraintsForStructExpression } from "./struct-expression"
export { generateConstraintsForTernaryExpression } from "./ternary"
export { generateConstraintsForTupleExpression } from "./tuple"
export { generateConstraintsForTupleDestructuring } from "./tuple-destructuring"
export {
  generateConstraintsForTypeAliasDeclaration,
  generateConstraintsForTypeDeclaration,
} from "./type-declaration"
export { generateConstraintsForUnaryOperation } from "./unary-operation"
// Statement generators
export { generateConstraintsForVariableDeclaration } from "./variable-declaration"
