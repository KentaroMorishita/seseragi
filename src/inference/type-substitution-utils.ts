/**
 * 型置換ユーティリティ (Type Substitution Utils) for Seseragi Language
 *
 * 型変数の置換・一般化に関する純粋関数群
 */

import * as AST from "../ast"
import { typeContainsVariable } from "./type-inspection"
import type { PolymorphicTypeVariable, TypeVariable } from "./type-variables"

/**
 * 型変数が環境に束縛されているかチェック
 */
export function isTypeVariableBoundInEnv(
  varName: string,
  env: Map<string, AST.Type>
): boolean {
  for (const [_, envType] of env) {
    if (typeContainsVariable(envType, varName)) {
      return true
    }
  }
  return false
}

/**
 * フリー型変数を取得
 * 環境に束縛されていない型変数を収集
 */
export function getFreeTypeVariables(
  type: AST.Type,
  env: Map<string, AST.Type>
): Set<string> {
  const freeVars = new Set<string>()

  const collect = (t: AST.Type): void => {
    switch (t.kind) {
      case "TypeVariable": {
        const tv = t as TypeVariable
        // 環境に束縛されていない型変数のみを収集
        if (!isTypeVariableBoundInEnv(tv.name, env)) {
          freeVars.add(tv.name)
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
        rt.fields.forEach((field) => collect(field.type))
        break
      }
      case "StructType": {
        const st = t as AST.StructType
        st.fields.forEach((field) => collect(field.type))
        break
      }
      // PolymorphicTypeVariable や PrimitiveType は処理不要
    }
  }

  collect(type)
  return freeVars
}

/**
 * 型変数の置換
 * 置換マップに従って型変数を置換した新しい型を返す
 */
export function substituteTypeVariables(
  type: AST.Type,
  substitutionMap: Map<string, AST.Type>
): AST.Type {
  switch (type.kind) {
    case "TypeVariable": {
      const tv = type as TypeVariable
      return substitutionMap.get(tv.name) || type
    }
    case "PolymorphicTypeVariable": {
      const ptv = type as PolymorphicTypeVariable
      return substitutionMap.get(ptv.name) || type
    }
    case "FunctionType": {
      const ft = type as AST.FunctionType
      return new AST.FunctionType(
        substituteTypeVariables(ft.paramType, substitutionMap),
        substituteTypeVariables(ft.returnType, substitutionMap),
        ft.line,
        ft.column
      )
    }
    case "TupleType": {
      const tt = type as AST.TupleType
      return new AST.TupleType(
        tt.elementTypes.map((t) => substituteTypeVariables(t, substitutionMap)),
        tt.line,
        tt.column
      )
    }
    case "GenericType": {
      const gt = type as AST.GenericType
      return new AST.GenericType(
        gt.name,
        gt.typeArguments.map((t) =>
          substituteTypeVariables(t, substitutionMap)
        ),
        gt.line,
        gt.column
      )
    }
    case "RecordType": {
      const rt = type as AST.RecordType
      return new AST.RecordType(
        rt.fields.map(
          (f) =>
            new AST.RecordField(
              f.name,
              substituteTypeVariables(f.type, substitutionMap),
              f.line,
              f.column
            )
        ),
        rt.line,
        rt.column
      )
    }
    case "StructType": {
      const st = type as AST.StructType
      return new AST.StructType(
        st.name,
        st.fields.map(
          (f) =>
            new AST.RecordField(
              f.name,
              substituteTypeVariables(f.type, substitutionMap),
              f.line,
              f.column
            )
        ),
        st.line,
        st.column
      )
    }
    case "PrimitiveType": {
      // PrimitiveTypeが型パラメータの場合は置換
      const pt = type as AST.PrimitiveType
      const substituted = substitutionMap.get(pt.name)
      if (substituted) {
        return substituted
      }
      return type
    }
    default:
      return type
  }
}

/**
 * 型の中から型変数を収集する
 */
export function collectTypeVariables(
  type: AST.Type,
  typeVariables: Set<TypeVariable>
): void {
  switch (type.kind) {
    case "TypeVariable":
      typeVariables.add(type as TypeVariable)
      break
    case "TupleType": {
      const tupleType = type as AST.TupleType
      tupleType.elementTypes.forEach((t) =>
        collectTypeVariables(t, typeVariables)
      )
      break
    }
    case "UnionType": {
      const unionType = type as AST.UnionType
      unionType.types.forEach((t) => collectTypeVariables(t, typeVariables))
      break
    }
    case "IntersectionType": {
      const intersectionType = type as AST.IntersectionType
      intersectionType.types.forEach((t) =>
        collectTypeVariables(t, typeVariables)
      )
      break
    }
    case "RecordType": {
      const recordType = type as AST.RecordType
      recordType.fields.forEach((field) =>
        collectTypeVariables(field.type, typeVariables)
      )
      break
    }
    case "FunctionType": {
      const functionType = type as AST.FunctionType
      collectTypeVariables(functionType.paramType, typeVariables)
      collectTypeVariables(functionType.returnType, typeVariables)
      break
    }
    case "GenericType": {
      const genericType = type as AST.GenericType
      if (genericType.typeArguments) {
        genericType.typeArguments.forEach((arg) =>
          collectTypeVariables(arg, typeVariables)
        )
      }
      break
    }
  }
}
