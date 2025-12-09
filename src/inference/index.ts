/**
 * 型推論モジュール (Type Inference Module) for Seseragi Language
 *
 * 型推論に関連するクラス・関数をエクスポート
 */

// 型制約
export {
  ApplicativeApplyConstraint,
  ArrayAccessConstraint,
  FunctorMapConstraint,
  SubtypeConstraint,
  TypeConstraint,
} from "./constraints"
// 環境ユーティリティ
export { createInitialEnvironment } from "./environment"
// エラー
export { TypeInferenceError } from "./errors"

// 型置換
export { TypeSubstitution } from "./substitution"
// 型比較ユーティリティ
export {
  createFlattenedUnionType,
  isRecordSubset,
  isRecordSubtype,
  mergeRecordTypes,
  typesEqual,
} from "./type-comparison"
// 型フォーマッター
export {
  formatType,
  typeToCanonicalString,
  typeToString,
} from "./type-formatter"
// 型検査ユーティリティ
export {
  collectPolymorphicTypeVariables,
  getTypeName,
  isArrayType,
  isEitherType,
  isFunctionType,
  isListType,
  isMaybeType,
  isPromiseType,
  isSignalType,
  isTaskType,
  occursCheck,
  typeContainsVariable,
} from "./type-inspection"
// 型置換ユーティリティ
export {
  collectTypeVariables,
  getFreeTypeVariables,
  isTypeVariableBoundInEnv,
  substituteTypeVariables,
} from "./type-substitution-utils"
// 型変数
export { PolymorphicTypeVariable, TypeVariable } from "./type-variables"
