/**
 * Import宣言の制約生成
 */

import * as AST from "../../../ast"
import { addError, type InferenceContext } from "../context"
import {
  generateConstraintsForMethodDeclaration,
  generateConstraintsForOperatorDeclaration,
} from "./struct-declaration"

// モジュールリゾルバ（Node.js環境でのみ利用）
let ModuleResolver: any
if (
  typeof globalThis !== "undefined" &&
  (globalThis as any).window === undefined
) {
  try {
    ModuleResolver = require("../../../module-resolver").ModuleResolver
  } catch {
    ModuleResolver = null
  }
}

/**
 * Import宣言の制約を生成
 */
export function generateConstraintsForImportDeclaration(
  ctx: InferenceContext,
  importDecl: AST.ImportDeclaration,
  env: Map<string, AST.Type>,
  currentFilePath: string = ""
): void {
  if (!ModuleResolver) {
    addError(
      ctx,
      "Module resolution is not available in this environment",
      importDecl.line,
      importDecl.column
    )
    return
  }

  const moduleResolver = new ModuleResolver()
  moduleResolver.clearCache()

  // moduleResolverをコンテキストに保存（codegenで再利用するため）
  ctx.moduleResolver = moduleResolver

  // モジュールを解決
  const resolvedModule = moduleResolver.resolve(
    importDecl.module,
    currentFilePath
  )

  if (!resolvedModule) {
    addError(
      ctx,
      `Cannot resolve module '${importDecl.module}'`,
      importDecl.line,
      importDecl.column
    )
    return
  }

  // Phase 1: ADT types first (no dependencies)
  for (const item of importDecl.items) {
    const exportedType = resolvedModule.exports.types.get(item.name)
    if (exportedType && exportedType.kind === "TypeDeclaration") {
      const importName = item.alias || item.name
      const typeDecl = exportedType as AST.TypeDeclaration

      // Union型を作成してADT型として環境に追加
      const unionTypes = typeDecl.fields.map(
        (field: AST.TypeField) =>
          new AST.PrimitiveType(field.name, field.line || 0, field.column || 0)
      )
      const unionType = new AST.UnionType(
        unionTypes,
        typeDecl.line,
        typeDecl.column
      )
      env.set(importName, unionType)

      // 各コンストラクタも環境に追加
      for (const field of typeDecl.fields) {
        const constructorType = new AST.PrimitiveType(
          field.name,
          field.line || 0,
          field.column || 0
        )
        env.set(field.name, constructorType)
      }
    }
  }

  // Phase 2: Type aliases first (functions may reference them)
  for (const item of importDecl.items) {
    const exportedType = resolvedModule.exports.types.get(item.name)
    if (exportedType && exportedType.kind === "TypeAliasDeclaration") {
      const importName = item.alias || item.name
      const aliasDecl = exportedType as AST.TypeAliasDeclaration
      env.set(importName, aliasDecl.aliasedType)
      // ctx.environmentにも登録（resolveTypeAliasで参照される）
      ctx.environment.set(importName, aliasDecl.aliasedType)
      // 型エイリアスとしても登録（型の統一で必要）
      ctx.typeAliases.set(importName, aliasDecl.aliasedType)
    }
  }

  // Phase 3: Structs
  for (const item of importDecl.items) {
    const exportedType = resolvedModule.exports.types.get(item.name)
    if (exportedType && exportedType.kind === "StructDeclaration") {
      const importName = item.alias || item.name
      const structDecl = exportedType as AST.StructDeclaration
      const structType = new AST.StructType(
        structDecl.name,
        structDecl.fields,
        structDecl.line,
        structDecl.column
      )
      env.set(importName, structType)

      // 対応するimpl定義も取得してメソッドを登録
      const implBlock = resolvedModule.exports.impls.get(structDecl.name)
      if (implBlock) {
        for (const method of implBlock.methods) {
          generateConstraintsForMethodDeclaration(ctx, method, env, structType)
        }
        for (const operator of implBlock.operators) {
          generateConstraintsForOperatorDeclaration(
            ctx,
            operator,
            env,
            structType
          )
        }
      }
    }
  }

  // Phase 4: Functions (may reference types from Phase 2)
  for (const item of importDecl.items) {
    const exportedFunction = resolvedModule.exports.functions.get(item.name)
    const exportedType = resolvedModule.exports.types.get(item.name)

    if (exportedFunction) {
      // 関数をインポート
      const funcType = createFunctionTypeFromDeclaration(exportedFunction)
      const importName = item.alias || item.name
      env.set(importName, funcType)
    } else if (exportedType) {
      // TypeAlias, Struct, ADTは既にPhase 1-3で処理済み
      // その他の型（存在する場合）のみここで処理
      if (
        exportedType.kind !== "TypeAliasDeclaration" &&
        exportedType.kind !== "TypeDeclaration" &&
        exportedType.kind !== "StructDeclaration"
      ) {
        const importName = item.alias || item.name
        env.set(importName, exportedType as AST.Type)
      }
    } else if (!resolvedModule.exports.types.get(item.name)) {
      // 関数でも型でもない場合のみエラー
      addError(
        ctx,
        `Module '${importDecl.module}' does not export '${item.name}'`,
        importDecl.line,
        importDecl.column
      )
    }
  }
}

/**
 * 関数宣言から関数型を作成
 */
function createFunctionTypeFromDeclaration(
  funcDecl: AST.FunctionDeclaration
): AST.Type {
  let resultType: AST.Type = funcDecl.returnType

  // 引数なしの場合は Unit -> ReturnType
  if (funcDecl.parameters.length === 0) {
    const unitType = new AST.PrimitiveType(
      "Unit",
      funcDecl.line,
      funcDecl.column
    )
    return new AST.FunctionType(
      unitType,
      resultType,
      funcDecl.line,
      funcDecl.column
    )
  }

  // カリー化形式で関数型を構築
  for (let i = funcDecl.parameters.length - 1; i >= 0; i--) {
    const param = funcDecl.parameters[i]
    if (param) {
      resultType = new AST.FunctionType(
        param.type,
        resultType,
        param.line,
        param.column
      )
    }
  }

  return resultType
}
