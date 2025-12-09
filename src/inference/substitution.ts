/**
 * 型置換クラス (Type Substitution) for Seseragi Language
 *
 * 型推論で使用される型置換を定義
 */

import * as AST from "../ast"
import { TypeConstraint } from "./constraints"
import { formatType } from "./type-formatter"
import type { TypeVariable } from "./type-variables"

// 型置換を表現するクラス
export class TypeSubstitution {
  private substitutions: Map<number, AST.Type> = new Map()

  set(varId: number, type: AST.Type): void {
    this.substitutions.set(varId, type)
  }

  get(varId: number): AST.Type | undefined {
    return this.substitutions.get(varId)
  }

  // 型に置換を適用
  apply(type: AST.Type): AST.Type {
    switch (type.kind) {
      case "TypeVariable": {
        const tv = type as TypeVariable
        const substituted = this.get(tv.id)
        return substituted ? this.apply(substituted) : type
      }

      case "PolymorphicTypeVariable":
        // 多相型変数も制約解決で具体化される場合がある
        // 具体的な使用文脈では具体的な型に解決される
        return type

      case "FunctionType": {
        const ft = type as AST.FunctionType
        return new AST.FunctionType(
          this.apply(ft.paramType),
          this.apply(ft.returnType),
          ft.line,
          ft.column
        )
      }

      case "GenericType": {
        const gt = type as AST.GenericType
        return new AST.GenericType(
          gt.name,
          gt.typeArguments.map((arg) => this.apply(arg)),
          gt.line,
          gt.column
        )
      }

      case "RecordType": {
        const rt = type as AST.RecordType
        return new AST.RecordType(
          rt.fields.map(
            (field) =>
              new AST.RecordField(
                field.name,
                this.apply(field.type),
                field.line,
                field.column
              )
          ),
          rt.line,
          rt.column
        )
      }

      case "TupleType": {
        const tt = type as AST.TupleType
        return new AST.TupleType(
          tt.elementTypes.map((elementType) => this.apply(elementType)),
          tt.line,
          tt.column
        )
      }

      case "StructType": {
        const st = type as AST.StructType
        return new AST.StructType(
          st.name,
          st.fields.map(
            (field) =>
              new AST.RecordField(
                field.name,
                this.apply(field.type),
                field.line,
                field.column
              )
          ),
          st.line,
          st.column
        )
      }

      default:
        return type
    }
  }

  // 制約に置換を適用
  applyToConstraint(constraint: TypeConstraint): TypeConstraint {
    return new TypeConstraint(
      this.apply(constraint.type1),
      this.apply(constraint.type2),
      constraint.line,
      constraint.column,
      constraint.context
    )
  }

  // 置換を合成
  compose(other: TypeSubstitution): TypeSubstitution {
    const result = new TypeSubstitution()

    // 現在の置換を適用
    for (const [varId, type] of this.substitutions) {
      result.set(varId, other.apply(type))
    }

    // 他の置換を追加
    for (const [varId, type] of other.substitutions) {
      if (!result.substitutions.has(varId)) {
        result.set(varId, type)
      }
    }

    return result
  }

  isEmpty(): boolean {
    return this.substitutions.size === 0
  }

  toString(): string {
    const entries = Array.from(this.substitutions.entries())
      .map(([id, type]) => `t${id} := ${formatType(type)}`)
      .join(", ")
    return `[${entries}]`
  }
}
