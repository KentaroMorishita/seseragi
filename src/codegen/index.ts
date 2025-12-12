/**
 * Seseragi コード生成モジュール
 *
 * SeseragiのASTをTypeScriptコードに変換する
 */

// 公開API
// コンテキスト
export {
  builtinFunctions,
  type CodeGenContext,
  type CodeGenOptions,
  createContext,
  decreaseIndent,
  defaultOptions,
  enterPromiseBlock,
  enterStructContext,
  exitPromiseBlock,
  exitStructContext,
  freshWildcard,
  getEnvironmentType,
  getIndent,
  getResolvedType,
  increaseIndent,
  isInsidePromiseBlock,
  registerFunctionType,
  registerStructMethod,
  registerStructOperator,
  registerTypeAlias,
  registerVariableType,
} from "./context"
// ジェネレーター
export { generateExpression } from "./generators"
export {
  generateApplicativeApply,
  generateArrayAccess,
  generateArrayLiteral,
  generateAssignmentExpression,
  generateBinaryOperation,
  generateBlockExpression,
  generateBuiltinFunctionCall,
  generateConditionalExpression,
  generateConsExpression,
  generateConstructorExpression,
  generateFoldMonoid,
  generateFunctionApplication,
  generateFunctionApplicationOperator,
  generateFunctionCall,
  generateFunctorMap,
  generateIdentifier,
  generateIsExpression,
  generateLambdaExpression,
  generateListComprehension,
  generateListComprehensionSugar,
  generateListSugar,
  generateLiteral,
  generateMatchExpression,
  generateMethodCall,
  generateMonadBind,
  generateNullishCoalescing,
  generatePipeline,
  generatePromiseBlock,
  generateRangeLiteral,
  generateRecordAccess,
  generateRecordExpression,
  generateRejectExpression,
  generateResolveExpression,
  generateReversePipe,
  generateSignalExpression,
  generateSpreadExpression,
  generateStructExpression,
  generateTemplateExpression,
  generateTernaryExpression,
  generateTryExpression,
  generateTupleExpression,
  generateTypeAssertion,
  generateUnaryOperation,
} from "./generators/expressions"
// Pattern generators
export {
  generatePatternBindings,
  generatePatternCondition,
} from "./generators/patterns"
// Statement generators
export {
  generateExpressionStatement,
  generateStatement,
  generateVariableDeclaration,
} from "./generators/statements"

// ヘルパー
export {
  escapeString,
  isArithmeticOperator,
  isBasicOperator,
  isBuiltinConstructor,
  isComparisonOperator,
  isJsReservedWord,
  isLogicalOperator,
  isMonadOperator,
  operatorToMethodName,
  safeIdentifier,
  sanitizeIdentifier,
} from "./helpers"
// 型ユーティリティ
export {
  findMatchingAliases,
  getTypeAliasName,
  isArrayType,
  isEitherType,
  isIntersectionTypeAlias,
  isIntType,
  isListType,
  isMaybeType,
  isPrimitiveType,
  isRecordLikeType,
  isSignalType,
  isTaskType,
  isTupleType,
  isTypeAlias,
  typeToStructuralString,
} from "./type-utils"
