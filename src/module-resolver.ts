/**
 * モジュール解決システム
 * import文のモジュールパス解決とファイル読み込みを担当
 */

import { existsSync, readFileSync } from "node:fs"
import { dirname, join, resolve } from "node:path"
import type * as AST from "./ast"
import { Parser } from "./parser"

export interface ResolvedModule {
  path: string
  content: string
  ast: AST.Statement[]
  exports: ModuleExports
}

export interface ModuleExports {
  functions: Map<string, AST.FunctionDeclaration>
  types: Map<
    string,
    AST.TypeDeclaration | AST.TypeAliasDeclaration | AST.StructDeclaration
  >
}

export class ModuleResolver {
  private cache = new Map<string, ResolvedModule>()

  /**
   * モジュールパスを解決してファイルを読み込む
   */
  resolve(moduleName: string, currentFilePath: string): ResolvedModule | null {
    console.log(`🔍 Resolving module: ${moduleName} from ${currentFilePath}`)
    const resolvedPath = this.resolvePath(moduleName, currentFilePath)
    if (!resolvedPath) {
      console.log(`❌ Failed to resolve path for module: ${moduleName}`)
      return null
    }
    console.log(`✅ Resolved path: ${resolvedPath}`)

    // キャッシュチェック
    if (this.cache.has(resolvedPath)) {
      return this.cache.get(resolvedPath)!
    }

    try {
      const content = readFileSync(resolvedPath, "utf-8")
      const parser = new Parser(content)
      const parseResult = parser.parse()

      if (parseResult.errors && parseResult.errors.length > 0) {
        console.error(`Parse errors in ${resolvedPath}:`, parseResult.errors)
        return null
      }

      const statements = parseResult.statements || []
      const exports = this.extractExports(statements)

      const module: ResolvedModule = {
        path: resolvedPath,
        content,
        ast: statements,
        exports,
      }

      this.cache.set(resolvedPath, module)
      return module
    } catch (error) {
      console.error(`Failed to load module ${resolvedPath}:`, error)
      return null
    }
  }

  /**
   * モジュール名からファイルパスを解決
   */
  private resolvePath(
    moduleName: string,
    currentFilePath: string
  ): string | null {
    const currentDir = dirname(currentFilePath)

    // 相対パスの場合（./ or ../）
    if (moduleName.startsWith("./") || moduleName.startsWith("../")) {
      const relativePath = join(currentDir, moduleName)

      // .ssrg拡張子を試す
      const ssrgPath = `${relativePath}.ssrg`
      if (existsSync(ssrgPath)) {
        return resolve(ssrgPath)
      }

      // 拡張子がすでについている場合
      if (existsSync(relativePath)) {
        return resolve(relativePath)
      }

      return null
    }

    // 絶対パスの場合（将来的に標準ライブラリ対応）
    // 今は相対パスのみサポート
    return null
  }

  /**
   * ASTからエクスポート可能な定義を抽出
   */
  private extractExports(statements: AST.Statement[]): ModuleExports {
    const functions = new Map<string, AST.FunctionDeclaration>()
    const types = new Map<
      string,
      AST.TypeDeclaration | AST.TypeAliasDeclaration | AST.StructDeclaration
    >()

    for (const stmt of statements) {
      switch (stmt.kind) {
        case "FunctionDeclaration": {
          const funcDecl = stmt as AST.FunctionDeclaration
          functions.set(funcDecl.name, funcDecl)
          break
        }

        case "TypeDeclaration": {
          const typeDecl = stmt as AST.TypeDeclaration
          types.set(typeDecl.name, typeDecl)
          break
        }

        case "TypeAliasDeclaration": {
          const aliasDecl = stmt as AST.TypeAliasDeclaration
          types.set(aliasDecl.name, aliasDecl)
          break
        }

        case "StructDeclaration": {
          const structDecl = stmt as AST.StructDeclaration
          types.set(structDecl.name, structDecl)
          break
        }

        // let文は意図的にエクスポートしない（設計方針）
      }
    }

    return { functions, types }
  }

  /**
   * キャッシュをクリア（開発時用）
   */
  clearCache(): void {
    this.cache.clear()
  }
}
