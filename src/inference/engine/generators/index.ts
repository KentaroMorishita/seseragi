/**
 * 制約生成モジュール
 *
 * 式・文の種類ごとに制約を生成する関数群
 */

// メインディスパッチャ
export { generateConstraintsForExpression } from "./dispatcher"
export { generateConstraintsForStatement } from "./statement-dispatcher"

// 個別のgenerator
export { generateConstraintsForLiteral } from "./literal"
export { generateConstraintsForIdentifier } from "./identifier"
export { generateConstraintsForBinaryOperation } from "./binary-operation"
export { generateConstraintsForUnaryOperation } from "./unary-operation"
export { generateConstraintsForConditional } from "./conditional"
export { generateConstraintsForLambdaExpression } from "./lambda"
export { generateConstraintsForTupleExpression } from "./tuple"
export { generateConstraintsForArrayLiteral } from "./array"
export { generateConstraintsForRecordAccess } from "./record-access"
export { generateConstraintsForArrayAccess } from "./array-access"
export { generateConstraintsForStructExpression } from "./struct-expression"
export { generateConstraintsForListSugar } from "./list-sugar"
export { generateConstraintsForPipeline } from "./pipeline"
export { generateConstraintsForBlockExpression } from "./block"
export { generateConstraintsForFunctionApplication } from "./function-application"
export { generateConstraintsForFunctionCall } from "./function-call"
export { generateConstraintsForMatchExpression } from "./match"
export { generateConstraintsForPattern } from "./pattern"
export { generateConstraintsForRecordExpression } from "./record-expression"
export { generateConstraintsForMethodCall } from "./method-call"
export { generateConstraintsForBuiltinFunctionCall } from "./builtin-function-call"
export { generateConstraintsForTernaryExpression } from "./ternary"
export { generateConstraintsForNullishCoalescing } from "./nullish-coalescing"
export { generateConstraintsForIsExpression } from "./is-expression"
export { generateConstraintsForFunctorMap } from "./functor-map"
export { generateConstraintsForApplicativeApply } from "./applicative-apply"
export { generateConstraintsForMonadBind } from "./monad-bind"
export { generateConstraintsForFunctionApplicationOperator } from "./function-application-operator"
export { generateConstraintsForConstructorExpression } from "./constructor-expression"
export { generateConstraintsForSignalExpression } from "./signal-expression"
export { generateConstraintsForAssignmentExpression } from "./assignment-expression"
export { generateConstraintsForConsExpression } from "./cons-expression"
export { generateConstraintsForRangeLiteral } from "./range-literal"
export {
  generateConstraintsForListComprehension,
  generateConstraintsForListComprehensionSugar,
} from "./list-comprehension"
export { generateConstraintsForSpreadExpression } from "./spread-expression"

// Statement generators
export { generateConstraintsForVariableDeclaration } from "./variable-declaration"
export { generateConstraintsForFunctionDeclaration } from "./function-declaration"
export {
  generateConstraintsForTypeDeclaration,
  generateConstraintsForTypeAliasDeclaration,
} from "./type-declaration"
export {
  generateConstraintsForStructDeclaration,
  generateConstraintsForImplBlock,
} from "./struct-declaration"
export { generateConstraintsForImportDeclaration } from "./import-declaration"
export { generateConstraintsForTupleDestructuring } from "./tuple-destructuring"
export { generateConstraintsForRecordDestructuring } from "./record-destructuring"
export { generateConstraintsForStructDestructuring } from "./struct-destructuring"

// ヘルパー
export { instantiatePolymorphicType, generalize } from "./helpers"
