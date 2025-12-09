/**
 * 型制約クラス (Type Constraints) for Seseragi Language
 *
 * 型推論で使用される各種制約を定義
 */

import type * as AST from "../ast"
import { formatType } from "./type-formatter"

// 部分型制約を表現するクラス
export class SubtypeConstraint {
  constructor(
    public subType: AST.Type,
    public superType: AST.Type,
    public line: number,
    public column: number,
    public context?: string
  ) {}

  toString(): string {
    return `${formatType(this.subType)} <: ${formatType(this.superType)}`
  }
}

// 型制約を表現するクラス
export class TypeConstraint {
  constructor(
    public type1: AST.Type,
    public type2: AST.Type,
    public line: number,
    public column: number,
    public context?: string
  ) {}

  toString(): string {
    return `${formatType(this.type1)} ~ ${formatType(this.type2)}`
  }
}

// ArrayAccess用の特別な制約クラス
export class ArrayAccessConstraint {
  constructor(
    public arrayType: AST.Type,
    public resultType: AST.Type,
    public line: number,
    public column: number,
    public context?: string
  ) {}

  toString(): string {
    return `ArrayAccess(${formatType(this.arrayType)}) -> ${formatType(this.resultType)}`
  }
}

// FunctorMap用の特別な制約クラス
export class FunctorMapConstraint {
  constructor(
    public containerType: AST.Type,
    public inputType: AST.Type,
    public outputType: AST.Type,
    public resultType: AST.Type,
    public line: number,
    public column: number,
    public context?: string
  ) {}

  toString(): string {
    return `FunctorMap(${formatType(this.containerType)}, ${formatType(this.inputType)} -> ${formatType(this.outputType)}) -> ${formatType(this.resultType)}`
  }
}

// ApplicativeApply用の特別な制約クラス
export class ApplicativeApplyConstraint {
  constructor(
    public funcContainerType: AST.Type,
    public valueContainerType: AST.Type,
    public inputType: AST.Type,
    public outputType: AST.Type,
    public resultType: AST.Type,
    public line: number,
    public column: number,
    public context?: string
  ) {}

  toString(): string {
    return `ApplicativeApply(${formatType(this.funcContainerType)}<${formatType(this.inputType)} -> ${formatType(this.outputType)}>, ${formatType(this.valueContainerType)}<${formatType(this.inputType)}>) -> ${formatType(this.resultType)}`
  }
}
