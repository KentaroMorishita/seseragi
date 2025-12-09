/**
 * 型検査ユーティリティ (Type Inspection) for Seseragi Language
 *
 * 型の検査・判定を行う純粋関数群
 */

import type * as AST from "../ast"
import type { PolymorphicTypeVariable, TypeVariable } from "./type-variables"

/**
 * 無限型（occurs check）の判定
 * 型変数が型の中に出現するかチェック
 */
export function occursCheck(varId: number, type: AST.Type): boolean {
  switch (type.kind) {
    case "TypeVariable":
      return (type as TypeVariable).id === varId

    case "FunctionType": {
      const ft = type as AST.FunctionType
      return (
        occursCheck(varId, ft.paramType) || occursCheck(varId, ft.returnType)
      )
    }

    case "GenericType": {
      const gt = type as AST.GenericType
      return gt.typeArguments.some((arg) => occursCheck(varId, arg))
    }

    case "RecordType": {
      const rt = type as AST.RecordType
      return rt.fields.some((field) => occursCheck(varId, field.type))
    }

    case "TupleType": {
      const tt = type as AST.TupleType
      return tt.elementTypes.some((elementType) =>
        occursCheck(varId, elementType)
      )
    }

    case "StructType": {
      const st = type as AST.StructType
      return st.fields.some((field) => occursCheck(varId, field.type))
    }

    default:
      return false
  }
}

/**
 * 型に特定の型変数名が含まれているかチェック
 */
export function typeContainsVariable(type: AST.Type, varName: string): boolean {
  switch (type.kind) {
    case "TypeVariable":
      return (type as TypeVariable).name === varName
    case "FunctionType": {
      const ft = type as AST.FunctionType
      return (
        typeContainsVariable(ft.paramType, varName) ||
        typeContainsVariable(ft.returnType, varName)
      )
    }
    case "TupleType": {
      const tt = type as AST.TupleType
      return tt.elementTypes.some((t) => typeContainsVariable(t, varName))
    }
    case "GenericType": {
      const gt = type as AST.GenericType
      return gt.typeArguments.some((t) => typeContainsVariable(t, varName))
    }
    case "RecordType": {
      const rt = type as AST.RecordType
      return rt.fields.some((f) => typeContainsVariable(f.type, varName))
    }
    case "StructType": {
      const st = type as AST.StructType
      return st.fields.some((f) => typeContainsVariable(f.type, varName))
    }
    default:
      return false
  }
}

/**
 * 型名を取得
 */
export function getTypeName(type: AST.Type): string {
  switch (type.kind) {
    case "PrimitiveType":
      return (type as AST.PrimitiveType).name
    case "GenericType":
      return (type as AST.GenericType).name
    case "FunctionType":
      return "Function"
    case "RecordType":
      return "Record"
    case "TupleType":
      return "Tuple"
    case "UnionType":
      return (type as AST.UnionType).name
    case "IntersectionType":
      return (type as AST.IntersectionType).name
    case "StructType":
      return (type as AST.StructType).name
    case "TypeVariable":
      return (type as TypeVariable).name
    case "PolymorphicTypeVariable":
      return (type as PolymorphicTypeVariable).name
    default:
      return "Unknown"
  }
}

/**
 * 関数型かどうか判定
 */
export function isFunctionType(type: AST.Type): boolean {
  switch (type.kind) {
    case "FunctionType":
      return true
    case "TypeVariable":
    case "PolymorphicTypeVariable":
      // 型変数の場合は制約から判定するのは複雑なので、
      // 保守的に関数型と見なす（一般化を適用）
      return true
    default:
      return false
  }
}

/**
 * Maybe型かどうか判定
 */
export function isMaybeType(type: AST.Type): boolean {
  if (type.kind === "GenericType") {
    const genericType = type as AST.GenericType
    return genericType.name === "Maybe"
  }
  return false
}

/**
 * Either型かどうか判定
 */
export function isEitherType(type: AST.Type): boolean {
  if (type.kind === "GenericType") {
    const genericType = type as AST.GenericType
    return genericType.name === "Either"
  }
  return false
}

/**
 * Promise型かどうか判定
 * 関数型の場合、戻り値型がPromiseかチェック
 */
export function isPromiseType(type: AST.Type): boolean {
  if (type.kind === "GenericType") {
    const genericType = type as AST.GenericType
    return genericType.name === "Promise"
  }

  // 関数型の場合、戻り値型がPromiseかチェック
  if (type.kind === "FunctionType") {
    const funcType = type as AST.FunctionType
    return isPromiseType(funcType.returnType)
  }

  return false
}

/**
 * Signal型かどうか判定
 */
export function isSignalType(type: AST.Type): boolean {
  if (type.kind === "GenericType") {
    const genericType = type as AST.GenericType
    return genericType.name === "Signal"
  }
  return false
}

/**
 * Task型かどうか判定
 */
export function isTaskType(type: AST.Type): boolean {
  if (type.kind === "GenericType") {
    const genericType = type as AST.GenericType
    return genericType.name === "Task"
  }
  return false
}

/**
 * List型かどうか判定
 */
export function isListType(type: AST.Type): boolean {
  if (type.kind === "GenericType") {
    const genericType = type as AST.GenericType
    return genericType.name === "List"
  }
  return false
}

/**
 * Array型かどうか判定
 */
export function isArrayType(type: AST.Type): boolean {
  if (type.kind === "GenericType") {
    const genericType = type as AST.GenericType
    return genericType.name === "Array"
  }
  return false
}

/**
 * 型から多相型変数名を収集
 * 重複なしで出現順に返す
 */
export function collectPolymorphicTypeVariables(type: AST.Type): string[] {
  const seen = new Set<string>()
  const vars: string[] = []

  const collect = (t: AST.Type): void => {
    switch (t.kind) {
      case "PolymorphicTypeVariable": {
        const ptv = t as PolymorphicTypeVariable
        if (!seen.has(ptv.name)) {
          seen.add(ptv.name)
          vars.push(ptv.name)
        }
        break
      }
      case "FunctionType": {
        const ft = t as AST.FunctionType
        collect(ft.paramType)
        collect(ft.returnType)
        break
      }
      case "TupleType": {
        const tt = t as AST.TupleType
        tt.elementTypes.forEach(collect)
        break
      }
      case "GenericType": {
        const gt = t as AST.GenericType
        gt.typeArguments.forEach(collect)
        break
      }
      case "RecordType": {
        const rt = t as AST.RecordType
        rt.fields.forEach((f) => collect(f.type))
        break
      }
      case "StructType": {
        const st = t as AST.StructType
        st.fields.forEach((f) => collect(f.type))
        break
      }
      // TypeVariable や PrimitiveType は処理不要
    }
  }

  collect(type)
  return vars
}
