/**
 * 構造体関連式の生成
 */

import type {
  RecordInitField,
  RecordShorthandField,
  RecordSpreadField,
  SpreadExpression,
  StructExpression,
} from "../../../ast"
import type { CodeGenContext } from "../../context"
import { generateExpression } from "../dispatcher"

/**
 * 構造体式をTypeScriptコードに変換
 */
export function generateStructExpression(
  ctx: CodeGenContext,
  structExpr: StructExpression
): string {
  // スプレッド構文または省略記法があるかチェック
  const hasSpread = hasSpreadFields(structExpr)
  const hasShorthand = hasShorthandFields(structExpr)

  if (hasSpread || hasShorthand) {
    return generateComplexStructExpression(ctx, structExpr)
  }

  return generateSimpleStructExpression(ctx, structExpr)
}

/**
 * スプレッド式をTypeScriptコードに変換
 */
export function generateSpreadExpression(
  ctx: CodeGenContext,
  spread: SpreadExpression
): string {
  // スプレッド式は通常直接使われることはないが、TypeScriptのスプレッド構文と同じ
  return `...${generateExpression(ctx, spread.expression)}`
}

/**
 * スプレッドフィールドが含まれているかチェック
 */
function hasSpreadFields(structExpr: StructExpression): boolean {
  return structExpr.fields.some((field) => field.kind === "RecordSpreadField")
}

/**
 * 省略記法フィールドが含まれているかチェック
 */
function hasShorthandFields(structExpr: StructExpression): boolean {
  return structExpr.fields.some(
    (field) => field.kind === "RecordShorthandField"
  )
}

/**
 * 複雑な構造体式の生成（スプレッドまたは省略記法を含む）
 */
function generateComplexStructExpression(
  ctx: CodeGenContext,
  structExpr: StructExpression
): string {
  // スプレッドフィールドとイニシャライザーフィールドを収集
  const { spreadExpressions, initFields } = collectStructFields(ctx, structExpr)

  // フィールド部分文字列を組み立て
  const allFields = combineStructFields(spreadExpressions, initFields)

  if (allFields) {
    // 一時オブジェクトを作成し、構造体定義の順序に従ってコンストラクタ引数を構築
    const tempVar = `__tmp${Math.random().toString(36).substring(2, 8)}`
    return `(() => { const ${tempVar} = { ${allFields} }; return Object.assign(Object.create(${structExpr.structName}.prototype), ${tempVar}); })()`
  }

  return `new ${structExpr.structName}({})`
}

/**
 * 単純な構造体式の生成（スプレッドや省略記法なし）
 */
function generateSimpleStructExpression(
  ctx: CodeGenContext,
  structExpr: StructExpression
): string {
  // 従来のコンストラクタ形式（スプレッドなし）
  const fieldEntries: string[] = []

  for (const field of structExpr.fields) {
    if (field.kind === "RecordInitField") {
      const initField = field as RecordInitField
      if (!initField.value) continue
      const value = generateExpression(ctx, initField.value)
      fieldEntries.push(`${initField.name}: ${value}`)
    } else if (field.kind === "RecordShorthandField") {
      const shorthandField = field as RecordShorthandField
      fieldEntries.push(shorthandField.name)
    }
  }

  const fieldsObject =
    fieldEntries.length > 0 ? `{ ${fieldEntries.join(", ")} }` : "{}"

  return `new ${structExpr.structName}(${fieldsObject})`
}

/**
 * 構造体フィールドを収集（スプレッドとイニシャライザーに分離）
 */
function collectStructFields(
  ctx: CodeGenContext,
  structExpr: StructExpression
): {
  spreadExpressions: string[]
  initFields: { name: string; value: string }[]
} {
  const spreadExpressions: string[] = []
  const initFields: { name: string; value: string }[] = []

  for (const field of structExpr.fields) {
    if (field.kind === "RecordSpreadField") {
      const spreadField = field as RecordSpreadField
      if (!spreadField.spreadExpression?.expression) continue
      const spreadValue = generateExpression(
        ctx,
        spreadField.spreadExpression.expression
      )
      spreadExpressions.push(spreadValue)
    } else if (field.kind === "RecordInitField") {
      const initField = field as RecordInitField
      if (!initField.value) continue
      const value = generateExpression(ctx, initField.value)
      initFields.push({ name: initField.name, value })
    } else if (field.kind === "RecordShorthandField") {
      const shorthandField = field as RecordShorthandField
      // Shorthand property: use variable name directly
      initFields.push({
        name: shorthandField.name,
        value: shorthandField.name,
      })
    }
  }

  return { spreadExpressions, initFields }
}

/**
 * スプレッドとフィールドを結合
 */
function combineStructFields(
  spreadExpressions: string[],
  initFields: { name: string; value: string }[]
): string {
  const spreadPart = spreadExpressions.map((expr) => `...${expr}`).join(", ")
  const fieldsPart = initFields.map((f) => `${f.name}: ${f.value}`).join(", ")

  if (spreadPart && fieldsPart) {
    return `${spreadPart}, ${fieldsPart}`
  } else if (spreadPart) {
    return spreadPart
  } else if (fieldsPart) {
    return fieldsPart
  }

  return ""
}
