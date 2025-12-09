/**
 * 制約生成のヘルパー関数群
 */

import * as AST from "../../../ast"
import { freshTypeVariable, type InferenceContext } from "../context"
import { PolymorphicTypeVariable } from "../../type-variables"
import {
  getFreeTypeVariables,
  substituteTypeVariables as substituteTypeVariablesUtil,
} from "../../type-substitution-utils"

/**
 * 多相型変数を具体的な型変数にインスタンス化する
 *
 * 例: forall a. a -> a を t1000 -> t1000 に変換
 */
export function instantiatePolymorphicType(
  ctx: InferenceContext,
  type: AST.Type,
  line: number,
  column: number
): AST.Type {
  const substitutionMap = new Map<string, AST.Type>()

  const substitute = (t: AST.Type): AST.Type => {
    switch (t.kind) {
      case "PolymorphicTypeVariable": {
        const polyVar = t as PolymorphicTypeVariable
        if (!substitutionMap.has(polyVar.name)) {
          substitutionMap.set(polyVar.name, freshTypeVariable(ctx, line, column))
        }
        return substitutionMap.get(polyVar.name)!
      }

      case "FunctionType": {
        const ft = t as AST.FunctionType
        return new AST.FunctionType(
          substitute(ft.paramType),
          substitute(ft.returnType),
          ft.line,
          ft.column
        )
      }

      case "GenericType": {
        const gt = t as AST.GenericType
        return new AST.GenericType(
          gt.name,
          gt.typeArguments.map((arg) => substitute(arg)),
          gt.line,
          gt.column
        )
      }

      case "RecordType": {
        const rt = t as AST.RecordType
        return new AST.RecordType(
          rt.fields.map(
            (field) =>
              new AST.RecordField(
                field.name,
                substitute(field.type),
                field.line,
                field.column
              )
          ),
          rt.line,
          rt.column
        )
      }

      case "StructType": {
        const st = t as AST.StructType
        return new AST.StructType(
          st.name,
          st.fields.map(
            (field) =>
              new AST.RecordField(
                field.name,
                substitute(field.type),
                field.line,
                field.column
              )
          ),
          st.line,
          st.column
        )
      }

      case "TupleType": {
        const tt = t as AST.TupleType
        return new AST.TupleType(
          tt.elementTypes.map((elem) => substitute(elem)),
          tt.line,
          tt.column
        )
      }

      case "UnionType": {
        const ut = t as AST.UnionType
        return new AST.UnionType(
          ut.types.map((member) => substitute(member)),
          ut.line,
          ut.column
        )
      }

      case "IntersectionType": {
        const it = t as AST.IntersectionType
        return new AST.IntersectionType(
          it.types.map((member) => substitute(member)),
          it.line,
          it.column
        )
      }

      default:
        return t
    }
  }

  return substitute(type)
}

/**
 * 型を一般化する（フリー型変数を多相型変数に変換）
 *
 * 関数宣言やラムダ式で使用され、型変数を多相にすることで
 * let多相型推論を実現する
 */
export function generalize(
  type: AST.Type,
  env: Map<string, AST.Type>
): AST.Type {
  const freeVars = getFreeTypeVariables(type, env)
  if (freeVars.size === 0) {
    return type
  }

  const substitutionMap = new Map<string, AST.Type>()
  let polyVarIndex = 0

  // フリー型変数を多相型変数に置換
  for (const varName of freeVars) {
    const polyVarName = String.fromCharCode(97 + polyVarIndex) // 'a', 'b', 'c', ...
    substitutionMap.set(
      varName,
      new PolymorphicTypeVariable(polyVarName, type.line, type.column)
    )
    polyVarIndex++
  }

  return substituteTypeVariablesUtil(type, substitutionMap)
}

/**
 * 型から多相型変数名を収集する
 */
export function collectPolymorphicTypeVariables(type: AST.Type): string[] {
  const vars: string[] = []
  const seen = new Set<string>()

  const collect = (t: AST.Type): void => {
    switch (t.kind) {
      case "PolymorphicTypeVariable": {
        const polyVar = t as PolymorphicTypeVariable
        if (!seen.has(polyVar.name)) {
          seen.add(polyVar.name)
          vars.push(polyVar.name)
        }
        break
      }

      case "FunctionType": {
        const ft = t as AST.FunctionType
        collect(ft.paramType)
        collect(ft.returnType)
        break
      }

      case "GenericType": {
        const gt = t as AST.GenericType
        for (const arg of gt.typeArguments) {
          collect(arg)
        }
        break
      }

      case "RecordType": {
        const rt = t as AST.RecordType
        for (const field of rt.fields) {
          collect(field.type)
        }
        break
      }

      case "StructType": {
        const st = t as AST.StructType
        for (const field of st.fields) {
          collect(field.type)
        }
        break
      }

      case "TupleType": {
        const tt = t as AST.TupleType
        for (const elem of tt.elementTypes) {
          collect(elem)
        }
        break
      }

      case "UnionType": {
        const ut = t as AST.UnionType
        for (const member of ut.types) {
          collect(member)
        }
        break
      }

      case "IntersectionType": {
        const it = t as AST.IntersectionType
        for (const member of it.types) {
          collect(member)
        }
        break
      }
    }
  }

  collect(type)
  return vars
}

/**
 * 型変数を指定されたマップに従って置換する
 */
export function substituteTypeVariables(
  type: AST.Type,
  substitutionMap: Map<string, AST.Type>
): AST.Type {
  const substitute = (t: AST.Type): AST.Type => {
    switch (t.kind) {
      case "PolymorphicTypeVariable": {
        const polyVar = t as PolymorphicTypeVariable
        const replacement = substitutionMap.get(polyVar.name)
        return replacement || t
      }

      case "FunctionType": {
        const ft = t as AST.FunctionType
        return new AST.FunctionType(
          substitute(ft.paramType),
          substitute(ft.returnType),
          ft.line,
          ft.column
        )
      }

      case "GenericType": {
        const gt = t as AST.GenericType
        return new AST.GenericType(
          gt.name,
          gt.typeArguments.map((arg) => substitute(arg)),
          gt.line,
          gt.column
        )
      }

      case "RecordType": {
        const rt = t as AST.RecordType
        return new AST.RecordType(
          rt.fields.map(
            (field) =>
              new AST.RecordField(
                field.name,
                substitute(field.type),
                field.line,
                field.column
              )
          ),
          rt.line,
          rt.column
        )
      }

      case "StructType": {
        const st = t as AST.StructType
        return new AST.StructType(
          st.name,
          st.fields.map(
            (field) =>
              new AST.RecordField(
                field.name,
                substitute(field.type),
                field.line,
                field.column
              )
          ),
          st.line,
          st.column
        )
      }

      case "TupleType": {
        const tt = t as AST.TupleType
        return new AST.TupleType(
          tt.elementTypes.map((elem) => substitute(elem)),
          tt.line,
          tt.column
        )
      }

      case "UnionType": {
        const ut = t as AST.UnionType
        return new AST.UnionType(
          ut.types.map((member) => substitute(member)),
          ut.line,
          ut.column
        )
      }

      case "IntersectionType": {
        const it = t as AST.IntersectionType
        return new AST.IntersectionType(
          it.types.map((member) => substitute(member)),
          it.line,
          it.column
        )
      }

      default:
        return t
    }
  }

  return substitute(type)
}
