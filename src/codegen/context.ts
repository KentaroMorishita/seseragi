/**
 * CodeGenContext - コード生成の状態を保持するコンテキスト
 *
 * CodeGeneratorクラスの `this.*` 依存を排除し、
 * 純粋関数ベースのコード生成を実現するためのコンテキストオブジェクト
 */

import type { Type } from "../ast"
import type { InferResult } from "../inference/engine/infer"
import type { UsageAnalysis } from "../usage-analyzer"

/**
 * コード生成オプション
 */
export interface CodeGenOptions {
  indent?: string
  useArrowFunctions?: boolean
  generateComments?: boolean
  runtimeMode?: "embedded" | "import"
  filePath?: string
  typeInferenceResult?: InferResult
}

/**
 * デフォルトオプション
 */
export const defaultOptions: CodeGenOptions = {
  indent: "  ",
  useArrowFunctions: true,
  generateComments: false,
  runtimeMode: "import",
}

/**
 * コード生成コンテキスト
 */
export interface CodeGenContext {
  // === オプション ===
  options: CodeGenOptions

  // === インデント管理 ===
  indentLevel: number

  // === カウンター ===
  wildcardCounter: number
  promiseBlockDepth: number

  // === 蓄積マップ ===
  structMethods: Map<string, Set<string>>
  structOperators: Map<string, Set<string>>
  typeAliases: Map<string, Type>
  functionTypes: Map<string, Type>
  variableTypes: Map<string, string>
  variableAliases: Map<string, string[]>

  // === 外部データ ===
  usageAnalysis: UsageAnalysis | null
  typeInferenceResult: InferResult | null

  // === スコープ状態 ===
  filePrefix: string
  currentStructContext: string | null
  currentFunctionTypeParams: any[]
}

/**
 * ビルトイン関数のマッピング
 */
export const builtinFunctions: Record<string, string> = {
  print: "ssrgPrint",
  putStrLn: "ssrgPutStrLn",
  toString: "ssrgToString",
  toInt: "ssrgToInt",
  toFloat: "ssrgToFloat",
  show: "ssrgShow",
  head: "headList",
  tail: "tailList",
  resolve: "resolve",
  run: "ssrgRun",
  tryRun: "ssrgTryRun",
}

/**
 * ファイルパスからプレフィックスを生成
 */
function generateFilePrefix(filePath: string): string {
  let hash = 0
  for (let i = 0; i < filePath.length; i++) {
    const char = filePath.charCodeAt(i)
    hash = (hash << 5) - hash + char
    hash = hash & hash
  }
  return `_${Math.abs(hash).toString(36)}`
}

/**
 * 空のCodeGenContextを作成
 */
export function createContext(options: CodeGenOptions = {}): CodeGenContext {
  const opts = { ...defaultOptions, ...options }
  return {
    options: opts,
    indentLevel: 0,
    wildcardCounter: 1,
    promiseBlockDepth: 0,
    structMethods: new Map(),
    structOperators: new Map(),
    typeAliases: new Map(),
    functionTypes: new Map(),
    variableTypes: new Map(),
    variableAliases: new Map(),
    usageAnalysis: null,
    typeInferenceResult: opts.typeInferenceResult ?? null,
    filePrefix: generateFilePrefix(opts.filePath || "unknown"),
    currentStructContext: null,
    currentFunctionTypeParams: [],
  }
}

// ============================================================
// Context操作ヘルパー関数
// ============================================================

/**
 * 現在のインデント文字列を取得
 */
export function getIndent(ctx: CodeGenContext): string {
  return (ctx.options.indent || "  ").repeat(ctx.indentLevel)
}

/**
 * インデントレベルを増やす
 */
export function increaseIndent(ctx: CodeGenContext): void {
  ctx.indentLevel++
}

/**
 * インデントレベルを減らす
 */
export function decreaseIndent(ctx: CodeGenContext): void {
  ctx.indentLevel = Math.max(0, ctx.indentLevel - 1)
}

/**
 * promiseブロック内かどうかを判定
 */
export function isInsidePromiseBlock(ctx: CodeGenContext): boolean {
  return ctx.promiseBlockDepth > 0
}

/**
 * promiseブロックに入る
 */
export function enterPromiseBlock(ctx: CodeGenContext): void {
  ctx.promiseBlockDepth++
}

/**
 * promiseブロックから出る
 */
export function exitPromiseBlock(ctx: CodeGenContext): void {
  ctx.promiseBlockDepth = Math.max(0, ctx.promiseBlockDepth - 1)
}

/**
 * ワイルドカード変数名を生成
 */
export function freshWildcard(ctx: CodeGenContext): string {
  return `_wild${ctx.wildcardCounter++}`
}

/**
 * struct context に入る
 */
export function enterStructContext(ctx: CodeGenContext, name: string): void {
  ctx.currentStructContext = name
}

/**
 * struct context から出る
 */
export function exitStructContext(ctx: CodeGenContext): void {
  ctx.currentStructContext = null
}

/**
 * struct メソッドを登録
 */
export function registerStructMethod(
  ctx: CodeGenContext,
  structName: string,
  methodName: string
): void {
  if (!ctx.structMethods.has(structName)) {
    ctx.structMethods.set(structName, new Set())
  }
  ctx.structMethods.get(structName)!.add(methodName)
}

/**
 * struct 演算子を登録
 */
export function registerStructOperator(
  ctx: CodeGenContext,
  structName: string,
  operator: string
): void {
  if (!ctx.structOperators.has(structName)) {
    ctx.structOperators.set(structName, new Set())
  }
  ctx.structOperators.get(structName)!.add(operator)
}

/**
 * 型エイリアスを登録
 */
export function registerTypeAlias(
  ctx: CodeGenContext,
  name: string,
  type: Type
): void {
  ctx.typeAliases.set(name, type)
}

/**
 * 関数型を登録
 */
export function registerFunctionType(
  ctx: CodeGenContext,
  name: string,
  type: Type
): void {
  ctx.functionTypes.set(name, type)
}

/**
 * 変数型情報を登録
 */
export function registerVariableType(
  ctx: CodeGenContext,
  name: string,
  typeStr: string,
  aliases?: string[]
): void {
  ctx.variableTypes.set(name, typeStr)
  if (aliases && aliases.length > 0) {
    ctx.variableAliases.set(name, aliases)
  }
}

/**
 * 型推論結果からノードの型を取得
 */
export function getResolvedType(
  ctx: CodeGenContext,
  node: any
): Type | undefined {
  return ctx.typeInferenceResult?.nodeTypeMap?.get(node)
}

/**
 * 環境から変数の型を取得
 */
export function getEnvironmentType(
  ctx: CodeGenContext,
  name: string
): Type | undefined {
  return ctx.typeInferenceResult?.environment?.get(name)
}
