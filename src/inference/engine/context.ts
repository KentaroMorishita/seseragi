/**
 * InferenceContext - 型推論の状態を保持するコンテキスト
 *
 * TypeInferenceSystemクラスの `this.*` 依存を排除し、
 * 純粋関数ベースの型推論を実現するためのコンテキストオブジェクト
 */

import type * as AST from "../../ast"
import type {
  ApplicativeApplyConstraint,
  ArrayAccessConstraint,
  FunctorMapConstraint,
  SubtypeConstraint,
  TypeConstraint,
} from "../constraints"
import { TypeInferenceError } from "../errors"
import { TypeVariable } from "../type-variables"

/**
 * 型推論コンテキストのインターフェース
 * すべての状態をこのオブジェクトで管理する
 */
export interface InferenceContext {
  // === 型変数管理 ===
  nextVarId: number

  // === 制約蓄積 ===
  constraints: (
    | TypeConstraint
    | ArrayAccessConstraint
    | FunctorMapConstraint
    | ApplicativeApplyConstraint
  )[]
  subtypeConstraints: SubtypeConstraint[]

  // === エラー蓄積 ===
  errors: TypeInferenceError[]

  // === 型マッピング ===
  nodeTypeMap: Map<AST.ASTNode, AST.Type>

  // === 環境（型束縛） ===
  environment: Map<string, AST.Type>

  // === 型定義 ===
  typeAliases: Map<string, AST.Type>
  structTypes: Map<string, AST.StructType>
  adtTypes: Map<string, AST.TypeDeclaration>

  // === メソッド環境 ===
  methodEnvironment: Map<string, AST.MethodDeclaration>
  implMethods: Map<string, Map<string, AST.FunctionDeclaration>>

  // === モジュール解決 ===
  moduleResolver: any | null
  currentFilePath: string

  // === 処理中のプログラム ===
  currentProgram: AST.Program | null
}

/**
 * 空のInferenceContextを作成
 */
export function createEmptyContext(): InferenceContext {
  return {
    nextVarId: 1000,
    constraints: [],
    subtypeConstraints: [],
    errors: [],
    nodeTypeMap: new Map(),
    environment: new Map(),
    typeAliases: new Map(),
    structTypes: new Map(),
    adtTypes: new Map(),
    methodEnvironment: new Map(),
    implMethods: new Map(),
    moduleResolver: null,
    currentFilePath: "",
    currentProgram: null,
  }
}

/**
 * InferenceContextをコピー（immutable更新用）
 */
export function cloneContext(ctx: InferenceContext): InferenceContext {
  return {
    nextVarId: ctx.nextVarId,
    constraints: [...ctx.constraints],
    subtypeConstraints: [...ctx.subtypeConstraints],
    errors: [...ctx.errors],
    nodeTypeMap: new Map(ctx.nodeTypeMap),
    environment: new Map(ctx.environment),
    typeAliases: new Map(ctx.typeAliases),
    structTypes: new Map(ctx.structTypes),
    adtTypes: new Map(ctx.adtTypes),
    methodEnvironment: new Map(ctx.methodEnvironment),
    implMethods: new Map(ctx.implMethods),
    moduleResolver: ctx.moduleResolver,
    currentFilePath: ctx.currentFilePath,
    currentProgram: ctx.currentProgram,
  }
}

// ============================================================
// Context操作ヘルパー関数（mutating版 - パフォーマンス重視）
// ============================================================

/**
 * 新しい型変数を生成
 */
export function freshTypeVariable(
  ctx: InferenceContext,
  line: number,
  column: number
): TypeVariable {
  return new TypeVariable(ctx.nextVarId++, line, column)
}

/**
 * 型制約を追加
 */
export function addConstraint(
  ctx: InferenceContext,
  constraint:
    | TypeConstraint
    | ArrayAccessConstraint
    | FunctorMapConstraint
    | ApplicativeApplyConstraint
): void {
  ctx.constraints.push(constraint)
}

/**
 * サブタイプ制約を追加
 */
export function addSubtypeConstraint(
  ctx: InferenceContext,
  constraint: SubtypeConstraint
): void {
  ctx.subtypeConstraints.push(constraint)
}

/**
 * エラーを追加
 */
export function addError(
  ctx: InferenceContext,
  message: string,
  line: number,
  column: number,
  errorContext?: string
): void {
  ctx.errors.push(new TypeInferenceError(message, line, column, errorContext))
}

/**
 * ASTノードに型を設定
 */
export function setNodeType(
  ctx: InferenceContext,
  node: AST.ASTNode,
  type: AST.Type
): void {
  ctx.nodeTypeMap.set(node, type)
}

/**
 * ASTノードの型を取得
 */
export function getNodeType(
  ctx: InferenceContext,
  node: AST.ASTNode
): AST.Type | undefined {
  return ctx.nodeTypeMap.get(node)
}

/**
 * 環境に型束縛を追加
 */
export function bindType(
  ctx: InferenceContext,
  name: string,
  type: AST.Type
): void {
  ctx.environment.set(name, type)
}

/**
 * 環境から型を取得
 */
export function lookupType(
  ctx: InferenceContext,
  name: string
): AST.Type | undefined {
  return ctx.environment.get(name)
}

/**
 * 新しいスコープを作成（環境のコピー）
 */
export function pushScope(ctx: InferenceContext): Map<string, AST.Type> {
  const oldEnv = ctx.environment
  ctx.environment = new Map(ctx.environment)
  return oldEnv
}

/**
 * スコープを復元
 */
export function popScope(
  ctx: InferenceContext,
  oldEnv: Map<string, AST.Type>
): void {
  ctx.environment = oldEnv
}

/**
 * 型エイリアスを設定
 */
export function setTypeAliases(
  ctx: InferenceContext,
  aliases: Map<string, AST.Type>
): void {
  ctx.typeAliases = aliases
}

/**
 * 型エイリアスを取得
 */
export function getTypeAlias(
  ctx: InferenceContext,
  name: string
): AST.Type | undefined {
  return ctx.typeAliases.get(name)
}

/**
 * 構造体型を登録
 */
export function registerStructType(
  ctx: InferenceContext,
  name: string,
  structType: AST.StructType
): void {
  ctx.structTypes.set(name, structType)
}

/**
 * 構造体型を取得
 */
export function getStructType(
  ctx: InferenceContext,
  name: string
): AST.StructType | undefined {
  return ctx.structTypes.get(name)
}

/**
 * ADT型を登録
 */
export function registerADTType(
  ctx: InferenceContext,
  name: string,
  typeDecl: AST.TypeDeclaration
): void {
  ctx.adtTypes.set(name, typeDecl)
}

/**
 * ADT型を取得
 */
export function getADTType(
  ctx: InferenceContext,
  name: string
): AST.TypeDeclaration | undefined {
  return ctx.adtTypes.get(name)
}

/**
 * メソッドを環境に登録
 */
export function registerMethod(
  ctx: InferenceContext,
  key: string,
  method: AST.MethodDeclaration
): void {
  ctx.methodEnvironment.set(key, method)
}

/**
 * メソッドを取得
 */
export function getMethod(
  ctx: InferenceContext,
  key: string
): AST.MethodDeclaration | undefined {
  return ctx.methodEnvironment.get(key)
}

/**
 * implメソッドを登録
 */
export function registerImplMethod(
  ctx: InferenceContext,
  typeName: string,
  methodName: string,
  func: AST.FunctionDeclaration
): void {
  let methods = ctx.implMethods.get(typeName)
  if (!methods) {
    methods = new Map()
    ctx.implMethods.set(typeName, methods)
  }
  methods.set(methodName, func)
}

/**
 * implメソッドを取得
 */
export function getImplMethod(
  ctx: InferenceContext,
  typeName: string,
  methodName: string
): AST.FunctionDeclaration | undefined {
  return ctx.implMethods.get(typeName)?.get(methodName)
}

// ============================================================
// TypeInferenceSystemからのマイグレーションユーティリティ
// ============================================================

/**
 * TypeInferenceSystemの状態からInferenceContextを作成
 * （移行期間中の互換性のため）
 */
export function createContextFromSystem(system: {
  constraints: InferenceContext["constraints"]
  subtypeConstraints: SubtypeConstraint[]
  errors: TypeInferenceError[]
  nodeTypeMap: Map<AST.ASTNode, AST.Type>
  environment: Map<string, AST.Type>
  typeAliases: Map<string, AST.Type>
  nextVarId: number
}): InferenceContext {
  return {
    nextVarId: system.nextVarId,
    constraints: system.constraints,
    subtypeConstraints: system.subtypeConstraints,
    errors: system.errors,
    nodeTypeMap: system.nodeTypeMap,
    environment: system.environment,
    typeAliases: system.typeAliases,
    structTypes: new Map(),
    adtTypes: new Map(),
    methodEnvironment: new Map(),
    implMethods: new Map(),
    moduleResolver: null,
    currentFilePath: "",
    currentProgram: null,
  }
}
