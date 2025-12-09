/**
 * 型推論エンジンのメインAPI
 *
 * TypeInferenceSystemクラスを置き換える関数ベースのインターフェース
 */

import * as AST from "../../ast"
import type { Program } from "../../parser"
import {
  addError,
  createEmptyContext,
  freshTypeVariable,
  type InferenceContext,
  setTypeAliases,
} from "./context"
import { generateConstraintsForStatement } from "./generators/statement-dispatcher"
import { solveConstraints } from "./solver"

/**
 * 推論結果
 */
export interface InferResult {
  /** 推論が成功したか */
  success: boolean
  /** 推論エラー */
  errors: Array<{
    message: string
    line: number
    column: number
    context?: string
  }>
  /** ノードと型のマッピング */
  nodeTypeMap: Map<AST.ASTNode, AST.Type>
  /** 環境（変数名 -> 型） */
  environment: Map<string, AST.Type>
  /** 推論に使用したコンテキスト */
  context: InferenceContext
}

/**
 * デフォルト環境を作成
 * 組み込み値（true, false など）を登録
 */
function createDefaultEnvironment(): Map<string, AST.Type> {
  const env = new Map<string, AST.Type>()

  // Bool literals
  env.set("true", new AST.PrimitiveType("Bool", 0, 0))
  env.set("false", new AST.PrimitiveType("Bool", 0, 0))

  // Unit
  env.set("()", new AST.PrimitiveType("Unit", 0, 0))

  return env
}

/**
 * プログラムの型を推論する
 *
 * @param program パース済みAST
 * @param typeAliases 型エイリアス定義（オプション）
 * @returns 推論結果
 */
export function infer(
  program: Program,
  typeAliases?: Map<string, AST.Type>
): InferResult {
  // コンテキストを初期化
  const ctx = createEmptyContext()

  // 型エイリアスを設定
  if (typeAliases) {
    setTypeAliases(ctx, typeAliases)
  }

  // デフォルト環境を作成
  const env = createDefaultEnvironment()

  // 各文に対して制約を生成
  for (const stmt of program.statements) {
    try {
      generateConstraintsForStatement(ctx, stmt, env)
    } catch (error) {
      if (error instanceof Error) {
        addError(ctx, error.message, stmt.line, stmt.column)
      } else {
        addError(ctx, String(error), stmt.line, stmt.column)
      }
    }
  }

  // 制約を解決
  let success = true
  try {
    const substitution = solveConstraints(ctx)

    // 解決結果を適用してnodeTypeMapを更新
    for (const [node, type] of ctx.nodeTypeMap) {
      ctx.nodeTypeMap.set(node, substitution.apply(type))
    }

    // 環境の型も解決
    for (const [name, type] of env) {
      env.set(name, substitution.apply(type))
    }
  } catch (error) {
    success = false
    if (error instanceof Error) {
      addError(ctx, error.message, 0, 0)
    }
  }

  // エラーがあれば失敗
  if (ctx.errors.length > 0) {
    success = false
  }

  return {
    success,
    errors: ctx.errors,
    nodeTypeMap: ctx.nodeTypeMap,
    environment: env,
    context: ctx,
  }
}

/**
 * 式の型を推論する
 *
 * @param expr 式AST
 * @param env 環境（オプション）
 * @returns 推論された型とエラー
 */
export function inferExpression(
  expr: AST.Expression,
  env?: Map<string, AST.Type>
): { type: AST.Type; errors: InferResult["errors"] } {
  const ctx = createEmptyContext()
  const environment = env ?? createDefaultEnvironment()

  // 式ラッパーを作成
  const { generateConstraintsForExpression } = require("./generators/dispatcher")
  const type = generateConstraintsForExpression(ctx, expr, environment)

  // 制約を解決
  try {
    const substitution = solveConstraints(ctx)
    return {
      type: substitution.apply(type),
      errors: ctx.errors,
    }
  } catch (error) {
    if (error instanceof Error) {
      addError(ctx, error.message, expr.line, expr.column)
    }
    return {
      type: freshTypeVariable(ctx, expr.line, expr.column),
      errors: ctx.errors,
    }
  }
}

/**
 * ノードの型を取得
 */
export function getTypeOfNode(
  result: InferResult,
  node: AST.ASTNode
): AST.Type | undefined {
  return result.nodeTypeMap.get(node)
}

/**
 * 変数の型を取得
 */
export function getTypeOfVariable(
  result: InferResult,
  name: string
): AST.Type | undefined {
  return result.environment.get(name)
}
