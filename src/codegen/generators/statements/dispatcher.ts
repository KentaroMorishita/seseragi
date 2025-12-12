/**
 * Statement Dispatcher - 文生成のルーター
 *
 * 各文の種類に応じて適切なジェネレーターに振り分ける
 */

import type {
  ExpressionStatement,
  FunctionDeclaration,
  ImplBlock,
  ImportDeclaration,
  RecordDestructuring,
  Statement,
  StructDeclaration,
  StructDestructuring,
  TupleDestructuring,
  TypeAliasDeclaration,
  TypeDeclaration,
  VariableDeclaration,
} from "../../../ast"
import type { CodeGenContext } from "../../context"
import {
  generateRecordDestructuring,
  generateStructDestructuring,
  generateTupleDestructuring,
} from "./destructuring"
import { generateExpressionStatement } from "./expression-statement"
import { generateFunctionDeclaration } from "./function-declaration"
import { generateImplBlock } from "./impl-block"
import { generateImportDeclaration } from "./import-declaration"
import { generateStructDeclaration } from "./struct-declaration"
import { generateTypeAliasDeclaration } from "./type-alias-declaration"
import { generateTypeDeclaration } from "./type-declaration"
import { generateVariableDeclaration } from "./variable-declaration"

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
    // 新モジュールに移行済み
    // ===================================================================

    case "ExpressionStatement":
      return generateExpressionStatement(ctx, stmt as ExpressionStatement)

    case "VariableDeclaration":
      return generateVariableDeclaration(ctx, stmt as VariableDeclaration)

    case "FunctionDeclaration":
      return generateFunctionDeclaration(ctx, stmt as FunctionDeclaration)

    case "ImportDeclaration":
      return generateImportDeclaration(ctx, stmt as ImportDeclaration)

    case "TypeDeclaration":
      return generateTypeDeclaration(ctx, stmt as TypeDeclaration)

    case "TypeAliasDeclaration":
      return generateTypeAliasDeclaration(ctx, stmt as TypeAliasDeclaration)

    case "StructDeclaration":
      return generateStructDeclaration(ctx, stmt as StructDeclaration)

    case "ImplBlock":
      return generateImplBlock(ctx, stmt as ImplBlock)

    case "TupleDestructuring":
      return generateTupleDestructuring(ctx, stmt as TupleDestructuring)

    case "RecordDestructuring":
      return generateRecordDestructuring(ctx, stmt as RecordDestructuring)

    case "StructDestructuring":
      return generateStructDestructuring(ctx, stmt as StructDestructuring)

    default:
      return `/* Unsupported statement: ${(stmt as any).kind} */`
  }
}
