/**
 * レコード式の制約生成
 */

import * as AST from "../../../ast"
import { isMaybeType } from "../../type-inspection"
import {
  addError,
  freshTypeVariable,
  type InferenceContext,
} from "../context"
import { generateConstraintsForExpression } from "./dispatcher"

/**
 * 型エイリアスを再帰的に解決（簡略版）
 * TODO: 完全な型エイリアス解決はtype-alias-resolver.tsと統合予定
 */
function resolveTypeAlias(
  ctx: InferenceContext,
  type: AST.Type
): AST.Type {
  if (type.kind === "PrimitiveType") {
    const aliasInfo = ctx.typeAliases.get((type as AST.PrimitiveType).name)
    if (aliasInfo) {
      return aliasInfo.type
    }
  }
  return type
}

/**
 * IntersectionTypeからRecordTypeを抽出
 */
function extractRecordFromIntersection(
  intersection: AST.IntersectionType
): AST.RecordType | null {
  const allFields: AST.RecordField[] = []

  for (const memberType of intersection.types) {
    if (memberType.kind === "RecordType") {
      const recordType = memberType as AST.RecordType
      for (const field of recordType.fields) {
        // 重複フィールドは後のもので上書き
        const existingIndex = allFields.findIndex(f => f.name === field.name)
        if (existingIndex >= 0) {
          allFields[existingIndex] = field
        } else {
          allFields.push(field)
        }
      }
    }
  }

  if (allFields.length > 0) {
    return new AST.RecordType(
      allFields,
      intersection.line,
      intersection.column
    )
  }
  return null
}

/**
 * レコード式の制約を生成
 * { field1: value1, field2: value2, ... } のような式を処理
 */
export function generateConstraintsForRecordExpression(
  ctx: InferenceContext,
  record: AST.RecordExpression,
  env: Map<string, AST.Type>,
  expectedType?: AST.Type
): AST.Type {
  const fieldMap = new Map<string, AST.RecordField>()

  // 期待される型からレコード型を取得
  let expectedRecordType: AST.RecordType | null = null
  if (expectedType) {
    const resolvedExpectedType = resolveTypeAlias(ctx, expectedType)
    if (resolvedExpectedType.kind === "RecordType") {
      expectedRecordType = resolvedExpectedType as AST.RecordType
    } else if (resolvedExpectedType.kind === "IntersectionType") {
      expectedRecordType = extractRecordFromIntersection(
        resolvedExpectedType as AST.IntersectionType
      )
    }
  }

  for (const field of record.fields) {
    if (field.kind === "RecordInitField") {
      const initField = field as AST.RecordInitField

      // 期待されるフィールド型を取得
      let expectedFieldType: AST.Type | undefined
      if (expectedRecordType) {
        const expectedField = expectedRecordType.fields.find(
          (f) => f.name === initField.name
        )
        if (expectedField) {
          expectedFieldType = expectedField.type
        }
      }

      const fieldType = generateConstraintsForExpression(
        ctx,
        initField.value,
        env,
        expectedFieldType
      )

      fieldMap.set(
        initField.name,
        new AST.RecordField(
          initField.name,
          fieldType,
          initField.line,
          initField.column
        )
      )
    } else if (field.kind === "RecordShorthandField") {
      const shorthandField = field as AST.RecordShorthandField
      const variableType = env.get(shorthandField.name)

      if (!variableType) {
        addError(
          ctx,
          `Undefined variable '${shorthandField.name}' in shorthand property`,
          shorthandField.line,
          shorthandField.column
        )
        const fallbackType = freshTypeVariable(ctx, shorthandField.line, shorthandField.column)
        fieldMap.set(
          shorthandField.name,
          new AST.RecordField(
            shorthandField.name,
            fallbackType,
            shorthandField.line,
            shorthandField.column
          )
        )
      } else {
        fieldMap.set(
          shorthandField.name,
          new AST.RecordField(
            shorthandField.name,
            variableType,
            shorthandField.line,
            shorthandField.column
          )
        )
      }
    } else if (field.kind === "RecordSpreadField") {
      const spreadField = field as AST.RecordSpreadField
      const spreadType = generateConstraintsForExpression(
        ctx,
        spreadField.spreadExpression,
        env
      )

      if (spreadType.kind === "RecordType") {
        const recordType = spreadType as AST.RecordType
        for (const sourceField of recordType.fields) {
          fieldMap.set(sourceField.name, sourceField)
        }
      } else {
        addError(
          ctx,
          `Cannot spread non-record type in record literal`,
          spreadField.line,
          spreadField.column
        )
      }
    }
  }

  // Maybe型フィールドの自動補完
  if (expectedRecordType) {
    for (const expectedField of expectedRecordType.fields) {
      if (!fieldMap.has(expectedField.name)) {
        if (isMaybeType(expectedField.type)) {
          // Nothing値を自動設定
          const nothingConstructor = new AST.ConstructorExpression(
            "Nothing",
            [],
            record.line,
            record.column
          )
          const nothingField = new AST.RecordInitField(
            expectedField.name,
            nothingConstructor,
            record.line,
            record.column
          )

          record.fields.push(nothingField)

          const nothingType = generateConstraintsForExpression(
            ctx,
            nothingConstructor,
            env,
            expectedField.type
          )
          fieldMap.set(
            expectedField.name,
            new AST.RecordField(
              expectedField.name,
              nothingType,
              record.line,
              record.column
            )
          )
        }
      }
    }
  }

  const fields = Array.from(fieldMap.values())
  return new AST.RecordType(fields, record.line, record.column)
}
