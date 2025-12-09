/**
 * 型比較ユーティリティ (Type Comparison) for Seseragi Language
 *
 * 型の等価性・部分型関係を判定する純粋関数群
 */

import * as AST from "../ast"
import type { PolymorphicTypeVariable, TypeVariable } from "./type-variables"

/**
 * 2つの型が等しいかどうかを判定
 * 構造的等価性をチェック
 */
export function typesEqual(type1: AST.Type, type2: AST.Type): boolean {
  if (type1.kind !== type2.kind) return false

  switch (type1.kind) {
    case "PrimitiveType":
      return (
        (type1 as AST.PrimitiveType).name === (type2 as AST.PrimitiveType).name
      )

    case "TypeVariable":
      return (type1 as TypeVariable).id === (type2 as TypeVariable).id

    case "PolymorphicTypeVariable":
      return (
        (type1 as PolymorphicTypeVariable).name ===
        (type2 as PolymorphicTypeVariable).name
      )

    case "FunctionType": {
      const ft1 = type1 as AST.FunctionType
      const ft2 = type2 as AST.FunctionType
      return (
        typesEqual(ft1.paramType, ft2.paramType) &&
        typesEqual(ft1.returnType, ft2.returnType)
      )
    }

    case "GenericType": {
      const gt1 = type1 as AST.GenericType
      const gt2 = type2 as AST.GenericType
      return (
        gt1.name === gt2.name &&
        gt1.typeArguments.length === gt2.typeArguments.length &&
        gt1.typeArguments.every((arg, i) =>
          typesEqual(arg, gt2.typeArguments[i])
        )
      )
    }

    case "RecordType": {
      const rt1 = type1 as AST.RecordType
      const rt2 = type2 as AST.RecordType

      if (rt1.fields.length !== rt2.fields.length) {
        return false
      }

      // フィールド名でソートして比較
      const fields1 = [...rt1.fields].sort((a, b) =>
        a.name.localeCompare(b.name)
      )
      const fields2 = [...rt2.fields].sort((a, b) =>
        a.name.localeCompare(b.name)
      )

      return fields1.every((field1, i) => {
        const field2 = fields2[i]
        return (
          field1.name === field2?.name && typesEqual(field1.type, field2.type)
        )
      })
    }

    case "StructType": {
      const st1 = type1 as AST.StructType
      const st2 = type2 as AST.StructType
      return st1.name === st2.name
    }

    case "TupleType": {
      const tt1 = type1 as AST.TupleType
      const tt2 = type2 as AST.TupleType

      if (tt1.elementTypes.length !== tt2.elementTypes.length) {
        return false
      }

      return tt1.elementTypes.every((elementType, i) =>
        typesEqual(elementType, tt2.elementTypes[i])
      )
    }

    default:
      return false
  }
}

/**
 * Record型の構造的部分型関係を判定
 * subRecord <: superRecord ⟺
 * superRecordのすべてのフィールドがsubRecordに存在し、各フィールドが部分型関係にある
 *
 * @param subRecord - 部分型候補のRecord型
 * @param superRecord - スーパータイプ候補のRecord型
 * @param isSubtypeFn - 部分型関係判定関数（再帰用）
 */
export function isRecordSubtype(
  subRecord: AST.RecordType,
  superRecord: AST.RecordType,
  isSubtypeFn: (sub: AST.Type, sup: AST.Type) => boolean
): boolean {
  for (const superField of superRecord.fields) {
    const subField = subRecord.fields.find((f) => f.name === superField.name)

    // スーパータイプのフィールドがサブタイプに存在しない場合は部分型関係なし
    if (!subField) {
      return false
    }

    // フィールドの型も部分型関係にある必要がある（再帰的チェック）
    if (!isSubtypeFn(subField.type, superField.type)) {
      return false
    }
  }
  return true
}

/**
 * 複数のRecord型をマージ（Intersection型の解決用）
 * 同じ名前のフィールドが異なる型を持つ場合はエラー
 */
export function mergeRecordTypes(
  recordTypes: AST.RecordType[]
): AST.RecordType {
  const mergedFields: AST.RecordField[] = []

  for (const recordType of recordTypes) {
    for (const field of recordType.fields) {
      // 同じ名前のフィールドが既に存在するかチェック
      const existingField = mergedFields.find((f) => f.name === field.name)
      if (existingField) {
        // 同じ名前のフィールドが異なる型を持つ場合はエラー
        if (
          existingField.type.kind !== field.type.kind ||
          existingField.type.name !== field.type.name
        ) {
          throw new Error(
            `Field '${field.name}' has conflicting types in intersection: ${existingField.type.name} and ${field.type.name}`
          )
        }
      } else {
        mergedFields.push(field)
      }
    }
  }

  // 最初のRecord型の位置情報を使用
  const firstRecord = recordTypes[0]
  return new AST.RecordType(mergedFields, firstRecord.line, firstRecord.column)
}

/**
 * ユニオン型を平坦化して作成（重複排除と型の正規化を行う）
 */
export function createFlattenedUnionType(
  types: AST.Type[],
  line: number,
  column: number
): AST.Type {
  const flattenedTypes: AST.Type[] = []

  // 型を平坦化する（ネストしたユニオン型を展開）
  const flattenType = (type: AST.Type) => {
    if (type.kind === "UnionType") {
      const unionType = type as AST.UnionType
      for (const memberType of unionType.types) {
        flattenType(memberType)
      }
    } else {
      flattenedTypes.push(type)
    }
  }

  // 全ての型を平坦化
  for (const type of types) {
    flattenType(type)
  }

  // 重複を排除（同じ型は一つにまとめる）
  const uniqueTypes: AST.Type[] = []
  for (const type of flattenedTypes) {
    const isDuplicate = uniqueTypes.some((existingType) =>
      typesEqual(type, existingType)
    )
    if (!isDuplicate) {
      uniqueTypes.push(type)
    }
  }

  // 1つの型しかない場合はユニオン型ではなくその型を返す
  if (uniqueTypes.length === 1) {
    return uniqueTypes[0]
  }

  // 複数の型がある場合はユニオン型を作成
  return new AST.UnionType(uniqueTypes, line, column)
}

/**
 * 構造的部分型：小さいレコードが大きいレコードのサブセットかどうかチェック
 * フィールドの型の互換性は後でunifyでチェックされる前提
 */
export function isRecordSubset(
  smallerRecord: AST.RecordType,
  largerRecord: AST.RecordType
): boolean {
  // 小さいレコードのすべてのフィールドが大きいレコードに存在するかチェック
  for (const smallerField of smallerRecord.fields) {
    const largerField = largerRecord.fields.find(
      (f) => f.name === smallerField.name
    )
    if (!largerField) {
      return false // フィールドが見つからない
    }
    // フィールドが見つかった場合、型の互換性は後で unify でチェックされる
  }
  return true
}
