/**
 * 型統一（Unification）モジュール
 *
 * Hindley-Milner型推論の核心部分
 * 2つの型を統一し、型代入（Substitution）を返す
 */

import * as AST from "../../ast"
import { TypeSubstitution } from "../substitution"
import {
  isRecordSubset,
  isRecordSubtype as isRecordSubtypeUtil,
  typesEqual,
} from "../type-comparison"
import { typeToCanonicalString, typeToString } from "../type-formatter"
import { getTypeName, occursCheck } from "../type-inspection"
import type { TypeVariable } from "../type-variables"
import type { InferenceContext } from "./context"
import { resolveTypeAlias } from "./type-alias-resolver"

/**
 * 統一結果
 * 成功時はsubstitution、失敗時はerror
 */
export type UnifyResult =
  | { success: true; substitution: TypeSubstitution }
  | { success: false; error: string }

/**
 * 2つの型を統一する（純粋関数版）
 * 例外を投げずに結果を返す
 */
export function unify(
  ctx: InferenceContext,
  type1: AST.Type,
  type2: AST.Type
): UnifyResult {
  try {
    const substitution = unifyInternal(ctx, type1, type2)
    return { success: true, substitution }
  } catch (e) {
    return { success: false, error: (e as Error).message }
  }
}

/**
 * 2つの型を統一する（例外を投げるバージョン）
 * 既存のTypeInferenceSystemとの互換性のため
 */
export function unifyOrThrow(
  ctx: InferenceContext,
  type1: AST.Type,
  type2: AST.Type
): TypeSubstitution {
  return unifyInternal(ctx, type1, type2)
}

/**
 * 内部統一ロジック
 */
function unifyInternal(
  ctx: InferenceContext,
  type1: AST.Type,
  type2: AST.Type
): TypeSubstitution {
  // null/undefined チェック
  if (!type1 || !type2) {
    throw new Error(`Cannot unify types: one or both types are undefined/null`)
  }

  const substitution = new TypeSubstitution()

  // 型エイリアスを解決
  const resolvedType1 = resolveTypeAlias(ctx, type1)
  const resolvedType2 = resolveTypeAlias(ctx, type2)

  if (!resolvedType1 || !resolvedType2) {
    throw new Error(
      `Cannot resolve type aliases: resolved types are undefined/null`
    )
  }

  // 同じ型の場合
  if (typesEqual(resolvedType1, resolvedType2)) {
    return substitution
  }

  // 型エイリアスの場合、解決前の名前でも比較
  if (type1.kind === "PrimitiveType" && type2.kind === "PrimitiveType") {
    const prim1 = type1 as AST.PrimitiveType
    const prim2 = type2 as AST.PrimitiveType
    if (prim1.name === prim2.name) {
      return substitution
    }
  }

  // 型変数の場合
  if (type1.kind === "TypeVariable") {
    const tv1 = type1 as TypeVariable
    if (occursCheck(tv1.id, resolvedType2)) {
      throw new Error(
        `Infinite type: ${tv1.name} occurs in ${typeToString(resolvedType2)}`
      )
    }
    substitution.set(tv1.id, resolvedType2)
    return substitution
  }

  if (type2.kind === "TypeVariable") {
    const tv2 = type2 as TypeVariable
    if (occursCheck(tv2.id, resolvedType1)) {
      throw new Error(
        `Infinite type: ${tv2.name} occurs in ${typeToString(resolvedType1)}`
      )
    }
    substitution.set(tv2.id, resolvedType1)
    return substitution
  }

  // 多相型変数の場合
  if (
    resolvedType1.kind === "PolymorphicTypeVariable" ||
    resolvedType2.kind === "PolymorphicTypeVariable"
  ) {
    return substitution
  }

  // プリミティブ型の場合
  if (
    resolvedType1.kind === "PrimitiveType" &&
    resolvedType2.kind === "PrimitiveType"
  ) {
    const pt1 = resolvedType1 as AST.PrimitiveType
    const pt2 = resolvedType2 as AST.PrimitiveType
    if (pt1.name === pt2.name) {
      return substitution
    }
    throw new Error(
      `Cannot unify ${typeToString(type1)} with ${typeToString(type2)}`
    )
  }

  // Function型の場合
  if (type1.kind === "FunctionType" && type2.kind === "FunctionType") {
    const ft1 = type1 as AST.FunctionType
    const ft2 = type2 as AST.FunctionType

    const paramSub = unifyInternal(ctx, ft1.paramType, ft2.paramType)
    const returnSub = unifyInternal(
      ctx,
      paramSub.apply(ft1.returnType),
      paramSub.apply(ft2.returnType)
    )

    return paramSub.compose(returnSub)
  }

  // ジェネリック型の場合
  if (type1.kind === "GenericType" && type2.kind === "GenericType") {
    const gt1 = type1 as AST.GenericType
    const gt2 = type2 as AST.GenericType

    // ArrayとListの相互変換を許可
    const isArrayListCompatible =
      (gt1.name === "Array" && gt2.name === "List") ||
      (gt1.name === "List" && gt2.name === "Array")

    if (
      !isArrayListCompatible &&
      (gt1.name !== gt2.name ||
        gt1.typeArguments.length !== gt2.typeArguments.length)
    ) {
      throw new Error(
        `Cannot unify ${typeToString(type1)} with ${typeToString(type2)}`
      )
    }

    let result = substitution
    for (let i = 0; i < gt1.typeArguments.length; i++) {
      const arg1 = gt1.typeArguments[i]
      const arg2 = gt2.typeArguments[i]
      if (!arg1 || !arg2) continue
      const argSub = unifyInternal(ctx, result.apply(arg1), result.apply(arg2))
      result = result.compose(argSub)
    }

    return result
  }

  // Tuple型の場合
  if (type1.kind === "TupleType" && type2.kind === "TupleType") {
    const tt1 = type1 as AST.TupleType
    const tt2 = type2 as AST.TupleType

    if (tt1.elementTypes.length !== tt2.elementTypes.length) {
      throw new Error(
        `Cannot unify ${typeToString(type1)} with ${typeToString(type2)}: different tuple lengths`
      )
    }

    let result = substitution
    for (let i = 0; i < tt1.elementTypes.length; i++) {
      const elem1 = tt1.elementTypes[i]
      const elem2 = tt2.elementTypes[i]
      if (!elem1 || !elem2) continue
      const elementSub = unifyInternal(
        ctx,
        result.apply(elem1),
        result.apply(elem2)
      )
      result = result.compose(elementSub)
    }

    return result
  }

  // Record型の場合
  if (
    resolvedType1.kind === "RecordType" &&
    resolvedType2.kind === "RecordType"
  ) {
    return unifyRecordTypes(
      ctx,
      resolvedType1 as AST.RecordType,
      resolvedType2 as AST.RecordType,
      type1,
      type2
    )
  }

  // Struct型の場合
  if (type1.kind === "StructType" && type2.kind === "StructType") {
    const st1 = type1 as AST.StructType
    const st2 = type2 as AST.StructType

    if (st1.name !== st2.name) {
      throw new Error(`Cannot unify struct types ${st1.name} and ${st2.name}`)
    }

    return substitution
  }

  // StructTypeとPrimitiveTypeの統一（型アノテーションでは構造体名がPrimitiveTypeとして解析される）
  if (
    (type1.kind === "StructType" && type2.kind === "PrimitiveType") ||
    (type1.kind === "PrimitiveType" && type2.kind === "StructType")
  ) {
    const structType =
      type1.kind === "StructType"
        ? (type1 as AST.StructType)
        : (type2 as AST.StructType)
    const primitiveType =
      type1.kind === "PrimitiveType"
        ? (type1 as AST.PrimitiveType)
        : (type2 as AST.PrimitiveType)

    // 名前が一致していれば統一可能
    if (structType.name === primitiveType.name) {
      return substitution
    }
  }

  // Struct型とRecord型の統一
  if (
    (type1.kind === "StructType" && type2.kind === "RecordType") ||
    (type1.kind === "RecordType" && type2.kind === "StructType")
  ) {
    return unifyStructAndRecord(ctx, type1, type2)
  }

  // Record型と他の型
  if (type1.kind === "RecordType" || type2.kind === "RecordType") {
    const recordType =
      type1.kind === "RecordType"
        ? (type1 as AST.RecordType)
        : (type2 as AST.RecordType)
    const otherType = type1.kind === "RecordType" ? type2 : type1

    if (otherType.kind === "TypeVariable") {
      const tv = otherType as TypeVariable
      if (occursCheck(tv.id, recordType)) {
        throw new Error(
          `Infinite type: ${tv.name} occurs in ${typeToString(recordType)}`
        )
      }
      substitution.set(tv.id, recordType)
      return substitution
    }

    throw new Error(
      `Cannot unify ${typeToString(type1)} with ${typeToString(type2)}`
    )
  }

  // Union型の場合（解決後の型で判定）
  if (
    resolvedType1.kind === "UnionType" ||
    resolvedType2.kind === "UnionType"
  ) {
    return unifyUnionTypes(ctx, type1, type2, resolvedType1, resolvedType2)
  }

  // Intersection型の場合
  if (type1.kind === "IntersectionType" || type2.kind === "IntersectionType") {
    return unifyIntersectionTypes(ctx, type1, type2)
  }

  throw new Error(
    `Cannot unify ${typeToString(type1)} with ${typeToString(type2)}`
  )
}

/**
 * Record型同士の統一
 */
function unifyRecordTypes(
  ctx: InferenceContext,
  rt1: AST.RecordType,
  rt2: AST.RecordType,
  originalType1: AST.Type,
  originalType2: AST.Type
): TypeSubstitution {
  const substitution = new TypeSubstitution()

  // 部分型関係をチェック
  if (isSubtype(ctx, rt1, rt2)) {
    let result = substitution
    for (const superField of rt2.fields) {
      const subField = rt1.fields.find((f) => f.name === superField.name)
      if (subField) {
        const fieldSub = unifyInternal(
          ctx,
          result.apply(subField.type),
          result.apply(superField.type)
        )
        result = result.compose(fieldSub)
      }
    }
    return result
  }

  if (isSubtype(ctx, rt2, rt1)) {
    let result = substitution
    for (const superField of rt1.fields) {
      const subField = rt2.fields.find((f) => f.name === superField.name)
      if (subField) {
        const fieldSub = unifyInternal(
          ctx,
          result.apply(subField.type),
          result.apply(superField.type)
        )
        result = result.compose(fieldSub)
      }
    }
    return result
  }

  // サブセット関係
  const [largerRecord, smallerRecord] =
    rt1.fields.length >= rt2.fields.length ? [rt1, rt2] : [rt2, rt1]
  const isSubset = isRecordSubset(smallerRecord, largerRecord)

  if (isSubset) {
    let result = substitution
    for (const smallerField of smallerRecord.fields) {
      const largerField = largerRecord.fields.find(
        (f) => f.name === smallerField.name
      )
      if (largerField) {
        const fieldSub = unifyInternal(
          ctx,
          result.apply(smallerField.type),
          result.apply(largerField.type)
        )
        result = result.compose(fieldSub)
      }
    }
    return result
  }

  // 完全一致が必要
  if (rt1.fields.length !== rt2.fields.length) {
    throw new Error(
      `Cannot unify ${typeToString(originalType1)} with ${typeToString(originalType2)}: incompatible record structures`
    )
  }

  // フィールド名でソートして比較
  const fields1 = [...rt1.fields].sort((a, b) => a.name.localeCompare(b.name))
  const fields2 = [...rt2.fields].sort((a, b) => a.name.localeCompare(b.name))

  let result = substitution
  for (let i = 0; i < fields1.length; i++) {
    const f1 = fields1[i]
    const f2 = fields2[i]
    if (!f1 || !f2) continue
    if (f1.name !== f2.name) {
      throw new Error(
        `Cannot unify ${typeToString(originalType1)} with ${typeToString(originalType2)}: field names don't match`
      )
    }

    const fieldSub = unifyInternal(
      ctx,
      result.apply(f1.type),
      result.apply(f2.type)
    )
    result = result.compose(fieldSub)
  }

  return result
}

/**
 * Struct型とRecord型の統一
 */
function unifyStructAndRecord(
  ctx: InferenceContext,
  type1: AST.Type,
  type2: AST.Type
): TypeSubstitution {
  const substitution = new TypeSubstitution()

  const structType =
    type1.kind === "StructType"
      ? (type1 as AST.StructType)
      : (type2 as AST.StructType)
  const recordType =
    type1.kind === "RecordType"
      ? (type1 as AST.RecordType)
      : (type2 as AST.RecordType)

  const structAsRecord = new AST.RecordType(
    structType.fields,
    structType.line,
    structType.column
  )

  if (isRecordSubset(recordType, structAsRecord)) {
    let result = substitution
    for (const recordField of recordType.fields) {
      const structField = structType.fields.find(
        (f) => f.name === recordField.name
      )
      if (structField) {
        const fieldSub = unifyInternal(
          ctx,
          result.apply(recordField.type),
          result.apply(structField.type)
        )
        result = result.compose(fieldSub)
      }
    }
    return result
  }

  throw new Error(
    `Cannot unify ${typeToString(type1)} with ${typeToString(type2)}`
  )
}

/**
 * Union型の統一
 */
function unifyUnionTypes(
  ctx: InferenceContext,
  type1: AST.Type,
  type2: AST.Type,
  resolvedType1: AST.Type,
  resolvedType2: AST.Type
): TypeSubstitution {
  const substitution = new TypeSubstitution()

  // 両方がUnion型の場合
  if (
    resolvedType1.kind === "UnionType" &&
    resolvedType2.kind === "UnionType"
  ) {
    const union1 = resolvedType1 as AST.UnionType
    const union2 = resolvedType2 as AST.UnionType

    if (union1.types.length === union2.types.length) {
      // 構造的等価性チェック
      if (areUnionTypesStructurallyEqual(ctx, union1, union2)) {
        return substitution
      }

      // 順序に関係なく統合を試みる
      let result = substitution
      const usedIndices = new Set<number>()

      for (let i = 0; i < union1.types.length; i++) {
        const type1Member = union1.types[i]
        if (!type1Member) continue
        let found = false
        for (let j = 0; j < union2.types.length; j++) {
          if (usedIndices.has(j)) continue
          const type2Member = union2.types[j]
          if (!type2Member) continue

          try {
            const memberSub = unifyInternal(ctx, type1Member, type2Member)
            result = result.compose(memberSub)
            usedIndices.add(j)
            found = true
            break
          } catch {
            // 次を試す
          }
        }

        if (!found) {
          throw new Error(
            `Cannot match union type member ${typeToString(type1Member)}`
          )
        }
      }

      return result
    }
  }

  // 型変数とユニオン型
  if (type1.kind === "TypeVariable" && resolvedType2.kind === "UnionType") {
    const tv = type1 as TypeVariable
    substitution.set(tv.id, resolvedType2)
    return substitution
  }

  if (type2.kind === "TypeVariable" && resolvedType1.kind === "UnionType") {
    const tv = type2 as TypeVariable
    substitution.set(tv.id, resolvedType1)
    return substitution
  }

  // ADTコンストラクタとADT型の統合
  if (
    resolvedType1.kind === "UnionType" &&
    resolvedType2.kind === "PrimitiveType"
  ) {
    const union = resolvedType1 as AST.UnionType
    const primitive = resolvedType2 as AST.PrimitiveType

    const isConstructor = union.types.some(
      (memberType) =>
        memberType.kind === "PrimitiveType" &&
        (memberType as AST.PrimitiveType).name === primitive.name
    )

    if (isConstructor) {
      return substitution
    }
  }

  if (
    resolvedType2.kind === "UnionType" &&
    resolvedType1.kind === "PrimitiveType"
  ) {
    const union = resolvedType2 as AST.UnionType
    const primitive = resolvedType1 as AST.PrimitiveType

    const isConstructor = union.types.some(
      (memberType) =>
        memberType.kind === "PrimitiveType" &&
        (memberType as AST.PrimitiveType).name === primitive.name
    )

    if (isConstructor) {
      return substitution
    }
  }

  // Union型を非Union型に統合
  if (resolvedType1.kind === "UnionType" && type2.kind !== "TypeVariable") {
    const union1 = resolvedType1 as AST.UnionType

    let resultSubstitution = substitution
    let allCanUnify = true
    let hasTypeVariable = false

    for (const memberType of union1.types) {
      if (
        memberType.kind === "TypeVariable" ||
        memberType.kind === "PolymorphicTypeVariable" ||
        (memberType.kind === "GenericType" &&
          (memberType as AST.GenericType).typeArguments.some(
            (arg) =>
              arg.kind === "TypeVariable" ||
              arg.kind === "PolymorphicTypeVariable"
          ))
      ) {
        hasTypeVariable = true
      }

      try {
        const memberSub = unifyInternal(ctx, memberType, resolvedType2)
        resultSubstitution = resultSubstitution.compose(memberSub)
      } catch {
        allCanUnify = false
        break
      }
    }

    if (hasTypeVariable && allCanUnify) {
      return resultSubstitution
    }

    if (resolvedType2.kind === "UnionType") {
      const union2 = resolvedType2 as AST.UnionType
      const allMembersIncluded = union1.types.every((member1) =>
        union2.types.some((member2) => typesEqual(member1, member2))
      )
      if (allMembersIncluded) {
        return substitution
      }
    }

    if (resolvedType2.kind !== "UnionType") {
      const canUnifyWithMember = union1.types.some((memberType) => {
        try {
          unifyInternal(ctx, memberType, resolvedType2)
          return true
        } catch {
          return false
        }
      })
      if (canUnifyWithMember) {
        return substitution
      }
    }

    throw new Error(
      `Union type ${typeToString(resolvedType1)} cannot be assigned to ${typeToString(resolvedType2)}`
    )
  }

  if (resolvedType2.kind === "UnionType" && type1.kind !== "TypeVariable") {
    const union2 = resolvedType2 as AST.UnionType
    const isMember = union2.types.some((memberType) => {
      try {
        // メンバー型も型エイリアス解決してから比較
        const resolvedMemberType = resolveTypeAlias(ctx, memberType)
        unifyInternal(ctx, resolvedType1, resolvedMemberType)
        return true
      } catch {
        return false
      }
    })
    if (isMember) {
      return substitution
    }
    throw new Error(
      `Type ${typeToString(resolvedType1)} is not assignable to union type ${typeToString(resolvedType2)}`
    )
  }

  throw new Error(
    `Cannot unify union types ${typeToString(type1)} with ${typeToString(type2)}`
  )
}

/**
 * Union型の構造的等価性チェック
 */
function areUnionTypesStructurallyEqual(
  ctx: InferenceContext,
  union1: AST.UnionType,
  union2: AST.UnionType
): boolean {
  if (union1.types.length !== union2.types.length) {
    return false
  }

  const normalizedTypes1 = union1.types
    .map((t) => typeToCanonicalString(resolveTypeAlias(ctx, t)))
    .sort()
  const normalizedTypes2 = union2.types
    .map((t) => typeToCanonicalString(resolveTypeAlias(ctx, t)))
    .sort()

  return normalizedTypes1.every((type1, i) => type1 === normalizedTypes2[i])
}

/**
 * Intersection型の統一
 */
function unifyIntersectionTypes(
  ctx: InferenceContext,
  type1: AST.Type,
  type2: AST.Type
): TypeSubstitution {
  const substitution = new TypeSubstitution()

  // 両方がIntersection型の場合
  if (type1.kind === "IntersectionType" && type2.kind === "IntersectionType") {
    const intersect1 = type1 as AST.IntersectionType
    const intersect2 = type2 as AST.IntersectionType

    const record1 = expandIntersectionToRecord(ctx, intersect1)
    const record2 = expandIntersectionToRecord(ctx, intersect2)

    if (record1 && record2) {
      return unifyInternal(ctx, record1, record2)
    }

    let result = substitution
    for (const member1 of intersect1.types) {
      for (const member2 of intersect2.types) {
        try {
          const sub = unifyInternal(ctx, member1, member2)
          result = result.compose(sub)
        } catch {
          // 次を試す
        }
      }
    }
    return result
  }

  // 片方がIntersection型の場合
  if (type1.kind === "IntersectionType") {
    const intersect1 = type1 as AST.IntersectionType
    const record1 = expandIntersectionToRecord(ctx, intersect1)
    if (record1) {
      return unifyInternal(ctx, record1, type2)
    }

    let result = substitution
    for (const memberType of intersect1.types) {
      const sub = unifyInternal(ctx, memberType, type2)
      result = result.compose(sub)
    }
    return result
  }

  if (type2.kind === "IntersectionType") {
    const intersect2 = type2 as AST.IntersectionType
    const record2 = expandIntersectionToRecord(ctx, intersect2)
    if (record2) {
      return unifyInternal(ctx, type1, record2)
    }

    let result = substitution
    for (const memberType of intersect2.types) {
      const sub = unifyInternal(ctx, type1, memberType)
      result = result.compose(sub)
    }
    return result
  }

  throw new Error(
    `Cannot unify intersection types ${typeToString(type1)} with ${typeToString(type2)}`
  )
}

/**
 * Intersection型をRecord型に展開
 */
function expandIntersectionToRecord(
  ctx: InferenceContext,
  intersectionType: AST.IntersectionType
): AST.RecordType | null {
  const mergedFields: AST.RecordField[] = []

  for (const memberType of intersectionType.types) {
    const resolvedType = resolveTypeAlias(ctx, memberType)

    if (resolvedType.kind === "RecordType") {
      const recordType = resolvedType as AST.RecordType

      for (const field of recordType.fields) {
        const existingField = mergedFields.find((f) => f.name === field.name)
        if (existingField) {
          const existingTypeName = getTypeName(existingField.type)
          const newTypeName = getTypeName(field.type)
          if (existingTypeName !== newTypeName) {
            throw new Error(
              `Field '${field.name}' has conflicting types in intersection: ${existingTypeName} and ${newTypeName}`
            )
          }
        } else {
          mergedFields.push(field)
        }
      }
    } else {
      return null
    }
  }

  return new AST.RecordType(
    mergedFields,
    intersectionType.line,
    intersectionType.column
  )
}

/**
 * 部分型関係を判定: subType <: superType
 */
export function isSubtype(
  ctx: InferenceContext,
  subType: AST.Type,
  superType: AST.Type
): boolean {
  if (typesEqual(subType, superType)) {
    return true
  }

  const resolvedSub = resolveTypeAlias(ctx, subType)
  const resolvedSuper = resolveTypeAlias(ctx, superType)

  if (typesEqual(resolvedSub, resolvedSuper)) {
    return true
  }

  // Record型の構造的部分型関係
  if (
    resolvedSub.kind === "RecordType" &&
    resolvedSuper.kind === "RecordType"
  ) {
    return isRecordSubtypeUtil(
      resolvedSub as AST.RecordType,
      resolvedSuper as AST.RecordType,
      (sub, sup) => isSubtype(ctx, sub, sup)
    )
  }

  return false
}
