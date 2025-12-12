/**
 * Destructuring Statement Generators - 分解代入文の生成
 */

import type {
  IdentifierPattern,
  RecordDestructuring,
  StructDestructuring,
  TupleDestructuring,
} from "../../../ast"
import type { CodeGenContext } from "../../context"
import { freshWildcard, getIndent } from "../../context"
import { sanitizeIdentifier } from "../../helpers"
import { generateExpression } from "../dispatcher"

/**
 * タプル分解代入をTypeScriptコードに変換
 *
 * 基本形式:
 * - `const [a, b] = expr.elements;`
 *
 * ワイルドカードサポート:
 * - `const [a, _] = expr.elements;` → `const [a, _wild1] = expr.elements;`
 *
 * @example
 * Seseragi: let (x, y) = point
 * TypeScript: const [x, y] = point.elements;
 */
export function generateTupleDestructuring(
  ctx: CodeGenContext,
  stmt: TupleDestructuring
): string {
  const indent = getIndent(ctx)
  const valueExpr = generateExpression(ctx, stmt.initializer)

  // パターンから変数名を抽出
  const patterns = stmt.pattern.patterns
  const bindings = patterns.map((pattern) => {
    if (pattern.kind === "IdentifierPattern") {
      const idPattern = pattern as IdentifierPattern
      // ワイルドカード "_" は一意な変数名に変換
      if (idPattern.name === "_") {
        return freshWildcard(ctx)
      }
      return sanitizeIdentifier(idPattern.name)
    }
    if (pattern.kind === "WildcardPattern") {
      return freshWildcard(ctx)
    }
    // その他のパターンは未サポート（必要に応じて拡張）
    throw new Error(
      `Unsupported pattern in tuple destructuring: ${pattern.kind}`
    )
  })

  // TypeScript分解代入生成
  return `${indent}const [${bindings.join(", ")}] = ${valueExpr}.elements;`
}

/**
 * レコード分解代入をTypeScriptコードに変換
 *
 * 基本形式:
 * - `const { x, y } = expr;`
 *
 * エイリアスサポート:
 * - `const { x: posX, y: posY } = expr;`
 *
 * ワイルドカードサポート:
 * - フィールド名が "_" の場合はスキップ
 *
 * @example
 * Seseragi: let { x, y } = point
 * TypeScript: const { x, y } = point;
 */
export function generateRecordDestructuring(
  ctx: CodeGenContext,
  stmt: RecordDestructuring
): string {
  const indent = getIndent(ctx)
  const valueExpr = generateExpression(ctx, stmt.initializer)

  // パターンからフィールドマッピングを抽出
  const fields = stmt.pattern.fields
  const bindings = fields
    .map((field) => {
      const fieldName = field.fieldName

      // ワイルドカードフィールドはスキップ
      if (fieldName === "_") {
        return null
      }

      // エイリアスがある場合: { x: posX }
      if (field.alias) {
        const alias = sanitizeIdentifier(field.alias)
        return `${fieldName}: ${alias}`
      }

      // ネストされたパターンがある場合（将来拡張）
      if (field.pattern) {
        // TODO: ネストされたパターンのサポート
        throw new Error("Nested patterns in record destructuring not supported")
      }

      // 通常のフィールド: { x }
      return sanitizeIdentifier(fieldName)
    })
    .filter((b) => b !== null)

  // 全フィールドがワイルドカードの場合は空の分解代入
  if (bindings.length === 0) {
    // 副作用のために値を評価するだけ
    return `${indent}${valueExpr};`
  }

  // TypeScript分解代入生成
  return `${indent}const { ${bindings.join(", ")} } = ${valueExpr};`
}

/**
 * 構造体分解代入をTypeScriptコードに変換
 *
 * 基本形式:
 * - `const { field1, field2 } = expr;`
 *
 * 構造体名は型情報として使われるが、実行時にはレコード分解と同じ扱い
 *
 * エイリアスサポート:
 * - `const { field1: alias1, field2: alias2 } = expr;`
 *
 * ワイルドカードサポート:
 * - フィールド名が "_" の場合はスキップ
 *
 * @example
 * Seseragi: let Point { x, y } = point
 * TypeScript: const { x, y } = point;
 */
export function generateStructDestructuring(
  ctx: CodeGenContext,
  stmt: StructDestructuring
): string {
  const indent = getIndent(ctx)
  const valueExpr = generateExpression(ctx, stmt.initializer)

  // 構造体名は型情報として利用（将来的に型チェックなどに使用可能）
  const _structName = stmt.pattern.structName

  // パターンからフィールドマッピングを抽出
  const fields = stmt.pattern.fields
  const bindings = fields
    .map((field) => {
      const fieldName = field.fieldName

      // ワイルドカードフィールドはスキップ
      if (fieldName === "_") {
        return null
      }

      // エイリアスがある場合: { x: posX }
      if (field.alias) {
        const alias = sanitizeIdentifier(field.alias)
        return `${fieldName}: ${alias}`
      }

      // ネストされたパターンがある場合（将来拡張）
      if (field.pattern) {
        // TODO: ネストされたパターンのサポート
        throw new Error("Nested patterns in struct destructuring not supported")
      }

      // 通常のフィールド: { x }
      return sanitizeIdentifier(fieldName)
    })
    .filter((b) => b !== null)

  // 全フィールドがワイルドカードの場合は空の分解代入
  if (bindings.length === 0) {
    // 副作用のために値を評価するだけ
    return `${indent}${valueExpr};`
  }

  // TypeScript分解代入生成
  return `${indent}const { ${bindings.join(", ")} } = ${valueExpr};`
}
