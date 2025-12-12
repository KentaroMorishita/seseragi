/**
 * Import Declaration Generator - インポート宣言の生成（インライン化）
 */

import type {
  FunctionDeclaration as FunctionDeclarationAST,
  ImplBlock,
  ImportDeclaration,
  StructDeclaration,
  TypeAliasDeclaration,
  TypeDeclaration,
} from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateFunctionDeclaration } from "./function-declaration"
import { generateImplBlock } from "./impl-block"
import { generateStructDeclaration } from "./struct-declaration"
import { generateTypeAliasDeclaration } from "./type-alias-declaration"
import { generateTypeDeclaration } from "./type-declaration"

/**
 * インポート宣言をTypeScriptコードに変換（インライン化）
 *
 * インポートされたモジュールの内容を解決し、インポート項目をインライン化する。
 * これにより、実行時のモジュール解決を回避し、単一ファイル出力を実現する。
 *
 * 処理フロー:
 * 1. moduleResolverを使用してモジュールパスを解決
 * 2. 解決されたモジュールから各インポート項目を取得
 * 3. 項目の種類（関数/型/型エイリアス/struct）に応じて生成関数を呼び出し
 * 4. structの場合は対応するimplブロックも同時にインライン化
 *
 * 生成例:
 * ```typescript
 * // Inlined from module: ./math
 * function add(x: number) { ... }
 * type Result<T> = ...
 * interface Point { ... }
 * ```
 *
 * @param ctx - コード生成コンテキスト
 * @param stmt - インポート宣言AST
 * @returns 生成されたTypeScriptコード（複数行）
 */
export function generateImportDeclaration(
  ctx: CodeGenContext,
  stmt: ImportDeclaration
): string {
  // 型推論結果からモジュールリゾルバーを取得
  if (!ctx.typeInferenceResult?.moduleResolver) {
    return `// Import resolution not available: ${stmt.module}`
  }

  const resolver = ctx.typeInferenceResult.moduleResolver
  const currentFilePath = ctx.typeInferenceResult.currentFilePath || ""

  // モジュールパスを解決
  const resolvedModule = resolver.resolve(stmt.module, currentFilePath)

  if (!resolvedModule) {
    return `// Failed to resolve module: ${stmt.module}`
  }

  const lines: string[] = []
  lines.push(`// Inlined from module: ${stmt.module}`)

  // インポートされた各項目を処理
  for (const item of stmt.items) {
    // 関数として確認
    const funcDecl = resolvedModule.exports.functions.get(item.name)
    if (funcDecl) {
      const funcCode = generateFunctionDeclaration(
        ctx,
        funcDecl as FunctionDeclarationAST
      )
      lines.push(funcCode)
      continue
    }

    // 型として確認
    const typeDecl = resolvedModule.exports.types.get(item.name)
    if (typeDecl) {
      if (typeDecl.kind === "TypeDeclaration") {
        const typeCode = generateTypeDeclaration(
          ctx,
          typeDecl as TypeDeclaration
        )
        lines.push(typeCode)
      } else if (typeDecl.kind === "TypeAliasDeclaration") {
        const aliasCode = generateTypeAliasDeclaration(
          ctx,
          typeDecl as TypeAliasDeclaration
        )
        lines.push(aliasCode)
      } else if (typeDecl.kind === "StructDeclaration") {
        const structCode = generateStructDeclaration(
          ctx,
          typeDecl as StructDeclaration
        )
        lines.push(structCode)

        // structに対応するimplも生成
        const implBlock = resolvedModule.exports.impls.get(item.name)
        if (implBlock) {
          const implCode = generateImplBlock(ctx, implBlock as ImplBlock)
          lines.push(implCode)
        }
      }
      continue
    }

    // 見つからない場合
    lines.push(
      `// Warning: Could not find export '${item.name}' in module ${stmt.module}`
    )
  }

  return lines.join("\n")
}
