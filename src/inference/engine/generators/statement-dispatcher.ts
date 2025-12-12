/**
 * 文の制約生成ディスパッチャ
 *
 * 文の種類に応じて適切なgeneratorを呼び出す
 */

import type * as AST from "../../../ast"
import type { InferenceContext } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"
import { generateConstraintsForFunctionDeclaration } from "./function-declaration"
import { generateConstraintsForImportDeclaration } from "./import-declaration"
import { generateConstraintsForRecordDestructuring } from "./record-destructuring"
import {
  generateConstraintsForImplBlock,
  generateConstraintsForStructDeclaration,
} from "./struct-declaration"
import { generateConstraintsForStructDestructuring } from "./struct-destructuring"
import { generateConstraintsForTupleDestructuring } from "./tuple-destructuring"
import {
  generateConstraintsForTypeAliasDeclaration,
  generateConstraintsForTypeDeclaration,
} from "./type-declaration"
import { generateConstraintsForVariableDeclaration } from "./variable-declaration"

/**
 * 文に対する制約を生成
 */
export function generateConstraintsForStatement(
  ctx: InferenceContext,
  statement: AST.Statement,
  env: Map<string, AST.Type>
): void {
  switch (statement.kind) {
    case "ExpressionStatement":
      generateConstraintsForExpression(
        ctx,
        (statement as AST.ExpressionStatement).expression,
        env
      )
      break

    case "VariableDeclaration":
      generateConstraintsForVariableDeclaration(
        ctx,
        statement as AST.VariableDeclaration,
        env
      )
      break

    case "FunctionDeclaration":
      generateConstraintsForFunctionDeclaration(
        ctx,
        statement as AST.FunctionDeclaration,
        env
      )
      break

    case "TypeDeclaration":
      generateConstraintsForTypeDeclaration(
        ctx,
        statement as AST.TypeDeclaration,
        env
      )
      break

    case "TypeAliasDeclaration":
      generateConstraintsForTypeAliasDeclaration(
        ctx,
        statement as AST.TypeAliasDeclaration,
        env
      )
      break

    case "StructDeclaration":
      generateConstraintsForStructDeclaration(
        ctx,
        statement as AST.StructDeclaration,
        env
      )
      break

    case "ImplBlock":
      generateConstraintsForImplBlock(ctx, statement as AST.ImplBlock, env)
      break

    case "ImportDeclaration":
      generateConstraintsForImportDeclaration(
        ctx,
        statement as AST.ImportDeclaration,
        env,
        ctx.currentFilePath
      )
      break

    case "TupleDestructuring":
      generateConstraintsForTupleDestructuring(
        ctx,
        statement as AST.TupleDestructuring,
        env
      )
      break

    case "RecordDestructuring":
      generateConstraintsForRecordDestructuring(
        ctx,
        statement as AST.RecordDestructuring,
        env
      )
      break

    case "StructDestructuring":
      generateConstraintsForStructDestructuring(
        ctx,
        statement as AST.StructDestructuring,
        env
      )
      break

    default:
      // 未実装の文タイプは警告なしでスキップ
      // （既存の型推論システムと同様の動作）
      break
  }
}
