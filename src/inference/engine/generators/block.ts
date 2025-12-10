/**
 * ブロック式の制約生成
 */

import * as AST from "../../../ast"
import type { InferenceContext } from "../context"
import { generateConstraintsForExpression } from "./dispatcher"
import { generateConstraintsForStatement } from "./statement-dispatcher"

/**
 * ブロック式の制約を生成
 * { stmt1; stmt2; expr } のような式を処理
 */
export function generateConstraintsForBlockExpression(
  ctx: InferenceContext,
  block: AST.BlockExpression,
  env: Map<string, AST.Type>
): AST.Type {
  // ブロック用の新しい環境を作成（親環境をコピー）
  const blockEnv = new Map(env)

  // ブロック内のステートメントを処理
  for (const statement of block.statements) {
    generateConstraintsForStatement(ctx, statement, blockEnv)
  }

  // ブロックの型は戻り値式によって決まる
  if (block.returnExpression) {
    return generateConstraintsForExpression(ctx, block.returnExpression, blockEnv)
  }

  // 明示的な戻り値式がない場合、最後のステートメントが式ステートメントならその型を返す
  if (block.statements.length > 0) {
    const lastStatement = block.statements[block.statements.length - 1]
    if (lastStatement && lastStatement.kind === "ExpressionStatement") {
      const exprStmt = lastStatement as AST.ExpressionStatement
      return generateConstraintsForExpression(ctx, exprStmt.expression, blockEnv)
    }
  }

  // それ以外は Unit 型を返す
  return new AST.PrimitiveType("Unit", block.line, block.column)
}
