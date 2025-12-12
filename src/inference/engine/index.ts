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
// Generators (Constraint Generation)
export {
  generalize,
  generateConstraintsForApplicativeApply,
  generateConstraintsForAssignmentExpression,
  generateConstraintsForBinaryOperation,
  generateConstraintsForConsExpression,
  generateConstraintsForConstructorExpression,
  // Dispatchers
  generateConstraintsForExpression,
  generateConstraintsForFunctionApplicationOperator,
  // Statement generators
  generateConstraintsForFunctionDeclaration,
  generateConstraintsForFunctorMap,
  generateConstraintsForIdentifier,
  generateConstraintsForImplBlock,
  generateConstraintsForImportDeclaration,
  generateConstraintsForIsExpression,
  generateConstraintsForListComprehension,
  generateConstraintsForListComprehensionSugar,
  // Expression generators
  generateConstraintsForLiteral,
  generateConstraintsForMonadBind,
  generateConstraintsForNullishCoalescing,
  generateConstraintsForRangeLiteral,
  generateConstraintsForRecordDestructuring,
  generateConstraintsForSignalExpression,
  generateConstraintsForSpreadExpression,
  generateConstraintsForStatement,
  generateConstraintsForStructDeclaration,
  generateConstraintsForStructDestructuring,
  generateConstraintsForTupleDestructuring,
  generateConstraintsForTypeAliasDeclaration,
  generateConstraintsForTypeDeclaration,
  generateConstraintsForVariableDeclaration,
  // Helpers
  instantiatePolymorphicType,
} from "./generators"
// Main API
export {
  getTypeOfNode,
  getTypeOfVariable,
  type InferResult,
  infer,
  inferExpression,
} from "./infer"

// Solver
export {
  type SolveResult,
  solveConstraints,
  solveConstraintsPartial,
} from "./solver"
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
