/**
 * Seseragi コード生成モジュール
 *
 * SeseragiのASTをTypeScriptコードに変換する
 */

// 公開API
export { type CodeGenOptions, defaultOptions } from "./context"

// コンテキスト
export {
  type CodeGenContext,
  createContext,
  getIndent,
  increaseIndent,
  decreaseIndent,
  isInsidePromiseBlock,
  enterPromiseBlock,
  exitPromiseBlock,
  freshWildcard,
  enterStructContext,
  exitStructContext,
  registerStructMethod,
  registerStructOperator,
  registerTypeAlias,
  registerFunctionType,
  registerVariableType,
  getResolvedType,
  getEnvironmentType,
  builtinFunctions,
} from "./context"

// ヘルパー
export {
  operatorToMethodName,
  sanitizeIdentifier,
  isBuiltinConstructor,
  isBasicOperator,
  isComparisonOperator,
  isLogicalOperator,
  isArithmeticOperator,
  isMonadOperator,
  escapeString,
  isJsReservedWord,
  safeIdentifier,
} from "./helpers"

// 型ユーティリティ
export {
  isMaybeType,
  isEitherType,
  isTaskType,
  isSignalType,
  isListType,
  isArrayType,
  isTupleType,
  isIntType,
  isPrimitiveType,
  isTypeAlias,
  isIntersectionTypeAlias,
  isRecordLikeType,
  getTypeAliasName,
  typeToStructuralString,
  findMatchingAliases,
} from "./type-utils"

// ジェネレーター
export { generateExpression } from "./generators"
export {
  generateLiteral,
  generateIdentifier,
  generateUnaryOperation,
  generateConditionalExpression,
  generateTernaryExpression,
  generateTemplateExpression,
} from "./generators/expressions"
