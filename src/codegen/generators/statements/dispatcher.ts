/**
 * Statement Dispatcher - 文生成のルーター
 *
 * 各文の種類に応じて適切なジェネレーターに振り分ける
 */

import type { Statement } from "../../../ast"
import type { CodeGenContext } from "../../context"

/**
 * 文をTypeScriptコードに変換
 *
 * 文の種類に応じて適切なジェネレーターに振り分ける
 */
export function generateStatement(
  ctx: CodeGenContext,
  stmt: Statement
): string {
  switch (stmt.kind) {
    // ===================================================================
    // TODO: 以下の文は段階的に移行する
    // 現在は旧CodeGeneratorクラスへフォールバック
    // ===================================================================

    case "ExpressionStatement":
    case "VariableDeclaration":
    case "FunctionDeclaration":
    case "TypeDeclaration":
    case "TypeAliasDeclaration":
    case "TupleDestructuring":
    case "RecordDestructuring":
    case "StructDestructuring":
    case "StructDeclaration":
    case "ImplBlock":
    case "ImportDeclaration":
      // 旧実装へのフォールバック
      // これらは段階的に新モジュールに移行する
      return fallbackToLegacy(ctx, stmt)

    default:
      return `/* Unsupported statement: ${(stmt as any).kind} */`
  }
}

/**
 * 旧CodeGeneratorクラスへのフォールバック
 *
 * 新モジュールに移行されていない文は旧実装を使用する
 * このフォールバックは段階的に削除される
 */
function fallbackToLegacy(ctx: CodeGenContext, stmt: Statement): string {
  // 旧実装へのフォールバック
  // CodeGeneratorインスタンスがctxに存在する場合はそれを使用
  if ((ctx as any).legacyGenerator) {
    return (ctx as any).legacyGenerator.generateStatement(stmt)
  }

  // フォールバックが利用できない場合はエラーコメント
  return `/* Statement requires legacy generator: ${(stmt as any).kind} */`
}
