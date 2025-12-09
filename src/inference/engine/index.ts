/**
 * Inference Engine - 型推論エンジン
 *
 * TypeInferenceSystemクラスを置き換える関数ベースの型推論エンジン
 * 並列実装として開発し、テストで既存実装と比較検証
 */

// Context
export {
  addConstraint,
  addError,
  addSubtypeConstraint,
  bindType,
  cloneContext,
  createContextFromSystem,
  createEmptyContext,
  freshTypeVariable,
  getADTType,
  getImplMethod,
  getMethod,
  getNodeType,
  getStructType,
  getTypeAlias,
  type InferenceContext,
  lookupType,
  popScope,
  pushScope,
  registerADTType,
  registerImplMethod,
  registerMethod,
  registerStructType,
  setNodeType,
  setTypeAliases,
} from "./context"

// Type Alias Resolver
export {
  resolveTypeAlias,
  resolveTypeAliasRecursively,
} from "./type-alias-resolver"

// Unifier
export {
  isSubtype,
  type UnifyResult,
  unify,
  unifyOrThrow,
} from "./unifier"

// Solver
export {
  solveConstraints,
  solveConstraintsPartial,
  type SolveResult,
} from "./solver"

// Generators (Constraint Generation)
export {
  // Dispatchers
  generateConstraintsForExpression,
  generateConstraintsForStatement,
  // Expression generators
  generateConstraintsForLiteral,
  generateConstraintsForIdentifier,
  generateConstraintsForBinaryOperation,
  generateConstraintsForNullishCoalescing,
  generateConstraintsForIsExpression,
  generateConstraintsForFunctorMap,
  generateConstraintsForApplicativeApply,
  generateConstraintsForMonadBind,
  generateConstraintsForFunctionApplicationOperator,
  generateConstraintsForConstructorExpression,
  generateConstraintsForSignalExpression,
  generateConstraintsForAssignmentExpression,
  generateConstraintsForConsExpression,
  generateConstraintsForRangeLiteral,
  generateConstraintsForListComprehension,
  generateConstraintsForListComprehensionSugar,
  generateConstraintsForSpreadExpression,
  // Statement generators
  generateConstraintsForFunctionDeclaration,
  generateConstraintsForVariableDeclaration,
  generateConstraintsForTypeDeclaration,
  generateConstraintsForTypeAliasDeclaration,
  generateConstraintsForStructDeclaration,
  generateConstraintsForImplBlock,
  generateConstraintsForImportDeclaration,
  generateConstraintsForTupleDestructuring,
  generateConstraintsForRecordDestructuring,
  generateConstraintsForStructDestructuring,
  // Helpers
  instantiatePolymorphicType,
  generalize,
} from "./generators"

// Main API
export {
  infer,
  inferExpression,
  getTypeOfNode,
  getTypeOfVariable,
  type InferResult,
} from "./infer"
