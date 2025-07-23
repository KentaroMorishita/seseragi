/**
 * ブラウザ環境用のモジュール解決システム（スタブ）
 * playgroundではimport機能は使用しないため空実装
 */

import type * as AST from "./ast"

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
   * ブラウザ環境では常にnullを返す（import機能無効）
   */
  resolve(_moduleName: string, _currentFilePath: string): ResolvedModule | null {
    console.warn("Module resolution is not supported in browser environment")
    return null
  }

  /**
   * キャッシュをクリア（開発時用）
   */
  clearCache(): void {
    this.cache.clear()
  }
}