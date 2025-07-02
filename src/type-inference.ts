/**
 * 型推論システム (Type Inference System) for Seseragi Language
 *
 * Hindley-Milner型推論アルゴリズムを実装
 */

import * as AST from "./ast"

// 型変数を表現するクラス
export class TypeVariable extends AST.Type {
  kind = "TypeVariable"
  name: string
  id: number

  constructor(id: number, line: number, column: number) {
    super(line, column)
    this.id = id
    this.name = `t${id}`
  }
}

// 多相型変数を表現するクラス (例: 'a, 'b)
export class PolymorphicTypeVariable extends AST.Type {
  kind = "PolymorphicTypeVariable"
  name: string

  constructor(name: string, line: number, column: number) {
    super(line, column)
    this.name = name
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
    return `${this.typeToString(this.type1)} ~ ${this.typeToString(this.type2)}`
  }

  private typeToString(type: AST.Type): string {
    switch (type.kind) {
      case "PrimitiveType":
        return (type as AST.PrimitiveType).name
      case "TypeVariable":
        return (type as TypeVariable).name
      case "PolymorphicTypeVariable":
        return `'${(type as PolymorphicTypeVariable).name}`
      case "FunctionType": {
        const ft = type as AST.FunctionType
        return `(${this.typeToString(ft.paramType)} -> ${this.typeToString(ft.returnType)})`
      }
      case "GenericType": {
        const gt = type as AST.GenericType
        const args = gt.typeArguments
          .map((t) => this.typeToString(t))
          .join(", ")
        return `${gt.name}<${args}>`
      }
      case "RecordType": {
        const rt = type as AST.RecordType
        const fields = rt.fields
          .map((field) => `${field.name}: ${this.typeToString(field.type)}`)
          .join(", ")
        return `{${fields}}`
      }
      case "TupleType": {
        const tupleType = type as AST.TupleType
        const elements = tupleType.elementTypes
          .map((elementType) => this.typeToString(elementType))
          .join(", ")
        return `(${elements})`
      }
      case "StructType": {
        const st = type as AST.StructType
        return st.name
      }
      default:
        return "Unknown"
    }
  }
}

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
        // 多相型変数は置換しない（常に多相のまま）
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
      .map(([id, type]) => `t${id} := ${this.typeToString(type)}`)
      .join(", ")
    return `[${entries}]`
  }

  private typeToString(type: AST.Type): string {
    switch (type.kind) {
      case "PrimitiveType":
        return (type as AST.PrimitiveType).name
      case "TypeVariable":
        return (type as TypeVariable).name
      case "PolymorphicTypeVariable":
        return `'${(type as PolymorphicTypeVariable).name}`
      case "FunctionType": {
        const ft = type as AST.FunctionType
        return `(${this.typeToString(ft.paramType)} -> ${this.typeToString(ft.returnType)})`
      }
      case "GenericType": {
        const gt = type as AST.GenericType
        const args = gt.typeArguments
          .map((t) => this.typeToString(t))
          .join(", ")
        return `${gt.name}<${args}>`
      }
      case "StructType":
        return (type as AST.StructType).name
      case "RecordType": {
        const rt = type as AST.RecordType
        const fields = rt.fields
          .map((f) => `${f.name}: ${this.typeToString(f.type)}`)
          .join(", ")
        return `{ ${fields} }`
      }
      case "TupleType": {
        const tt = type as AST.TupleType
        const elements = tt.elementTypes
          .map((t) => this.typeToString(t))
          .join(", ")
        return `(${elements})`
      }
      default:
        return "Unknown"
    }
  }
}

// 型推論エラークラス
export class TypeInferenceError {
  constructor(
    public message: string,
    public line: number,
    public column: number,
    public context?: string
  ) {}

  toString(): string {
    let result = `Type inference error at line ${this.line}, column ${this.column}: ${this.message}`
    if (this.context) {
      result += `\n  Context: ${this.context}`
    }
    return result
  }
}

// 型推論結果インターフェース
export interface InferenceResult {
  errors: TypeInferenceError[]
  inferredTypes?: Map<string, AST.Type>
  typeEnvironment?: Map<string, AST.Type>
}

// 型推論システムのメインクラス
export class TypeInferenceSystem {
  private nextVarId = 1000 // Start from 1000 to avoid conflicts with parser-generated type variables
  private constraints: TypeConstraint[] = []
  private errors: TypeInferenceError[] = []
  private nodeTypeMap: Map<any, AST.Type> = new Map() // Track types for AST nodes
  private methodEnvironment: Map<string, AST.MethodDeclaration> = new Map() // Track methods by type.method

  // 新しい型変数を生成
  freshTypeVariable(line: number, column: number): TypeVariable {
    return new TypeVariable(this.nextVarId++, line, column)
  }

  private formatType(
    type: AST.Type | TypeVariable | PolymorphicTypeVariable | null | undefined
  ): string {
    if (!type) {
      return "null"
    }

    switch (type.kind) {
      case "PrimitiveType":
        return (type as AST.PrimitiveType).name
      case "FunctionType": {
        const ft = type as AST.FunctionType
        return `(${this.formatType(ft.paramType)} -> ${this.formatType(ft.returnType)})`
      }
      case "TypeVariable":
        if (type instanceof TypeVariable) {
          return type.name
        } else {
          return (type as AST.TypeVariable).name
        }
      case "PolymorphicTypeVariable":
        return (type as PolymorphicTypeVariable).name
      case "GenericType": {
        const gt = type as AST.GenericType
        if (gt.typeArguments.length > 0) {
          return `${gt.name}<${gt.typeArguments.map((t) => this.formatType(t)).join(", ")}>`
        }
        return gt.name
      }
      case "TupleType": {
        const tt = type as AST.TupleType
        return `(${tt.elementTypes.map((t) => this.formatType(t)).join(", ")})`
      }
      case "RecordType": {
        const rt = type as AST.RecordType
        const fields = rt.fields
          .map((f) => `${f.name}: ${this.formatType(f.type)}`)
          .join(", ")
        return `{ ${fields} }`
      }
      case "StructType": {
        const st = type as AST.StructType
        return st.name
      }
      default:
        return `UnknownType(${type.kind})`
    }
  }

  // 型の一般化（generalization）- フリー型変数を多相型変数に変換
  generalize(type: AST.Type, env: Map<string, AST.Type>): AST.Type {
    const freeVars = this.getFreeTypeVariables(type, env)
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

    return this.substituteTypeVariables(type, substitutionMap)
  }

  // フリー型変数を取得
  private getFreeTypeVariables(
    type: AST.Type,
    env: Map<string, AST.Type>
  ): Set<string> {
    const freeVars = new Set<string>()

    const collect = (t: AST.Type): void => {
      switch (t.kind) {
        case "TypeVariable": {
          const tv = t as TypeVariable
          // 環境に束縛されていない型変数のみを収集
          if (!this.isTypeVariableBoundInEnv(tv.name, env)) {
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

  // 型変数が環境に束縛されているかチェック
  private isTypeVariableBoundInEnv(
    varName: string,
    env: Map<string, AST.Type>
  ): boolean {
    for (const [_, envType] of env) {
      if (this.typeContainsVariable(envType, varName)) {
        return true
      }
    }
    return false
  }

  // 型に特定の型変数が含まれているかチェック
  private typeContainsVariable(type: AST.Type, varName: string): boolean {
    switch (type.kind) {
      case "TypeVariable":
        return (type as TypeVariable).name === varName
      case "FunctionType": {
        const ft = type as AST.FunctionType
        return (
          this.typeContainsVariable(ft.paramType, varName) ||
          this.typeContainsVariable(ft.returnType, varName)
        )
      }
      case "TupleType": {
        const tt = type as AST.TupleType
        return tt.elementTypes.some((t) =>
          this.typeContainsVariable(t, varName)
        )
      }
      case "GenericType": {
        const gt = type as AST.GenericType
        return gt.typeArguments.some((t) =>
          this.typeContainsVariable(t, varName)
        )
      }
      case "RecordType": {
        const rt = type as AST.RecordType
        return rt.fields.some((f) => this.typeContainsVariable(f.type, varName))
      }
      case "StructType": {
        const st = type as AST.StructType
        return st.fields.some((f) => this.typeContainsVariable(f.type, varName))
      }
      default:
        return false
    }
  }

  // 型変数の置換
  private substituteTypeVariables(
    type: AST.Type,
    substitutionMap: Map<string, AST.Type>
  ): AST.Type {
    switch (type.kind) {
      case "TypeVariable": {
        const tv = type as TypeVariable
        return substitutionMap.get(tv.name) || type
      }
      case "FunctionType": {
        const ft = type as AST.FunctionType
        return new AST.FunctionType(
          this.substituteTypeVariables(ft.paramType, substitutionMap),
          this.substituteTypeVariables(ft.returnType, substitutionMap),
          ft.line,
          ft.column
        )
      }
      case "TupleType": {
        const tt = type as AST.TupleType
        return new AST.TupleType(
          tt.elementTypes.map((t) =>
            this.substituteTypeVariables(t, substitutionMap)
          ),
          tt.line,
          tt.column
        )
      }
      case "GenericType": {
        const gt = type as AST.GenericType
        return new AST.GenericType(
          gt.name,
          gt.typeArguments.map((t) =>
            this.substituteTypeVariables(t, substitutionMap)
          ),
          gt.line,
          gt.column
        )
      }
      case "RecordType": {
        const rt = type as AST.RecordType
        return new AST.RecordType(
          rt.fields.map((f) => ({
            name: f.name,
            type: this.substituteTypeVariables(f.type, substitutionMap),
          })),
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
                this.substituteTypeVariables(f.type, substitutionMap),
                f.line,
                f.column
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

  // 多相型を具体化（インスタンス化）
  instantiatePolymorphicType(
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
            substitutionMap.set(
              polyVar.name,
              this.freshTypeVariable(line, column)
            )
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

        default:
          return t
      }
    }

    return substitute(type)
  }

  // 制約を追加
  addConstraint(constraint: TypeConstraint): void {
    // console.log(`➕ Adding constraint: ${constraint.toString()} at ${constraint.line}:${constraint.column}${constraint.context ? ` (${constraint.context})` : ''}`)
    this.constraints.push(constraint)
  }

  // 型推論のメインエントリーポイント
  infer(program: AST.Program): {
    substitution: TypeSubstitution
    errors: TypeInferenceError[]
    nodeTypeMap: Map<any, AST.Type>
  } {
    this.constraints = []
    this.currentEnvironment.clear() // 環境をクリア
    this.errors = []
    this.nextVarId = 1000 // Reset to 1000 to avoid conflicts with parser-generated type variables
    this.nodeTypeMap.clear()

    // 型環境の初期化
    const env = this.createInitialEnvironment()

    // 制約生成
    this.generateConstraints(program, env)

    // 制約解決（単一化）
    const substitution = this.solveConstraints()

    // Apply substitution to all tracked node types
    const resolvedNodeTypeMap = new Map<any, AST.Type>()
    for (const [node, type] of this.nodeTypeMap) {
      resolvedNodeTypeMap.set(node, substitution.apply(type))
    }

    return {
      substitution,
      errors: this.errors,
      nodeTypeMap: resolvedNodeTypeMap,
    }
  }

  // 初期型環境を作成
  private createInitialEnvironment(): Map<string, AST.Type> {
    const env = new Map<string, AST.Type>()

    // 組み込み関数の型を定義

    // print: 'a -> Unit (多相関数)
    const printType = new AST.FunctionType(
      new PolymorphicTypeVariable("a", 0, 0),
      new AST.PrimitiveType("Unit", 0, 0),
      0,
      0
    )
    env.set("print", printType)

    // putStrLn: 'a -> Unit (多相関数)
    const putStrLnType = new AST.FunctionType(
      new PolymorphicTypeVariable("a", 0, 0),
      new AST.PrimitiveType("Unit", 0, 0),
      0,
      0
    )
    env.set("putStrLn", putStrLnType)

    // toString: 'a -> String (多相関数)
    const toStringType = new AST.FunctionType(
      new PolymorphicTypeVariable("a", 0, 0),
      new AST.PrimitiveType("String", 0, 0),
      0,
      0
    )
    env.set("toString", toStringType)

    // show: 'a -> Unit (多相関数)
    const showType = new AST.FunctionType(
      new PolymorphicTypeVariable("a", 0, 0),
      new AST.PrimitiveType("Unit", 0, 0),
      0,
      0
    )
    env.set("show", showType)

    // arrayToList: Array<'a> -> List<'a>
    const aTypeVar = new PolymorphicTypeVariable("a", 0, 0)
    const arrayToListType = new AST.FunctionType(
      new AST.GenericType("Array", [aTypeVar], 0, 0),
      new AST.GenericType("List", [aTypeVar], 0, 0),
      0,
      0
    )
    env.set("arrayToList", arrayToListType)

    // listToArray: List<'a> -> Array<'a>
    const bTypeVar = new PolymorphicTypeVariable("b", 0, 0)
    const listToArrayType = new AST.FunctionType(
      new AST.GenericType("List", [bTypeVar], 0, 0),
      new AST.GenericType("Array", [bTypeVar], 0, 0),
      0,
      0
    )
    env.set("listToArray", listToArrayType)

    // List constructors for pattern matching and expressions
    // Empty : List<'a>
    const emptyTypeVar = new PolymorphicTypeVariable("a", 0, 0)
    const emptyType = new AST.GenericType("List", [emptyTypeVar], 0, 0)
    env.set("Empty", emptyType)

    // Cons : 'a -> List<'a> -> List<'a>
    const consTypeVar = new PolymorphicTypeVariable("a", 0, 0)
    const consListType = new AST.GenericType("List", [consTypeVar], 0, 0)
    const consType = new AST.FunctionType(
      consTypeVar,
      new AST.FunctionType(consListType, consListType, 0, 0),
      0,
      0
    )
    env.set("Cons", consType)

    // Maybe constructors for pattern matching and expressions
    // Nothing : Maybe<'a>
    const nothingTypeVar = new PolymorphicTypeVariable("a", 0, 0)
    const nothingType = new AST.GenericType("Maybe", [nothingTypeVar], 0, 0)
    env.set("Nothing", nothingType)

    // Just : 'a -> Maybe<'a>
    const justTypeVar = new PolymorphicTypeVariable("a", 0, 0)
    const justMaybeType = new AST.GenericType("Maybe", [justTypeVar], 0, 0)
    const justType = new AST.FunctionType(justTypeVar, justMaybeType, 0, 0)
    env.set("Just", justType)

    // Either constructors for pattern matching and expressions
    // Left : 'a -> Either<'a, 'b>
    const leftTypeVar = new PolymorphicTypeVariable("a", 0, 0)
    const leftRightTypeVar = new PolymorphicTypeVariable("b", 0, 0)
    const leftEitherType = new AST.GenericType(
      "Either",
      [leftTypeVar, leftRightTypeVar],
      0,
      0
    )
    const leftType = new AST.FunctionType(leftTypeVar, leftEitherType, 0, 0)
    env.set("Left", leftType)

    // Right : 'b -> Either<'a, 'b>
    const rightLeftTypeVar = new PolymorphicTypeVariable("a", 0, 0)
    const rightTypeVar = new PolymorphicTypeVariable("b", 0, 0)
    const rightEitherType = new AST.GenericType(
      "Either",
      [rightLeftTypeVar, rightTypeVar],
      0,
      0
    )
    const rightType = new AST.FunctionType(rightTypeVar, rightEitherType, 0, 0)
    env.set("Right", rightType)

    // Boolean constants
    const boolType = new AST.PrimitiveType("Bool", 0, 0)
    env.set("true", boolType)
    env.set("false", boolType)

    return env
  }

  // 制約生成
  private generateConstraints(
    program: AST.Program,
    env: Map<string, AST.Type>
  ): void {
    // 現在の環境を設定（型エイリアス解決用）
    this.currentEnvironment = env

    // Two-pass approach to handle forward references:
    // Pass 1: Process all function declarations, type declarations, and struct declarations first
    // This allows variables to reference functions and types defined later in the file
    for (const statement of program.statements) {
      if (
        statement.kind === "FunctionDeclaration" ||
        statement.kind === "TypeDeclaration" ||
        statement.kind === "TypeAliasDeclaration" ||
        statement.kind === "StructDeclaration"
      ) {
        this.generateConstraintsForStatement(statement, env)
      }
    }

    // Pass 2: Process impl blocks, variable declarations and expression statements in original order
    // At this point all functions and types are available in the environment
    for (const statement of program.statements) {
      if (statement.kind === "ImplBlock") {
        this.generateConstraintsForStatement(statement, env)
      }
    }

    // Pass 3: Process variable declarations and expression statements
    for (const statement of program.statements) {
      if (
        statement.kind === "VariableDeclaration" ||
        statement.kind === "ExpressionStatement" ||
        statement.kind === "TupleDestructuring" ||
        statement.kind === "RecordDestructuring" ||
        statement.kind === "StructDestructuring"
      ) {
        this.generateConstraintsForStatement(statement, env)
      }
    }
  }

  private generateConstraintsForStatement(
    statement: AST.Statement,
    env: Map<string, AST.Type>
  ): void {
    switch (statement.kind) {
      case "FunctionDeclaration":
        this.generateConstraintsForFunctionDeclaration(
          statement as AST.FunctionDeclaration,
          env
        )
        break
      case "VariableDeclaration":
        this.generateConstraintsForVariableDeclaration(
          statement as AST.VariableDeclaration,
          env
        )
        break
      case "ExpressionStatement":
        this.generateConstraintsForExpression(
          (statement as AST.ExpressionStatement).expression,
          env
        )
        break
      case "TypeDeclaration":
        this.generateConstraintsForTypeDeclaration(
          statement as AST.TypeDeclaration,
          env
        )
        break
      case "TypeAliasDeclaration":
        this.generateConstraintsForTypeAliasDeclaration(
          statement as AST.TypeAliasDeclaration,
          env
        )
        break
      case "TupleDestructuring":
        this.generateConstraintsForTupleDestructuring(
          statement as AST.TupleDestructuring,
          env
        )
        break
      case "RecordDestructuring":
        this.generateConstraintsForRecordDestructuring(
          statement as AST.RecordDestructuring,
          env
        )
        break
      case "StructDestructuring":
        this.generateConstraintsForStructDestructuring(
          statement as AST.StructDestructuring,
          env
        )
        break
      case "StructDeclaration":
        this.generateConstraintsForStructDeclaration(
          statement as AST.StructDeclaration,
          env
        )
        break
      case "ImplBlock":
        this.generateConstraintsForImplBlock(statement as AST.ImplBlock, env)
        break
      default:
        // 他の文の種類は後で実装
        break
    }
  }

  private generateConstraintsForFunctionDeclaration(
    func: AST.FunctionDeclaration,
    env: Map<string, AST.Type>
  ): void {
    // 戻り値の型が指定されていない場合は型変数を作成
    const returnType =
      func.returnType || this.freshTypeVariable(func.line, func.column)

    // パラメータの型を事前に決定
    const paramTypes: AST.Type[] = []
    for (const param of func.parameters) {
      let paramType: AST.Type

      if (param.type) {
        paramType = param.type
      } else {
        paramType = this.freshTypeVariable(param.line, param.column)
      }

      paramTypes.push(paramType)
    }

    // 関数の型を構築
    let funcType: AST.Type = returnType

    // パラメータから関数型を構築（カリー化）
    if (func.parameters.length === 0) {
      // 引数なしの関数は Unit -> ReturnType
      const unitType = new AST.PrimitiveType("Unit", func.line, func.column)
      funcType = new AST.FunctionType(
        unitType,
        funcType,
        func.line,
        func.column
      )
    } else {
      // 引数ありの関数は通常のカリー化（後ろから前に構築）
      for (let i = paramTypes.length - 1; i >= 0; i--) {
        funcType = new AST.FunctionType(
          paramTypes[i],
          funcType,
          func.line,
          func.column
        )
      }
    }

    // 関数を環境に追加
    const generalizedType = this.generalize(funcType, env)
    // console.log(`🔧 Function '${func.name}' generalized from ${this.typeToString(funcType)} to ${this.typeToString(generalizedType)}`)
    env.set(func.name, generalizedType)

    // 関数本体の型推論用の環境を作成
    const bodyEnv = new Map(env)

    // パラメータの型を環境に追加（型エイリアス解決後）
    for (let i = 0; i < func.parameters.length; i++) {
      const param = func.parameters[i]
      const paramType = paramTypes[i]

      // パラメータ型も型エイリアス解決を行う
      const resolvedParamType = this.resolveTypeAlias(paramType)
      bodyEnv.set(param.name, resolvedParamType)
    }

    // 関数本体の型を推論
    const bodyType = this.generateConstraintsForExpression(func.body, bodyEnv)

    // 関数本体の型と戻り値型が一致することを制約として追加
    this.addConstraint(
      new TypeConstraint(
        bodyType,
        returnType,
        func.body.line,
        func.body.column,
        `Function ${func.name} body type`
      )
    )
  }

  private generateConstraintsForVariableDeclaration(
    varDecl: AST.VariableDeclaration,
    env: Map<string, AST.Type>
  ): AST.Type {
    // 初期化式の型を推論
    const initType = this.generateConstraintsForExpression(
      varDecl.initializer,
      env
    )

    let finalType: AST.Type
    if (varDecl.type) {
      // 明示的な型注釈がある場合は型エイリアスを解決
      let resolvedType = varDecl.type
      if (varDecl.type.kind === "PrimitiveType") {
        const aliasedType = env.get(varDecl.type.name)
        if (aliasedType) {
          resolvedType = aliasedType
        }
      }
      
      // 制約を追加（解決された型で）
      this.addConstraint(
        new TypeConstraint(
          initType,
          resolvedType,
          varDecl.line,
          varDecl.column,
          `Variable ${varDecl.name} type annotation`
        )
      )
      env.set(varDecl.name, resolvedType)
      finalType = resolvedType
    } else {
      // 型注釈がない場合は推論された型を使用
      // 型変数、ジェネリック型、関数型すべてを正しく保持
      env.set(varDecl.name, initType)
      finalType = initType
    }

    // Track the type for this variable declaration
    this.nodeTypeMap.set(varDecl, finalType)

    // Also track the initializer expression type
    this.nodeTypeMap.set(varDecl.initializer, initType)

    return finalType
  }

  private generateConstraintsForTupleDestructuring(
    tupleDestr: AST.TupleDestructuring,
    env: Map<string, AST.Type>
  ): void {
    // 初期化式の型を推論
    const initType = this.generateConstraintsForExpression(
      tupleDestr.initializer,
      env
    )

    // タプルパターンを処理して変数を環境に追加
    this.generateConstraintsForPattern(tupleDestr.pattern, initType, env)

    // ノード型マップに情報を記録
    this.nodeTypeMap.set(tupleDestr, initType)
    this.nodeTypeMap.set(tupleDestr.initializer, initType)
  }

  private generateConstraintsForTypeDeclaration(
    typeDecl: AST.TypeDeclaration,
    env: Map<string, AST.Type>
  ): void {
    // ADT型を環境に追加
    const adtType = new AST.PrimitiveType(
      typeDecl.name,
      typeDecl.line,
      typeDecl.column
    )

    // ADT型自体を環境に登録
    env.set(typeDecl.name, adtType)

    // 各バリアント（コンストラクタ）を環境に追加
    for (const field of typeDecl.fields) {
      const constructorType = this.createConstructorType(field, adtType)
      env.set(field.name, constructorType)
    }
  }

  private generateConstraintsForTypeAliasDeclaration(
    typeAlias: AST.TypeAliasDeclaration,
    env: Map<string, AST.Type>
  ): void {
    // 型エイリアスを環境に追加
    // エイリアスされる型がそのままエイリアス名で参照される
    env.set(typeAlias.name, typeAlias.aliasedType)

    // 型エイリアス解決用の環境にも追加
    this.currentEnvironment.set(typeAlias.name, typeAlias.aliasedType)
  }

  private createConstructorType(
    field: AST.TypeField,
    adtType: AST.Type
  ): AST.Type {
    if (field.type instanceof AST.PrimitiveType && field.type.name === "Unit") {
      // データなしのコンストラクタ (Red, Green, Blue)
      return adtType
    } else if (
      field.type instanceof AST.GenericType &&
      field.type.name === "Tuple"
    ) {
      // データ付きのコンストラクタ (RGB Int Int Int)
      let resultType = adtType

      // 型引数から逆順でカリー化された関数型を構築
      for (let i = field.type.typeArguments.length - 1; i >= 0; i--) {
        const paramType = field.type.typeArguments[i]
        resultType = new AST.FunctionType(
          paramType,
          resultType,
          field.line,
          field.column
        )
      }

      return resultType
    } else {
      // その他のケース（単一データ）
      return new AST.FunctionType(field.type, adtType, field.line, field.column)
    }
  }

  public generateConstraintsForExpression(
    expr: AST.Expression,
    env: Map<string, AST.Type>
  ): AST.Type {
    let resultType: AST.Type

    switch (expr.kind) {
      case "Literal":
        resultType = this.generateConstraintsForLiteral(expr as AST.Literal)
        break

      case "Identifier":
        resultType = this.generateConstraintsForIdentifier(
          expr as AST.Identifier,
          env
        )
        break

      case "BinaryOperation":
        resultType = this.generateConstraintsForBinaryOperation(
          expr as AST.BinaryOperation,
          env
        )
        break

      case "UnaryOperation":
        resultType = this.generateConstraintsForUnaryOperation(
          expr as AST.UnaryOperation,
          env
        )
        break

      case "FunctionCall":
        resultType = this.generateConstraintsForFunctionCall(
          expr as AST.FunctionCall,
          env
        )
        break

      case "MethodCall":
        resultType = this.generateConstraintsForMethodCall(
          expr as AST.MethodCall,
          env
        )
        break

      case "BuiltinFunctionCall":
        resultType = this.generateConstraintsForBuiltinFunctionCall(
          expr as AST.BuiltinFunctionCall,
          env
        )
        break

      case "FunctionApplication":
        resultType = this.generateConstraintsForFunctionApplication(
          expr as AST.FunctionApplication,
          env
        )
        break

      case "Pipeline":
        resultType = this.generateConstraintsForPipeline(
          expr as AST.Pipeline,
          env
        )
        break

      case "ConditionalExpression":
        resultType = this.generateConstraintsForConditional(
          expr as AST.ConditionalExpression,
          env
        )
        break

      case "TernaryExpression":
        resultType = this.generateConstraintsForTernary(
          expr as AST.TernaryExpression,
          env
        )
        break

      case "BlockExpression":
        resultType = this.generateConstraintsForBlockExpression(
          expr as AST.BlockExpression,
          env
        )
        break

      case "ConstructorExpression":
        resultType = this.generateConstraintsForConstructorExpression(
          expr as AST.ConstructorExpression,
          env
        )
        break

      case "FunctorMap":
        resultType = this.generateConstraintsForFunctorMap(
          expr as AST.FunctorMap,
          env
        )
        break

      case "ApplicativeApply":
        resultType = this.generateConstraintsForApplicativeApply(
          expr as AST.ApplicativeApply,
          env
        )
        break

      case "MonadBind":
        resultType = this.generateConstraintsForMonadBind(
          expr as AST.MonadBind,
          env
        )
        break

      case "RangeLiteral":
        resultType = this.generateConstraintsForRangeLiteral(
          expr as AST.RangeLiteral,
          env
        )
        break

      case "ListComprehension":
        resultType = this.generateConstraintsForListComprehension(
          expr as AST.ListComprehension,
          env
        )
        break

      case "ListComprehensionSugar":
        resultType = this.generateConstraintsForListComprehensionSugar(
          expr as AST.ListComprehensionSugar,
          env
        )
        break

      case "FunctionApplicationOperator":
        resultType = this.generateConstraintsForFunctionApplicationOperator(
          expr as AST.FunctionApplicationOperator,
          env
        )
        break

      case "LambdaExpression":
        resultType = this.generateConstraintsForLambdaExpression(
          expr as AST.LambdaExpression,
          env
        )
        break

      case "MatchExpression":
        resultType = this.generateConstraintsForMatchExpression(
          expr as AST.MatchExpression,
          env
        )
        break

      case "RecordExpression":
        resultType = this.generateConstraintsForRecordExpression(
          expr as AST.RecordExpression,
          env
        )
        break

      case "RecordAccess":
        resultType = this.generateConstraintsForRecordAccess(
          expr as AST.RecordAccess,
          env
        )
        break

      case "ArrayLiteral":
        resultType = this.generateConstraintsForArrayLiteral(
          expr as AST.ArrayLiteral,
          env
        )
        break

      case "ArrayAccess":
        resultType = this.generateConstraintsForArrayAccess(
          expr as AST.ArrayAccess,
          env
        )
        break

      case "ListSugar":
        resultType = this.generateConstraintsForListSugar(
          expr as AST.ListSugar,
          env
        )
        break

      case "ConsExpression":
        resultType = this.generateConstraintsForConsExpression(
          expr as AST.ConsExpression,
          env
        )
        break

      case "TupleExpression":
        resultType = this.generateConstraintsForTupleExpression(
          expr as AST.TupleExpression,
          env
        )
        break

      case "StructExpression":
        resultType = this.generateConstraintsForStructExpression(
          expr as AST.StructExpression,
          env
        )
        break

      case "SpreadExpression":
        resultType = this.generateConstraintsForSpreadExpression(
          expr as AST.SpreadExpression,
          env
        )
        break

      default:
        this.errors.push(
          new TypeInferenceError(
            `Unhandled expression type: ${expr.kind}`,
            expr.line,
            expr.column
          )
        )
        resultType = this.freshTypeVariable(expr.line, expr.column)
        break
    }

    // Track the type for this expression
    this.nodeTypeMap.set(expr, resultType)
    return resultType
  }

  private generateConstraintsForLiteral(literal: AST.Literal): AST.Type {
    switch (literal.literalType) {
      case "string":
        return new AST.PrimitiveType("String", literal.line, literal.column)
      case "integer":
        return new AST.PrimitiveType("Int", literal.line, literal.column)
      case "float":
        return new AST.PrimitiveType("Float", literal.line, literal.column)
      case "boolean":
        return new AST.PrimitiveType("Bool", literal.line, literal.column)
      default:
        return this.freshTypeVariable(literal.line, literal.column)
    }
  }

  private generateConstraintsForIdentifier(
    identifier: AST.Identifier,
    env: Map<string, AST.Type>
  ): AST.Type {
    // Normal identifier lookup
    const type = env.get(identifier.name)
    if (!type) {
      this.errors.push(
        new TypeInferenceError(
          `Undefined variable: ${identifier.name}`,
          identifier.line,
          identifier.column
        )
      )
      return this.freshTypeVariable(identifier.line, identifier.column)
    }

    // Debug logging for makeTuple function type resolution
    // if (identifier.name === 'makeTuple') {
    //   console.log(`DEBUG: makeTuple type resolution at ${identifier.line}:${identifier.column}`)
    //   console.log(`DEBUG: Original type from env:`, this.typeToString(type))
    //   const instantiated = this.instantiatePolymorphicType(type, identifier.line, identifier.column)
    //   console.log(`DEBUG: Instantiated type:`, this.typeToString(instantiated))
    //   return instantiated
    // }

    // Instantiate polymorphic types when looking up from environment
    return this.instantiatePolymorphicType(
      type,
      identifier.line,
      identifier.column
    )
  }

  private generateConstraintsForBinaryOperation(
    binOp: AST.BinaryOperation,
    env: Map<string, AST.Type>
  ): AST.Type {
    const leftType = this.generateConstraintsForExpression(binOp.left, env)
    const rightType = this.generateConstraintsForExpression(binOp.right, env)

    // left/rightの式に型情報を設定
    binOp.left.type = leftType
    binOp.right.type = rightType

    switch (binOp.operator) {
      case "+":
        // + 演算子は数値演算か文字列結合のどちらか
        // より具体的な制約生成で型安全性を保つ

        // 左右のオペランドが同じ型である制約
        this.addConstraint(
          new TypeConstraint(
            leftType,
            rightType,
            binOp.line,
            binOp.column,
            `Binary operation + operands must have same type`
          )
        )

        // 結果の型は左のオペランドと同じ型
        return leftType

      case "-":
      case "*":
      case "/":
      case "%": {
        // 数値演算: 両オペランドは同じ型でなければならず、結果も同じ型
        // ただし、構造体型が関わっている場合は演算子オーバーロードの可能性があるため制約を緩める

        // 構造体型が関わっているかチェック
        const hasStructType =
          this.isStructOrResolvesToStruct(leftType, env) ||
          this.isStructOrResolvesToStruct(rightType, env)

        if (!hasStructType) {
          // 通常の数値演算の場合のみ、左右のオペランドが同じ型である制約を追加
          this.addConstraint(
            new TypeConstraint(
              leftType,
              rightType,
              binOp.line,
              binOp.column,
              `Binary operation ${binOp.operator} operands must have same type`
            )
          )
        }

        // 結果の型は左のオペランドと同じ型（構造体の場合は演算子オーバーロードで決まる）
        return leftType
      }

      case "==":
      case "!=":
      case "<":
      case ">":
      case "<=":
      case ">=":
        // 比較演算: 両オペランドは同じ型、結果はBool
        this.addConstraint(
          new TypeConstraint(
            leftType,
            rightType,
            binOp.line,
            binOp.column,
            `Comparison ${binOp.operator} operands must match`
          )
        )
        return new AST.PrimitiveType("Bool", binOp.line, binOp.column)

      case "&&":
      case "||": {
        // 論理演算: 基本的にはBool、ただし構造体のオーバーロードも考慮
        const hasStructTypeLogical =
          this.isStructOrResolvesToStruct(leftType, env) ||
          this.isStructOrResolvesToStruct(rightType, env)

        if (hasStructTypeLogical) {
          // 構造体のオーバーロードが考えられる場合: 左右のオペランドが同じ型である制約のみ
          this.addConstraint(
            new TypeConstraint(
              leftType,
              rightType,
              binOp.line,
              binOp.column,
              `Logical operation ${binOp.operator} operands must have same type (struct overload)`
            )
          )
          // 結果の型は演算子オーバーロードによって決まるが、一般的にはBoolを返す
          // 構造体オーバーロードでは通常Bool型を返すことが多い
          return new AST.PrimitiveType("Bool", binOp.line, binOp.column)
        } else {
          // 通常の論理演算: 両オペランドはBool、結果もBool
          const boolType = new AST.PrimitiveType(
            "Bool",
            binOp.line,
            binOp.column
          )
          this.addConstraint(
            new TypeConstraint(
              leftType,
              boolType,
              binOp.left.line,
              binOp.left.column,
              `Logical operation ${binOp.operator} left operand`
            )
          )
          this.addConstraint(
            new TypeConstraint(
              rightType,
              boolType,
              binOp.right.line,
              binOp.right.column,
              `Logical operation ${binOp.operator} right operand`
            )
          )
          return boolType
        }
      }

      case ":": {
        // CONS演算子: a : List<a> -> List<a>
        // 左オペランドは要素、右オペランドはリスト、結果は同じ要素型のリスト
        const expectedListType = new AST.GenericType(
          "List",
          [leftType],
          binOp.right.line,
          binOp.right.column
        )

        // 右オペランドはList<leftType>型でなければならない
        this.addConstraint(
          new TypeConstraint(
            rightType,
            expectedListType,
            binOp.right.line,
            binOp.right.column,
            `CONS operator (:) right operand must be List<${this.typeToString(leftType)}>`
          )
        )

        // 結果の型もList<leftType>
        return expectedListType
      }

      default:
        this.errors.push(
          new TypeInferenceError(
            `Unknown binary operator: ${binOp.operator}`,
            binOp.line,
            binOp.column
          )
        )
        return this.freshTypeVariable(binOp.line, binOp.column)
    }
  }

  private generateConstraintsForUnaryOperation(
    unaryOp: AST.UnaryOperation,
    env: Map<string, AST.Type>
  ): AST.Type {
    const operandType = this.generateConstraintsForExpression(
      unaryOp.operand,
      env
    )

    switch (unaryOp.operator) {
      case "-": {
        // 数値の単項マイナス: Int -> Int, Float -> Float
        const intType = new AST.PrimitiveType(
          "Int",
          unaryOp.line,
          unaryOp.column
        )
        const floatType = new AST.PrimitiveType(
          "Float",
          unaryOp.line,
          unaryOp.column
        )

        // まず Int 型として制約を追加
        this.addConstraint(
          new TypeConstraint(
            operandType,
            intType,
            unaryOp.operand.line,
            unaryOp.operand.column,
            `Unary minus operand (Int)`
          )
        )
        return intType
      }

      case "!": {
        // 論理否定: Bool -> Bool
        const boolType = new AST.PrimitiveType(
          "Bool",
          unaryOp.line,
          unaryOp.column
        )
        this.addConstraint(
          new TypeConstraint(
            operandType,
            boolType,
            unaryOp.operand.line,
            unaryOp.operand.column,
            `Logical negation operand`
          )
        )
        return boolType
      }

      default:
        this.errors.push(
          new TypeInferenceError(
            `Unknown unary operator: ${unaryOp.operator}`,
            unaryOp.line,
            unaryOp.column
          )
        )
        return this.freshTypeVariable(unaryOp.line, unaryOp.column)
    }
  }

  private generateConstraintsForFunctionCall(
    call: AST.FunctionCall,
    env: Map<string, AST.Type>
  ): AST.Type {
    // print/putStrLn関数の特別処理
    if (call.function.kind === "Identifier") {
      const funcName = (call.function as AST.Identifier).name
      if (
        (funcName === "print" || funcName === "putStrLn") &&
        call.arguments.length === 1
      ) {
        // print/putStrLn関数は任意の型を受け取り、Unit型を返す
        this.generateConstraintsForExpression(call.arguments[0], env)
        return new AST.PrimitiveType("Unit", call.line, call.column)
      }
    }

    const funcType = this.generateConstraintsForExpression(call.function, env)

    // 引数が0個の場合は、関数がユニット型を取る関数として扱う
    if (call.arguments.length === 0) {
      // 関数の型が既知の場合、その戻り値型を抽出
      if (funcType.kind === "FunctionType") {
        const ft = funcType as AST.FunctionType
        // Unit -> ReturnType の形を期待
        const expectedFuncType = new AST.FunctionType(
          new AST.PrimitiveType("Unit", call.line, call.column),
          ft.returnType, // 既存の戻り値型を使用
          call.line,
          call.column
        )

        this.addConstraint(
          new TypeConstraint(
            funcType,
            expectedFuncType,
            call.line,
            call.column,
            `Unit function application`
          )
        )

        return ft.returnType // 戻り値型を直接返す
      }

      // 関数型が不明な場合のフォールバック
      const resultType = this.freshTypeVariable(call.line, call.column)
      const expectedFuncType = new AST.FunctionType(
        new AST.PrimitiveType("Unit", call.line, call.column),
        resultType,
        call.line,
        call.column
      )

      this.addConstraint(
        new TypeConstraint(
          funcType,
          expectedFuncType,
          call.line,
          call.column,
          `Unit function application`
        )
      )

      return resultType
    }

    // 関数呼び出しの結果型
    // 多相型を具体化してから制約を生成
    let resultType = this.instantiatePolymorphicType(
      funcType,
      call.line,
      call.column
    )

    // 各引数に対して関数適用の制約を生成
    for (const arg of call.arguments) {
      const argType = this.generateConstraintsForExpression(arg, env)
      const newResultType = this.freshTypeVariable(call.line, call.column)

      // 現在の結果型は引数型から新しい結果型への関数型でなければならない
      const expectedFuncType = new AST.FunctionType(
        argType,
        newResultType,
        call.line,
        call.column
      )

      this.addConstraint(
        new TypeConstraint(
          resultType,
          expectedFuncType,
          call.line,
          call.column,
          `Function application with ${this.typeToString(argType)}`
        )
      )

      resultType = newResultType
    }

    return resultType
  }

  private generateConstraintsForBuiltinFunctionCall(
    call: AST.BuiltinFunctionCall,
    env: Map<string, AST.Type>
  ): AST.Type {
    switch (call.functionName) {
      case "print":
      case "putStrLn":
        // Type: 'a -> Unit (polymorphic)
        if (call.arguments.length === 1) {
          // Just check that the argument has some type, but we accept anything
          this.generateConstraintsForExpression(call.arguments[0], env)
        }
        return new AST.PrimitiveType("Unit", call.line, call.column)

      case "toString":
        // Type: 'a -> String (polymorphic)
        if (call.arguments.length === 1) {
          // Just check that the argument has some type, but we accept anything
          this.generateConstraintsForExpression(call.arguments[0], env)
        }
        return new AST.PrimitiveType("String", call.line, call.column)

      default:
        this.errors.push(
          new TypeInferenceError(
            `Unknown builtin function: ${call.functionName}`,
            call.line,
            call.column
          )
        )
        return this.freshTypeVariable(call.line, call.column)
    }
  }

  private generateConstraintsForFunctionApplication(
    app: AST.FunctionApplication,
    env: Map<string, AST.Type>
  ): AST.Type {
    const funcType = this.generateConstraintsForExpression(app.function, env)
    const argType = this.generateConstraintsForExpression(app.argument, env)
    const resultType = this.freshTypeVariable(app.line, app.column)

    // 関数型は引数型から結果型への関数でなければならない
    const expectedFuncType = new AST.FunctionType(
      argType,
      resultType,
      app.line,
      app.column
    )

    this.addConstraint(
      new TypeConstraint(
        funcType,
        expectedFuncType,
        app.line,
        app.column,
        `Function application`
      )
    )

    return resultType
  }

  private generateConstraintsForPipeline(
    pipe: AST.Pipeline,
    env: Map<string, AST.Type>
  ): AST.Type {
    const leftType = this.generateConstraintsForExpression(pipe.left, env)
    const rightType = this.generateConstraintsForExpression(pipe.right, env)
    const resultType = this.freshTypeVariable(pipe.line, pipe.column)

    // 右側は左側の型から結果型への関数でなければならない
    const expectedFuncType = new AST.FunctionType(
      leftType,
      resultType,
      pipe.line,
      pipe.column
    )

    this.addConstraint(
      new TypeConstraint(
        rightType,
        expectedFuncType,
        pipe.line,
        pipe.column,
        `Pipeline operator`
      )
    )

    return resultType
  }

  private generateConstraintsForConditional(
    cond: AST.ConditionalExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    const condType = this.generateConstraintsForExpression(cond.condition, env)
    const thenType = this.generateConstraintsForExpression(
      cond.thenExpression,
      env
    )
    const elseType = this.generateConstraintsForExpression(
      cond.elseExpression,
      env
    )

    // 条件はBool型でなければならない
    this.addConstraint(
      new TypeConstraint(
        condType,
        new AST.PrimitiveType(
          "Bool",
          cond.condition.line,
          cond.condition.column
        ),
        cond.condition.line,
        cond.condition.column,
        `Conditional expression condition`
      )
    )

    // thenとelseの分岐は同じ型でなければならない
    this.addConstraint(
      new TypeConstraint(
        thenType,
        elseType,
        cond.line,
        cond.column,
        `Conditional expression branches`
      )
    )

    return thenType
  }

  private generateConstraintsForTernary(
    ternary: AST.TernaryExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    const condType = this.generateConstraintsForExpression(
      ternary.condition,
      env
    )
    const trueType = this.generateConstraintsForExpression(
      ternary.trueExpression,
      env
    )
    const falseType = this.generateConstraintsForExpression(
      ternary.falseExpression,
      env
    )

    // 条件はBool型でなければならない
    this.addConstraint(
      new TypeConstraint(
        condType,
        new AST.PrimitiveType(
          "Bool",
          ternary.condition.line,
          ternary.condition.column
        ),
        ternary.condition.line,
        ternary.condition.column,
        `Ternary expression condition`
      )
    )

    // 真と偽の分岐は同じ型でなければならない
    this.addConstraint(
      new TypeConstraint(
        trueType,
        falseType,
        ternary.line,
        ternary.column,
        `Ternary expression branches`
      )
    )

    return trueType
  }

  private generateConstraintsForBlockExpression(
    block: AST.BlockExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // Create a new environment for the block scope
    const blockEnv = new Map(env)

    // Process all statements in the block
    for (const statement of block.statements) {
      this.generateConstraintsForStatement(statement, blockEnv)
    }

    // The block's type is determined by its return expression
    if (block.returnExpression) {
      return this.generateConstraintsForExpression(
        block.returnExpression,
        blockEnv
      )
    }

    // If no explicit return expression, check if the last statement is an expression
    if (block.statements.length > 0) {
      const lastStatement = block.statements[block.statements.length - 1]
      if (lastStatement.kind === "ExpressionStatement") {
        const exprStmt = lastStatement as AST.ExpressionStatement
        return this.generateConstraintsForExpression(
          exprStmt.expression,
          blockEnv
        )
      }
    }

    // If no return expression and last statement is not an expression, return Unit
    return new AST.PrimitiveType("Unit", block.line, block.column)
  }

  private generateConstraintsForFunctorMap(
    functorMap: AST.FunctorMap,
    env: Map<string, AST.Type>
  ): AST.Type {
    // <$> operator: f <$> m
    // Type signature: (a -> b) -> f a -> f b
    const funcType = this.generateConstraintsForExpression(functorMap.left, env)
    const containerType = this.generateConstraintsForExpression(
      functorMap.right,
      env
    )

    const inputType = this.freshTypeVariable(functorMap.line, functorMap.column)
    const outputType = this.freshTypeVariable(
      functorMap.line,
      functorMap.column
    )

    // Function must be of type (a -> b)
    const expectedFuncType = new AST.FunctionType(
      inputType,
      outputType,
      functorMap.line,
      functorMap.column
    )

    this.addConstraint(
      new TypeConstraint(
        funcType,
        expectedFuncType,
        functorMap.line,
        functorMap.column,
        `FunctorMap function type`
      )
    )

    // Container must be of type f a (for the same 'a' as function input)
    if (containerType.kind === "GenericType") {
      const gt = containerType as AST.GenericType

      if (gt.name === "Maybe" && gt.typeArguments.length === 1) {
        // Maybe<a> case
        this.addConstraint(
          new TypeConstraint(
            gt.typeArguments[0],
            inputType,
            functorMap.line,
            functorMap.column,
            `FunctorMap Maybe container input type`
          )
        )

        return new AST.GenericType(
          "Maybe",
          [outputType],
          functorMap.line,
          functorMap.column
        )
      } else if (gt.name === "Either" && gt.typeArguments.length === 2) {
        // Either<e, a> case - function operates on the 'a' type (right side)
        const errorType = gt.typeArguments[0] // Left type stays the same
        this.addConstraint(
          new TypeConstraint(
            gt.typeArguments[1], // Right type
            inputType,
            functorMap.line,
            functorMap.column,
            `FunctorMap Either container input type`
          )
        )

        return new AST.GenericType(
          "Either",
          [errorType, outputType], // Keep error type, map value type
          functorMap.line,
          functorMap.column
        )
      } else if (gt.typeArguments.length > 0) {
        // Generic functor case
        this.addConstraint(
          new TypeConstraint(
            gt.typeArguments[gt.typeArguments.length - 1], // Last type argument
            inputType,
            functorMap.line,
            functorMap.column,
            `FunctorMap container input type`
          )
        )

        // Replace last type argument with output type
        const newArgs = [...gt.typeArguments]
        newArgs[newArgs.length - 1] = outputType

        return new AST.GenericType(
          gt.name,
          newArgs,
          functorMap.line,
          functorMap.column
        )
      }
    }

    // Fallback: assume container is a generic type and return a generic result
    return new AST.GenericType(
      "Functor",
      [outputType],
      functorMap.line,
      functorMap.column
    )
  }

  private generateConstraintsForApplicativeApply(
    applicativeApply: AST.ApplicativeApply,
    env: Map<string, AST.Type>
  ): AST.Type {
    // <*> operator: f (a -> b) <*> f a
    // Type signature: f (a -> b) -> f a -> f b
    const funcContainerType = this.generateConstraintsForExpression(
      applicativeApply.left,
      env
    )
    const valueContainerType = this.generateConstraintsForExpression(
      applicativeApply.right,
      env
    )

    const inputType = this.freshTypeVariable(
      applicativeApply.line,
      applicativeApply.column
    )
    const outputType = this.freshTypeVariable(
      applicativeApply.line,
      applicativeApply.column
    )

    // Function type inside the container
    const funcType = new AST.FunctionType(
      inputType,
      outputType,
      applicativeApply.line,
      applicativeApply.column
    )

    if (
      funcContainerType.kind === "GenericType" &&
      valueContainerType.kind === "GenericType"
    ) {
      const funcGt = funcContainerType as AST.GenericType
      const valueGt = valueContainerType as AST.GenericType

      // Ensure both containers are of the same type
      if (funcGt.name === valueGt.name) {
        if (
          funcGt.name === "Maybe" &&
          funcGt.typeArguments.length === 1 &&
          valueGt.typeArguments.length === 1
        ) {
          // Maybe case: Maybe<(a -> b)> <*> Maybe<a> -> Maybe<b>
          this.addConstraint(
            new TypeConstraint(
              funcGt.typeArguments[0],
              funcType,
              applicativeApply.line,
              applicativeApply.column,
              `ApplicativeApply Maybe function container type`
            )
          )

          this.addConstraint(
            new TypeConstraint(
              valueGt.typeArguments[0],
              inputType,
              applicativeApply.line,
              applicativeApply.column,
              `ApplicativeApply Maybe value container type`
            )
          )

          return new AST.GenericType(
            "Maybe",
            [outputType],
            applicativeApply.line,
            applicativeApply.column
          )
        } else if (
          funcGt.name === "Either" &&
          funcGt.typeArguments.length === 2 &&
          valueGt.typeArguments.length === 2
        ) {
          // Either case: Either<e, (a -> b)> <*> Either<e, a> -> Either<e, b>
          const errorType1 = funcGt.typeArguments[0]
          const errorType2 = valueGt.typeArguments[0]

          // Error types must match
          this.addConstraint(
            new TypeConstraint(
              errorType1,
              errorType2,
              applicativeApply.line,
              applicativeApply.column,
              `ApplicativeApply Either error type consistency`
            )
          )

          this.addConstraint(
            new TypeConstraint(
              funcGt.typeArguments[1],
              funcType,
              applicativeApply.line,
              applicativeApply.column,
              `ApplicativeApply Either function container type`
            )
          )

          this.addConstraint(
            new TypeConstraint(
              valueGt.typeArguments[1],
              inputType,
              applicativeApply.line,
              applicativeApply.column,
              `ApplicativeApply Either value container type`
            )
          )

          return new AST.GenericType(
            "Either",
            [errorType1, outputType],
            applicativeApply.line,
            applicativeApply.column
          )
        }
      }
    }

    // Fallback
    return new AST.GenericType(
      "Applicative",
      [outputType],
      applicativeApply.line,
      applicativeApply.column
    )
  }

  private generateConstraintsForMonadBind(
    monadBind: AST.MonadBind,
    env: Map<string, AST.Type>
  ): AST.Type {
    // >>= operator: m a >>= (a -> m b)
    // Type signature: m a -> (a -> m b) -> m b
    const monadType = this.generateConstraintsForExpression(monadBind.left, env)
    const funcType = this.generateConstraintsForExpression(monadBind.right, env)

    const inputType = this.freshTypeVariable(monadBind.line, monadBind.column)
    const outputType = this.freshTypeVariable(monadBind.line, monadBind.column)

    // Create a generic monad variable that will be constrained later
    const monadVar = this.freshTypeVariable(monadBind.line, monadBind.column)

    // Constrain the left side to be a monad of inputType
    this.addConstraint(
      new TypeConstraint(
        monadType,
        monadVar,
        monadBind.line,
        monadBind.column,
        `MonadBind left side monad type`
      )
    )

    // The function should take inputType and return a monad of outputType
    const expectedOutputMonad = this.freshTypeVariable(
      monadBind.line,
      monadBind.column
    )
    const expectedFuncType = new AST.FunctionType(
      inputType,
      expectedOutputMonad,
      monadBind.line,
      monadBind.column
    )

    this.addConstraint(
      new TypeConstraint(
        funcType,
        expectedFuncType,
        monadBind.line,
        monadBind.column,
        `MonadBind function type`
      )
    )

    // The result should have the same monad structure as the input but with outputType
    const resultType = this.freshTypeVariable(monadBind.line, monadBind.column)

    // Add constraint that the output monad and result should be the same
    this.addConstraint(
      new TypeConstraint(
        expectedOutputMonad,
        resultType,
        monadBind.line,
        monadBind.column,
        `MonadBind result type`
      )
    )

    // Add specific constraints for known monad types
    if (monadType.kind === "GenericType") {
      const monadGt = monadType as AST.GenericType

      if (monadGt.name === "Maybe" && monadGt.typeArguments.length === 1) {
        // Maybe case: Maybe<a> >>= (a -> Maybe<b>) -> Maybe<b>
        this.addConstraint(
          new TypeConstraint(
            monadGt.typeArguments[0],
            inputType,
            monadBind.line,
            monadBind.column,
            `MonadBind Maybe input type`
          )
        )

        const outputMonadType = new AST.GenericType(
          "Maybe",
          [outputType],
          monadBind.line,
          monadBind.column
        )

        this.addConstraint(
          new TypeConstraint(
            expectedOutputMonad,
            outputMonadType,
            monadBind.line,
            monadBind.column,
            `MonadBind Maybe output type`
          )
        )

        return new AST.GenericType(
          "Maybe",
          [outputType],
          monadBind.line,
          monadBind.column
        )
      } else if (
        monadGt.name === "Either" &&
        monadGt.typeArguments.length === 2
      ) {
        // Either case: Either<e, a> >>= (a -> Either<e, b>) -> Either<e, b>
        const errorType = monadGt.typeArguments[0]
        const valueType = monadGt.typeArguments[1]

        this.addConstraint(
          new TypeConstraint(
            valueType,
            inputType,
            monadBind.line,
            monadBind.column,
            `MonadBind Either input type`
          )
        )

        const outputMonadType = new AST.GenericType(
          "Either",
          [errorType, outputType],
          monadBind.line,
          monadBind.column
        )

        this.addConstraint(
          new TypeConstraint(
            expectedOutputMonad,
            outputMonadType,
            monadBind.line,
            monadBind.column,
            `MonadBind Either output type`
          )
        )

        return new AST.GenericType(
          "Either",
          [errorType, outputType],
          monadBind.line,
          monadBind.column
        )
      } else if (
        monadGt.name === "List" &&
        monadGt.typeArguments.length === 1
      ) {
        // List case: List<a> >>= (a -> List<b>) -> List<b>
        this.addConstraint(
          new TypeConstraint(
            monadGt.typeArguments[0],
            inputType,
            monadBind.line,
            monadBind.column,
            `MonadBind List input type`
          )
        )

        const outputMonadType = new AST.GenericType(
          "List",
          [outputType],
          monadBind.line,
          monadBind.column
        )

        this.addConstraint(
          new TypeConstraint(
            expectedOutputMonad,
            outputMonadType,
            monadBind.line,
            monadBind.column,
            `MonadBind List output type`
          )
        )

        return new AST.GenericType(
          "List",
          [outputType],
          monadBind.line,
          monadBind.column
        )
      }
    }

    // Handle Array type (JavaScript arrays)
    if (
      monadType.kind === "PrimitiveType" &&
      (monadType as AST.PrimitiveType).name === "Array"
    ) {
      // Array case: Array >>= (any -> Array) -> Array
      return new AST.PrimitiveType("Array", monadBind.line, monadBind.column)
    }

    // For unknown types, let constraint resolution figure it out
    return resultType
  }



  private generateConstraintsForFunctionApplicationOperator(
    funcApp: AST.FunctionApplicationOperator,
    env: Map<string, AST.Type>
  ): AST.Type {
    // $ operator: f $ x = f(x)
    // This is the same as function application but with infix syntax
    const funcType = this.generateConstraintsForExpression(funcApp.left, env)
    const argType = this.generateConstraintsForExpression(funcApp.right, env)

    const resultType = this.freshTypeVariable(funcApp.line, funcApp.column)

    // The function should be of type argType -> resultType
    const expectedFuncType = new AST.FunctionType(
      argType,
      resultType,
      funcApp.line,
      funcApp.column
    )

    this.addConstraint(
      new TypeConstraint(
        funcType,
        expectedFuncType,
        funcApp.line,
        funcApp.column,
        `Function application operator $`
      )
    )

    return resultType
  }

  private generateConstraintsForConstructorExpression(
    ctor: AST.ConstructorExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    const constructorName = ctor.constructorName

    switch (constructorName) {
      case "Just":
        if (ctor.arguments && ctor.arguments.length > 0) {
          const argType = this.generateConstraintsForExpression(
            ctor.arguments[0],
            env
          )
          return new AST.GenericType("Maybe", [argType], ctor.line, ctor.column)
        } else if (!ctor.arguments || ctor.arguments.length === 0) {
          // Just without arguments - treat as a curried function
          // Just : 'a -> Maybe<'a>
          const elemType = new PolymorphicTypeVariable(
            "a",
            ctor.line,
            ctor.column
          )
          const maybeType = new AST.GenericType(
            "Maybe",
            [elemType],
            ctor.line,
            ctor.column
          )
          return new AST.FunctionType(
            elemType,
            maybeType,
            ctor.line,
            ctor.column
          )
        }
        // Just with wrong number of arguments - should be error
        this.errors.push(
          new TypeInferenceError(
            "Just constructor requires exactly one argument",
            ctor.line,
            ctor.column
          )
        )
        return new AST.GenericType(
          "Maybe",
          [this.freshTypeVariable(ctor.line, ctor.column)],
          ctor.line,
          ctor.column
        )

      case "Nothing":
        // Nothing doesn't take arguments
        if (ctor.arguments && ctor.arguments.length > 0) {
          this.errors.push(
            new TypeInferenceError(
              "Nothing constructor does not take any arguments",
              ctor.line,
              ctor.column
            )
          )
        }
        // Nothing is polymorphic: Maybe<'a>
        return new AST.GenericType(
          "Maybe",
          [new PolymorphicTypeVariable("a", ctor.line, ctor.column)],
          ctor.line,
          ctor.column
        )

      case "Right":
        if (ctor.arguments && ctor.arguments.length > 0) {
          const argType = this.generateConstraintsForExpression(
            ctor.arguments[0],
            env
          )
          // Right is polymorphic in its left type: Either<'a, argType>
          return new AST.GenericType(
            "Either",
            [new PolymorphicTypeVariable("a", ctor.line, ctor.column), argType],
            ctor.line,
            ctor.column
          )
        } else if (!ctor.arguments || ctor.arguments.length === 0) {
          // Right without arguments - treat as a curried function
          // Right : 'b -> Either<'a, 'b>
          const leftType = new PolymorphicTypeVariable(
            "a",
            ctor.line,
            ctor.column
          )
          const rightType = new PolymorphicTypeVariable(
            "b",
            ctor.line,
            ctor.column
          )
          const eitherType = new AST.GenericType(
            "Either",
            [leftType, rightType],
            ctor.line,
            ctor.column
          )
          return new AST.FunctionType(
            rightType,
            eitherType,
            ctor.line,
            ctor.column
          )
        }
        this.errors.push(
          new TypeInferenceError(
            "Right constructor requires exactly one argument",
            ctor.line,
            ctor.column
          )
        )
        return new AST.GenericType(
          "Either",
          [
            new PolymorphicTypeVariable("a", ctor.line, ctor.column),
            new PolymorphicTypeVariable("b", ctor.line, ctor.column),
          ],
          ctor.line,
          ctor.column
        )

      case "Left":
        if (ctor.arguments && ctor.arguments.length > 0) {
          const argType = this.generateConstraintsForExpression(
            ctor.arguments[0],
            env
          )
          // Left is polymorphic in its right type: Either<argType, 'b>
          return new AST.GenericType(
            "Either",
            [argType, new PolymorphicTypeVariable("b", ctor.line, ctor.column)],
            ctor.line,
            ctor.column
          )
        } else if (!ctor.arguments || ctor.arguments.length === 0) {
          // Left without arguments - treat as a curried function
          // Left : 'a -> Either<'a, 'b>
          const leftType = new PolymorphicTypeVariable(
            "a",
            ctor.line,
            ctor.column
          )
          const rightType = new PolymorphicTypeVariable(
            "b",
            ctor.line,
            ctor.column
          )
          const eitherType = new AST.GenericType(
            "Either",
            [leftType, rightType],
            ctor.line,
            ctor.column
          )
          return new AST.FunctionType(
            leftType,
            eitherType,
            ctor.line,
            ctor.column
          )
        }
        this.errors.push(
          new TypeInferenceError(
            "Left constructor requires exactly one argument",
            ctor.line,
            ctor.column
          )
        )
        return new AST.GenericType(
          "Either",
          [
            new PolymorphicTypeVariable("a", ctor.line, ctor.column),
            new PolymorphicTypeVariable("b", ctor.line, ctor.column),
          ],
          ctor.line,
          ctor.column
        )

      case "Empty":
        // Empty is polymorphic: List<'a>
        return new AST.GenericType(
          "List",
          [new PolymorphicTypeVariable("a", ctor.line, ctor.column)],
          ctor.line,
          ctor.column
        )

      case "Cons":
        if (ctor.arguments && ctor.arguments.length === 2) {
          const headType = this.generateConstraintsForExpression(
            ctor.arguments[0],
            env
          )
          const tailType = this.generateConstraintsForExpression(
            ctor.arguments[1],
            env
          )

          // Cons head tail should have type List<headType>
          const expectedTailType = new AST.GenericType(
            "List",
            [headType],
            ctor.line,
            ctor.column
          )

          // Add constraint that tail must be List<headType>
          this.addConstraint(
            new TypeConstraint(
              tailType,
              expectedTailType,
              ctor.line,
              ctor.column,
              "Cons tail type"
            )
          )

          return expectedTailType
        } else if (!ctor.arguments || ctor.arguments.length === 0) {
          // Cons without arguments - treat as a curried function
          // Cons : 'a -> List<'a> -> List<'a>
          const elemType = new PolymorphicTypeVariable(
            "a",
            ctor.line,
            ctor.column
          )
          const listType = new AST.GenericType(
            "List",
            [elemType],
            ctor.line,
            ctor.column
          )
          return new AST.FunctionType(
            elemType,
            new AST.FunctionType(listType, listType, ctor.line, ctor.column),
            ctor.line,
            ctor.column
          )
        }
        this.errors.push(
          new TypeInferenceError(
            "Cons constructor requires exactly two arguments (head and tail)",
            ctor.line,
            ctor.column
          )
        )
        return new AST.GenericType(
          "List",
          [new PolymorphicTypeVariable("a", ctor.line, ctor.column)],
          ctor.line,
          ctor.column
        )

      default: {
        // Check if this is an ADT constructor from the environment
        const constructorType = env.get(constructorName)
        if (constructorType) {
          // This is a known ADT constructor
          return this.applyConstructor(constructorType, ctor, env)
        }

        this.errors.push(
          new TypeInferenceError(
            `Unknown constructor: ${constructorName}`,
            ctor.line,
            ctor.column
          )
        )
        return this.freshTypeVariable(ctor.line, ctor.column)
      }
    }
  }

  private applyConstructor(
    constructorType: AST.Type,
    ctor: AST.ConstructorExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // Extract parameter types and result type from constructor function type
    let currentType = constructorType
    const expectedParamTypes: AST.Type[] = []

    // Traverse function type to get parameter types
    while (currentType instanceof AST.FunctionType) {
      expectedParamTypes.push(currentType.paramType)
      currentType = currentType.returnType
    }

    // The final type should be the ADT type
    const resultType = currentType

    // Check argument count
    if (ctor.arguments.length !== expectedParamTypes.length) {
      this.errors.push(
        new TypeInferenceError(
          `Constructor ${ctor.constructorName} expects ${expectedParamTypes.length} arguments, but got ${ctor.arguments.length}`,
          ctor.line,
          ctor.column
        )
      )
      return resultType
    }

    // Type check each argument
    for (let i = 0; i < ctor.arguments.length; i++) {
      const argType = this.generateConstraintsForExpression(
        ctor.arguments[i],
        env
      )

      // Add constraint that argument type matches expected parameter type
      this.addConstraint(
        new TypeConstraint(
          argType,
          expectedParamTypes[i],
          ctor.arguments[i].line,
          ctor.arguments[i].column,
          `Constructor ${ctor.constructorName} argument ${i + 1}`
        )
      )
    }

    return resultType
  }

  private generateConstraintsForLambdaExpression(
    lambda: AST.LambdaExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // Create a new environment for the lambda body
    const lambdaEnv = new Map(env)

    // Create fresh type variables for parameters with placeholder types
    const parameterTypes: AST.Type[] = []

    for (const param of lambda.parameters) {
      let paramType = param.type

      // If parameter type is placeholder "_", create fresh type variable
      if (
        paramType.kind === "PrimitiveType" &&
        (paramType as AST.PrimitiveType).name === "_"
      ) {
        paramType = this.freshTypeVariable(param.line, param.column)
      }

      parameterTypes.push(paramType)
      lambdaEnv.set(param.name, paramType)
    }

    // Infer the type of the lambda body
    const bodyType = this.generateConstraintsForExpression(
      lambda.body,
      lambdaEnv
    )

    // Build the function type from right to left (currying)
    let resultType: AST.Type = bodyType
    for (let i = lambda.parameters.length - 1; i >= 0; i--) {
      resultType = new AST.FunctionType(
        parameterTypes[i],
        resultType,
        lambda.line,
        lambda.column
      )
    }

    return resultType
  }

  // 制約解決（単一化アルゴリズム）
  private solveConstraints(): TypeSubstitution {
    let substitution = new TypeSubstitution()

    for (const constraint of this.constraints) {
      try {
        const constraintSub = this.unify(
          substitution.apply(constraint.type1),
          substitution.apply(constraint.type2)
        )
        substitution = substitution.compose(constraintSub)
      } catch (error) {
        this.errors.push(
          new TypeInferenceError(
            `Cannot unify types: ${error}`,
            constraint.line,
            constraint.column,
            constraint.context
          )
        )
      }
    }

    return substitution
  }

  // 単一化アルゴリズム
  private unify(type1: AST.Type, type2: AST.Type): TypeSubstitution {
    // console.log(`🔍 Unifying: ${this.typeToString(type1)} with ${this.typeToString(type2)}`)
    const substitution = new TypeSubstitution()

    // 型エイリアスを解決
    const resolvedType1 = this.resolveTypeAlias(type1)
    const resolvedType2 = this.resolveTypeAlias(type2)

    // 同じ型の場合
    if (this.typesEqual(resolvedType1, resolvedType2)) {
      return substitution
    }

    // 型変数の場合（解決前の元の型で処理）
    if (type1.kind === "TypeVariable") {
      const tv1 = type1 as TypeVariable
      if (this.occursCheck(tv1.id, resolvedType2)) {
        throw new Error(
          `Infinite type: ${tv1.name} occurs in ${this.typeToString(resolvedType2)}`
        )
      }
      substitution.set(tv1.id, resolvedType2)
      return substitution
    }

    if (type2.kind === "TypeVariable") {
      const tv2 = type2 as TypeVariable
      if (this.occursCheck(tv2.id, resolvedType1)) {
        throw new Error(
          `Infinite type: ${tv2.name} occurs in ${this.typeToString(resolvedType1)}`
        )
      }
      substitution.set(tv2.id, resolvedType1)
      return substitution
    }

    // 多相型変数の場合 - これらは常に多相のまま残す
    if (
      resolvedType1.kind === "PolymorphicTypeVariable" ||
      resolvedType2.kind === "PolymorphicTypeVariable"
    ) {
      // 多相型変数は統一しない（常に多相のまま）
      return substitution
    }

    // プリミティブ型の場合（解決後の型で比較）
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
        `Cannot unify ${this.typeToString(type1)} with ${this.typeToString(type2)}`
      )
    }

    // 関数型の場合
    if (type1.kind === "FunctionType" && type2.kind === "FunctionType") {
      const ft1 = type1 as AST.FunctionType
      const ft2 = type2 as AST.FunctionType

      const paramSub = this.unify(ft1.paramType, ft2.paramType)
      const returnSub = this.unify(
        paramSub.apply(ft1.returnType),
        paramSub.apply(ft2.returnType)
      )

      return paramSub.compose(returnSub)
    }

    // ジェネリック型の場合
    if (type1.kind === "GenericType" && type2.kind === "GenericType") {
      const gt1 = type1 as AST.GenericType
      const gt2 = type2 as AST.GenericType

      // 特殊ケース: ArrayとListの相互変換を許可（内包表記のため）
      const isArrayListCompatible =
        (gt1.name === "Array" && gt2.name === "List") ||
        (gt1.name === "List" && gt2.name === "Array")

      if (
        !isArrayListCompatible &&
        (gt1.name !== gt2.name ||
          gt1.typeArguments.length !== gt2.typeArguments.length)
      ) {
        throw new Error(
          `Cannot unify ${this.typeToString(type1)} with ${this.typeToString(type2)}`
        )
      }

      let result = substitution
      for (let i = 0; i < gt1.typeArguments.length; i++) {
        const argSub = this.unify(
          result.apply(gt1.typeArguments[i]),
          result.apply(gt2.typeArguments[i])
        )
        result = result.compose(argSub)
      }

      return result
    }

    // Tuple型の場合
    if (type1.kind === "TupleType" && type2.kind === "TupleType") {
      const tt1 = type1 as AST.TupleType
      const tt2 = type2 as AST.TupleType

      // 要素数が一致する必要がある
      if (tt1.elementTypes.length !== tt2.elementTypes.length) {
        throw new Error(
          `Cannot unify ${this.typeToString(type1)} with ${this.typeToString(type2)}: different tuple lengths`
        )
      }

      // 各要素を統一
      let result = substitution
      for (let i = 0; i < tt1.elementTypes.length; i++) {
        const elementSub = this.unify(
          result.apply(tt1.elementTypes[i]),
          result.apply(tt2.elementTypes[i])
        )
        result = result.compose(elementSub)
      }

      return result
    }

    // Record型の場合
    if (type1.kind === "RecordType" && type2.kind === "RecordType") {
      const rt1 = type1 as AST.RecordType
      const rt2 = type2 as AST.RecordType

      // 構造的部分型：一方が他方のサブセットの場合は統一可能
      // 長いレコードの方を基準にして、短いレコードがサブセットかチェック
      const [largerRecord, smallerRecord] =
        rt1.fields.length >= rt2.fields.length ? [rt1, rt2] : [rt2, rt1]
      const isSubset = this.isRecordSubset(smallerRecord, largerRecord)

      if (isSubset) {
        // サブセット関係がある場合、共通フィールドを統一
        let result = substitution
        for (const smallerField of smallerRecord.fields) {
          const largerField = largerRecord.fields.find(
            (f) => f.name === smallerField.name
          )
          if (largerField) {
            const fieldSub = this.unify(
              result.apply(smallerField.type),
              result.apply(largerField.type)
            )
            result = result.compose(fieldSub)
          }
        }
        return result
      }

      // サブセット関係がない場合、完全一致が必要
      if (rt1.fields.length !== rt2.fields.length) {
        throw new Error(
          `Cannot unify ${this.typeToString(type1)} with ${this.typeToString(type2)}: incompatible record structures`
        )
      }

      // フィールド名でソートして比較
      const fields1 = [...rt1.fields].sort((a, b) =>
        a.name.localeCompare(b.name)
      )
      const fields2 = [...rt2.fields].sort((a, b) =>
        a.name.localeCompare(b.name)
      )

      let result = substitution
      for (let i = 0; i < fields1.length; i++) {
        if (fields1[i].name !== fields2[i].name) {
          throw new Error(
            `Cannot unify ${this.typeToString(type1)} with ${this.typeToString(type2)}: field names don't match`
          )
        }

        const fieldSub = this.unify(
          result.apply(fields1[i].type),
          result.apply(fields2[i].type)
        )
        result = result.compose(fieldSub)
      }

      return result
    }

    // Struct型の場合
    if (type1.kind === "StructType" && type2.kind === "StructType") {
      const st1 = type1 as AST.StructType
      const st2 = type2 as AST.StructType

      // 同じ名前の構造体型でなければならない
      if (st1.name !== st2.name) {
        throw new Error(`Cannot unify struct types ${st1.name} and ${st2.name}`)
      }

      return substitution
    }

    // Struct型とRecord型の統一（構造的型付け）
    if (
      (type1.kind === "StructType" && type2.kind === "RecordType") ||
      (type1.kind === "RecordType" && type2.kind === "StructType")
    ) {
      const structType =
        type1.kind === "StructType"
          ? (type1 as AST.StructType)
          : (type2 as AST.StructType)
      const recordType =
        type1.kind === "RecordType"
          ? (type1 as AST.RecordType)
          : (type2 as AST.RecordType)

      // レコード型が構造体のフィールドのサブセットかチェック
      const structAsRecord = new AST.RecordType(
        structType.fields,
        structType.line,
        structType.column
      )
      if (this.isRecordSubset(recordType, structAsRecord)) {
        // 共通フィールドの型を統一
        let result = substitution
        for (const recordField of recordType.fields) {
          const structField = structType.fields.find(
            (f) => f.name === recordField.name
          )
          if (structField) {
            const fieldSub = this.unify(
              result.apply(recordField.type),
              result.apply(structField.type)
            )
            result = result.compose(fieldSub)
          }
        }
        return result
      }

      throw new Error(
        `Cannot unify ${this.typeToString(type1)} with ${this.typeToString(type2)}`
      )
    }

    // Record型と他の型の部分的統一（構造的部分型）
    if (type1.kind === "RecordType" || type2.kind === "RecordType") {
      // どちらか一方がRecordTypeの場合、構造的部分型をチェック
      const recordType =
        type1.kind === "RecordType"
          ? (type1 as AST.RecordType)
          : (type2 as AST.RecordType)
      const otherType = type1.kind === "RecordType" ? type2 : type1

      if (otherType.kind === "TypeVariable") {
        // 型変数の場合は通常の統一を行う
        const tv = otherType as TypeVariable
        if (this.occursCheck(tv.id, recordType)) {
          throw new Error(
            `Infinite type: ${tv.name} occurs in ${this.typeToString(recordType)}`
          )
        }
        substitution.set(tv.id, recordType)
        return substitution
      }

      // その他の場合は統一不可能
      throw new Error(
        `Cannot unify ${this.typeToString(type1)} with ${this.typeToString(type2)}`
      )
    }

    throw new Error(
      `Cannot unify ${this.typeToString(type1)} with ${this.typeToString(type2)}`
    )
  }

  // 型エイリアスの解決
  private resolveTypeAlias(type: AST.Type): AST.Type {
    if (type.kind === "PrimitiveType") {
      const pt = type as AST.PrimitiveType
      // 現在の環境で型エイリアスをチェック
      for (const [name, aliasedType] of this.currentEnvironment) {
        if (name === pt.name) {
          // 再帰的にエイリアスを解決
          return this.resolveTypeAlias(aliasedType)
        }
      }
    }
    return type
  }

  // 環境を保持するためのフィールド（既に存在する可能性）
  private currentEnvironment: Map<string, AST.Type> = new Map()

  // 構造的部分型：小さいレコードが大きいレコードのサブセットかどうかチェック
  private isRecordSubset(
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

  // Occurs Check: 型変数が型の中に現れるかチェック
  private occursCheck(varId: number, type: AST.Type): boolean {
    switch (type.kind) {
      case "TypeVariable":
        return (type as TypeVariable).id === varId

      case "FunctionType": {
        const ft = type as AST.FunctionType
        return (
          this.occursCheck(varId, ft.paramType) ||
          this.occursCheck(varId, ft.returnType)
        )
      }

      case "GenericType": {
        const gt = type as AST.GenericType
        return gt.typeArguments.some((arg) => this.occursCheck(varId, arg))
      }

      case "RecordType": {
        const rt = type as AST.RecordType
        return rt.fields.some((field) => this.occursCheck(varId, field.type))
      }

      case "TupleType": {
        const tt = type as AST.TupleType
        return tt.elementTypes.some((elementType) =>
          this.occursCheck(varId, elementType)
        )
      }

      case "StructType": {
        const st = type as AST.StructType
        return st.fields.some((field) => this.occursCheck(varId, field.type))
      }

      default:
        return false
    }
  }

  // メソッド呼び出しの型推論
  private generateConstraintsForMethodCall(
    call: AST.MethodCall,
    env: Map<string, AST.Type>
  ): AST.Type {
    // レシーバーの型を推論
    const receiverType = this.generateConstraintsForExpression(
      call.receiver,
      env
    )

    // 引数の型を推論
    const argTypes: AST.Type[] = []
    for (const arg of call.arguments) {
      argTypes.push(this.generateConstraintsForExpression(arg, env))
    }

    // レシーバーの型名を取得
    let receiverTypeName: string | null = null
    if (receiverType.kind === "StructType") {
      receiverTypeName = (receiverType as AST.StructType).name
    } else if (receiverType.kind === "PrimitiveType") {
      receiverTypeName = (receiverType as AST.PrimitiveType).name
    } else if (receiverType.kind === "TypeVariable") {
      // 型変数の場合、nodeTypeMapから解決を試みる
      for (const [node, type] of this.nodeTypeMap.entries()) {
        if (type === receiverType && type.kind === "StructType") {
          receiverTypeName = (type as AST.StructType).name
          break
        }
      }
    }

    // implブロックからメソッドを検索
    let methodReturnType: AST.Type | null = null
    if (receiverTypeName) {
      const methodKey = `${receiverTypeName}.${call.methodName}`
      const methodInfo = this.methodEnvironment.get(methodKey)

      if (methodInfo && methodInfo.kind === "MethodDeclaration") {
        const method = methodInfo as AST.MethodDeclaration

        // メソッドの戻り値型を取得
        if (method.returnType) {
          methodReturnType = method.returnType

          // 引数の型チェック（selfを除く）
          const methodParams = method.parameters.filter(
            (p) => !p.isImplicitSelf
          )
          if (methodParams.length === argTypes.length) {
            // 各引数の型制約を追加
            for (let i = 0; i < methodParams.length; i++) {
              this.constraints.push(
                new TypeConstraint(
                  argTypes[i],
                  methodParams[i].type,
                  call.line,
                  call.column,
                  `Method argument ${i} type mismatch`
                )
              )
            }
          }

          // レシーバー型制約を追加
          this.constraints.push(
            new TypeConstraint(
              receiverType,
              new AST.StructType(receiverTypeName, [], call.line, call.column),
              call.line,
              call.column,
              `Method receiver type mismatch`
            )
          )
        }
      }
    }

    // メソッドが見つからない場合は新しい型変数を使用
    if (!methodReturnType) {
      methodReturnType = this.freshTypeVariable(call.line, call.column)

      // カリー化された関数型として制約を構築（従来の方法）
      let expectedMethodType: AST.Type = methodReturnType

      for (let i = argTypes.length - 1; i >= 0; i--) {
        expectedMethodType = new AST.FunctionType(
          argTypes[i],
          expectedMethodType,
          call.line,
          call.column
        )
      }

      expectedMethodType = new AST.FunctionType(
        receiverType,
        expectedMethodType,
        call.line,
        call.column
      )

      this.constraints.push(
        new TypeConstraint(
          expectedMethodType,
          expectedMethodType,
          call.line,
          call.column,
          `Method call ${call.methodName} on type ${this.formatType(receiverType)}`
        )
      )
    }

    // MethodCallノードと型を関連付け（LSP hover用）
    this.nodeTypeMap.set(call, methodReturnType)

    return methodReturnType
  }

  // 型の等価性チェック
  private typesEqual(type1: AST.Type, type2: AST.Type): boolean {
    if (type1.kind !== type2.kind) return false

    switch (type1.kind) {
      case "PrimitiveType":
        return (
          (type1 as AST.PrimitiveType).name ===
          (type2 as AST.PrimitiveType).name
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
          this.typesEqual(ft1.paramType, ft2.paramType) &&
          this.typesEqual(ft1.returnType, ft2.returnType)
        )
      }

      case "GenericType": {
        const gt1 = type1 as AST.GenericType
        const gt2 = type2 as AST.GenericType
        return (
          gt1.name === gt2.name &&
          gt1.typeArguments.length === gt2.typeArguments.length &&
          gt1.typeArguments.every((arg, i) =>
            this.typesEqual(arg, gt2.typeArguments[i])
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
            field1.name === field2.name &&
            this.typesEqual(field1.type, field2.type)
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
          this.typesEqual(elementType, tt2.elementTypes[i])
        )
      }

      default:
        return false
    }
  }

  // 型を文字列に変換
  public typeToString(type: AST.Type): string {
    switch (type.kind) {
      case "PrimitiveType":
        return (type as AST.PrimitiveType).name

      case "TypeVariable":
        return (type as TypeVariable).name

      case "PolymorphicTypeVariable":
        return `'${(type as PolymorphicTypeVariable).name}`

      case "FunctionType": {
        const ft = type as AST.FunctionType
        return `(${this.typeToString(ft.paramType)} -> ${this.typeToString(ft.returnType)})`
      }

      case "GenericType": {
        const gt = type as AST.GenericType
        const args = gt.typeArguments
          .map((t) => this.typeToString(t))
          .join(", ")
        return `${gt.name}<${args}>`
      }

      case "RecordType": {
        const rt = type as AST.RecordType
        const fields = rt.fields
          .map((field) => `${field.name}: ${this.typeToString(field.type)}`)
          .join(", ")
        return `{${fields}}`
      }

      case "TupleType": {
        const tupleType = type as AST.TupleType
        const elements = tupleType.elementTypes
          .map((elementType) => this.typeToString(elementType))
          .join(", ")
        return `(${elements})`
      }

      case "StructType": {
        const st = type as AST.StructType
        return st.name
      }

      default:
        return "Unknown"
    }
  }

  private generateConstraintsForTupleExpression(
    tuple: AST.TupleExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // 各要素の型を推論
    const elementTypes: AST.Type[] = []

    for (const element of tuple.elements) {
      const elementType = this.generateConstraintsForExpression(element, env)
      elementTypes.push(elementType)
    }

    // タプル型を作成
    return new AST.TupleType(elementTypes, tuple.line, tuple.column)
  }

  private generateConstraintsForStructDeclaration(
    structDecl: AST.StructDeclaration,
    env: Map<string, AST.Type>
  ): void {
    // 構造体型を作成
    const structType = new AST.StructType(
      structDecl.name,
      structDecl.fields,
      structDecl.line,
      structDecl.column
    )

    // Debug: Log struct registration
    console.log(`🔧 Registering struct ${structDecl.name}`)
    console.log(`🔧 StructType kind: ${structType.kind}`)
    console.log(`🔧 StructType name: ${structType.name}`)
    console.log(`🔧 StructType: ${this.typeToString(structType)}`)

    // 環境に構造体型を登録
    env.set(structDecl.name, structType)

    // nodeTypeMapにも登録
    this.nodeTypeMap.set(structDecl, structType)
  }

  private generateConstraintsForImplBlock(
    implBlock: AST.ImplBlock,
    env: Map<string, AST.Type>
  ): void {
    // impl ブロックの型名が存在するかチェック
    const implType = env.get(implBlock.typeName)
    if (!implType) {
      this.errors.push(
        new TypeInferenceError(
          `Unknown type for impl block: ${implBlock.typeName}`,
          implBlock.line,
          implBlock.column
        )
      )
      return
    }

    // メソッドの制約を生成
    for (const method of implBlock.methods) {
      this.generateConstraintsForMethodDeclaration(method, env, implType)
    }

    // 演算子の制約を生成
    for (const operator of implBlock.operators) {
      this.generateConstraintsForOperatorDeclaration(operator, env, implType)
    }

    // モノイドの制約を生成
    if (implBlock.monoid) {
      this.generateConstraintsForMonoidDeclaration(
        implBlock.monoid,
        env,
        implType
      )
    }
  }

  private generateConstraintsForMethodDeclaration(
    method: AST.MethodDeclaration,
    env: Map<string, AST.Type>,
    implType: AST.Type
  ): void {
    // メソッドをmethodEnvironmentに登録
    if (implType.kind === "StructType") {
      const structType = implType as AST.StructType
      const methodKey = `${structType.name}.${method.name}`
      this.methodEnvironment.set(methodKey, method)
    }

    // メソッドを関数として処理
    const functionType = this.buildFunctionType(
      method.parameters,
      method.returnType
    )

    // 環境にメソッドを登録
    env.set(`${method.name}`, functionType)
    this.nodeTypeMap.set(method, functionType)

    // メソッド本体を処理するために新しい環境を作成（元の環境をコピー）
    const methodEnv = new Map(env)

    // 現在のimpl対象の型も環境に確実に追加
    if (implType.kind === "StructType") {
      const structType = implType as AST.StructType
      methodEnv.set(structType.name, implType)
    }

    // 全ての構造体型を環境に確実に追加
    for (const [key, value] of env.entries()) {
      if (value.kind === "StructType") {
        methodEnv.set(key, value)
      }
    }

    // パラメータを環境に追加（型エイリアスを解決）
    for (const param of method.parameters) {
      let resolvedType: AST.Type

      // 暗黙的selfまたはotherパラメータの場合は、impl対象の型を使用
      if (param.isImplicitSelf || param.isImplicitOther) {
        resolvedType = implType
        // param.typeも更新しておく（後の処理のため）
        param.type = implType
      } else {
        resolvedType = this.resolveTypeAlias(param.type)

        // 構造体型の場合は、環境から実際のStructTypeを取得
        if (resolvedType.kind === "PrimitiveType") {
          const structTypeFromEnv = env.get(
            (resolvedType as AST.PrimitiveType).name
          )
          if (structTypeFromEnv && structTypeFromEnv.kind === "StructType") {
            resolvedType = structTypeFromEnv
          }
        }
      }

      methodEnv.set(param.name, resolvedType)
    }

    // メソッド本体の制約を生成
    const bodyType = this.generateConstraintsForExpression(
      method.body,
      methodEnv
    )

    // 戻り値型との制約を追加
    this.addConstraint(
      new TypeConstraint(
        bodyType,
        method.returnType,
        method.line,
        method.column,
        `Method ${method.name} body type`
      )
    )
  }

  private generateConstraintsForOperatorDeclaration(
    operator: AST.OperatorDeclaration,
    env: Map<string, AST.Type>,
    implType: AST.Type
  ): void {
    // 演算子を関数として処理
    const functionType = this.buildFunctionType(
      operator.parameters,
      operator.returnType
    )

    // 環境に演算子を登録
    env.set(`${operator.operator}`, functionType)
    this.nodeTypeMap.set(operator, functionType)

    // 演算子本体を処理するために新しい環境を作成（元の環境をコピー）
    const operatorEnv = new Map(env)

    // 現在のimpl対象の型も環境に確実に追加
    if (implType.kind === "StructType") {
      const structType = implType as AST.StructType
      operatorEnv.set(structType.name, implType)
    }

    // 全ての構造体型を環境に確実に追加
    for (const [key, value] of env.entries()) {
      if (value.kind === "StructType") {
        operatorEnv.set(key, value)
      }
    }

    // パラメータを環境に追加（型エイリアスを解決）
    for (const param of operator.parameters) {
      let resolvedType: AST.Type

      // 暗黙的selfまたはotherパラメータの場合は、impl対象の型を使用
      if (param.isImplicitSelf || param.isImplicitOther) {
        resolvedType = implType
        // param.typeも更新しておく（後の処理のため）
        param.type = implType
      } else {
        resolvedType = this.resolveTypeAlias(param.type)

        // 構造体型の場合は、環境から実際のStructTypeを取得
        if (resolvedType.kind === "PrimitiveType") {
          const structTypeFromEnv = env.get(
            (resolvedType as AST.PrimitiveType).name
          )
          if (structTypeFromEnv && structTypeFromEnv.kind === "StructType") {
            resolvedType = structTypeFromEnv
          }
        }
      }

      operatorEnv.set(param.name, resolvedType)
      this.nodeTypeMap.set(param, resolvedType)
    }

    // 演算子本体の制約を生成
    const bodyType = this.generateConstraintsForExpression(
      operator.body,
      operatorEnv
    )

    // 戻り値型との制約を追加
    this.addConstraint(
      new TypeConstraint(
        bodyType,
        operator.returnType,
        operator.line,
        operator.column,
        `Operator ${operator.operator} body type`
      )
    )
  }

  private generateConstraintsForMonoidDeclaration(
    monoid: AST.MonoidDeclaration,
    env: Map<string, AST.Type>,
    implType: AST.Type
  ): void {
    // identity値の制約を生成
    const identityType = this.generateConstraintsForExpression(
      monoid.identity,
      env
    )

    // identity値は型と一致する必要がある
    this.addConstraint(
      new TypeConstraint(
        identityType,
        implType,
        monoid.line,
        monoid.column,
        "Monoid identity type"
      )
    )

    // 演算子の制約を生成
    this.generateConstraintsForOperatorDeclaration(
      monoid.operator,
      env,
      implType
    )
  }

  // 型が構造体型または構造体型に解決される型変数かをチェック
  private isStructOrResolvesToStruct(
    type: AST.Type,
    env: Map<string, AST.Type>
  ): boolean {
    // 直接的に構造体型の場合
    if (type.kind === "StructType") {
      return true
    }

    // プリミティブ型の場合、環境から構造体型を検索
    if (type.kind === "PrimitiveType") {
      const resolved = env.get((type as AST.PrimitiveType).name)
      return resolved?.kind === "StructType"
    }

    // 型変数の場合、現在の型置換を確認
    // 注意: 制約生成段階では型変数は未解決のため、構造体型の可能性として扱う
    if (type.kind === "TypeVariable") {
      return true // 保守的に true を返し、構造体の可能性を考慮
    }

    return false
  }

  // パラメータリストから関数型を構築（カリー化）
  private buildFunctionType(
    parameters: AST.Parameter[],
    returnType: AST.Type
  ): AST.Type {
    if (parameters.length === 0) {
      return returnType
    }

    // 右結合でカリー化された関数型を構築
    let result = returnType
    for (let i = parameters.length - 1; i >= 0; i--) {
      result = new AST.FunctionType(
        parameters[i].type,
        result,
        parameters[i].line,
        parameters[i].column
      )
    }

    return result
  }

  private generateConstraintsForStructExpression(
    structExpr: AST.StructExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // 構造体型を環境から取得
    const structType = env.get(structExpr.structName)

    if (!structType) {
      this.errors.push(
        new TypeInferenceError(
          `Unknown struct type: ${structExpr.structName}`,
          structExpr.line,
          structExpr.column
        )
      )
      return this.freshTypeVariable(structExpr.line, structExpr.column)
    }

    if (structType.kind !== "StructType") {
      this.errors.push(
        new TypeInferenceError(
          `${structExpr.structName} is not a struct type`,
          structExpr.line,
          structExpr.column
        )
      )
      return this.freshTypeVariable(structExpr.line, structExpr.column)
    }

    // フィールドの型チェック
    const providedFieldMap = new Map<string, { field: AST.RecordInitField | AST.RecordSpreadField, type: AST.Type }>()

    // まずスプレッドフィールドを処理
    for (const field of structExpr.fields) {
      if (field.kind === "RecordSpreadField") {
        const spreadField = field as AST.RecordSpreadField
        const spreadType = this.generateConstraintsForExpression(spreadField.spreadExpression.expression, env)
        
        // スプレッド元が同じ構造体型であることを確認
        if (spreadType.kind === "StructType") {
          const sourceStruct = spreadType as AST.StructType
          // スプレッド元のフィールドをすべて追加
          for (const sourceField of sourceStruct.fields) {
            providedFieldMap.set(sourceField.name, { 
              field: spreadField, 
              type: sourceField.type 
            })
          }
        } else {
          this.errors.push(
            new TypeInferenceError(
              `Cannot spread non-struct type in struct literal`,
              spreadField.line,
              spreadField.column
            )
          )
        }
      }
    }

    // 次に明示的なフィールドで上書き
    for (const field of structExpr.fields) {
      if (field.kind === "RecordInitField") {
        const initField = field as AST.RecordInitField
        const fieldType = this.generateConstraintsForExpression(initField.value, env)
        providedFieldMap.set(initField.name, { field: initField, type: fieldType })
      } else if (field.kind === "RecordShorthandField") {
        const shorthandField = field as AST.RecordShorthandField
        // 変数名と同じ名前の変数を環境から検索
        const variableType = env.get(shorthandField.name)
        if (!variableType) {
          this.errors.push(
            new TypeInferenceError(
              `Undefined variable '${shorthandField.name}' in shorthand property`,
              shorthandField.line,
              shorthandField.column
            )
          )
          // エラーの場合はTypeVariableをフォールバック
          const fallbackType = this.freshTypeVariable(shorthandField.line, shorthandField.column)
          providedFieldMap.set(shorthandField.name, { field: shorthandField, type: fallbackType })
        } else {
          providedFieldMap.set(shorthandField.name, { field: shorthandField, type: variableType })
        }
      }
    }

    // 必要なフィールドがすべて提供されているかチェック
    for (const field of structType.fields) {
      const providedData = providedFieldMap.get(field.name)

      if (!providedData) {
        this.errors.push(
          new TypeInferenceError(
            `Missing field '${field.name}' in struct ${structExpr.structName}`,
            structExpr.line,
            structExpr.column
          )
        )
        continue
      }

      // フィールドの型と値の型が一致することを制約として追加
      this.constraints.push(
        new TypeConstraint(
          providedData.type,
          field.type,
          providedData.field.line,
          providedData.field.column,
          `Struct field ${field.name}`
        )
      )
    }

    // 余分なフィールドがないかチェック
    for (const [fieldName, _] of providedFieldMap) {
      if (!structType.fields.find((f) => f.name === fieldName)) {
        this.errors.push(
          new TypeInferenceError(
            `Unknown field '${fieldName}' in struct ${structExpr.structName}`,
            structExpr.line,
            structExpr.column
          )
        )
      }
    }

    return structType
  }

  private generateConstraintsForMatchExpression(
    match: AST.MatchExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // マッチ対象の式の型を推論
    const exprType = this.generateConstraintsForExpression(
      match.expression,
      env
    )

    if (match.cases.length === 0) {
      this.errors.push(
        new TypeInferenceError(
          "Match expression must have at least one case",
          match.line,
          match.column
        )
      )
      return this.freshTypeVariable(match.line, match.column)
    }

    // 最初のケースの結果型を基準とする
    const firstCase = match.cases[0]

    // パターンマッチングで新しい変数環境を作成
    const patternEnv = new Map(env)
    this.generateConstraintsForPattern(firstCase.pattern, exprType, patternEnv)

    // 最初のケースの結果型を推論
    const resultType = this.generateConstraintsForExpression(
      firstCase.expression,
      patternEnv
    )

    // 残りのケースの型をチェックし、すべて同じ型になるよう制約を追加
    for (let i = 1; i < match.cases.length; i++) {
      const caseItem = match.cases[i]

      // パターンマッチングで新しい変数環境を作成
      const caseEnv = new Map(env)
      this.generateConstraintsForPattern(caseItem.pattern, exprType, caseEnv)

      // ケースの結果型を推論
      const caseResultType = this.generateConstraintsForExpression(
        caseItem.expression,
        caseEnv
      )

      // 結果型が一致するよう制約を追加
      this.addConstraint(
        new TypeConstraint(
          caseResultType,
          resultType,
          caseItem.expression.line,
          caseItem.expression.column,
          `Match case ${i + 1} result type`
        )
      )
    }

    return resultType
  }

  private generateConstraintsForPattern(
    pattern: AST.Pattern,
    expectedType: AST.Type,
    env: Map<string, AST.Type>
  ): void {
    switch (pattern.kind) {
      case "IdentifierPattern": {
        // 変数パターン: 変数を環境に追加
        const idPattern = pattern as AST.IdentifierPattern
        env.set(idPattern.name, expectedType)
        break
      }

      case "LiteralPattern": {
        // リテラルパターン: 期待する型と一致するかチェック
        const litPattern = pattern as AST.LiteralPattern
        let literalType: AST.Type

        switch (typeof litPattern.value) {
          case "string":
            literalType = new AST.PrimitiveType(
              "String",
              pattern.line,
              pattern.column
            )
            break
          case "number":
            literalType = new AST.PrimitiveType(
              "Int",
              pattern.line,
              pattern.column
            )
            break
          case "boolean":
            literalType = new AST.PrimitiveType(
              "Bool",
              pattern.line,
              pattern.column
            )
            break
          default:
            literalType = this.freshTypeVariable(pattern.line, pattern.column)
        }

        this.addConstraint(
          new TypeConstraint(
            expectedType,
            literalType,
            pattern.line,
            pattern.column,
            `Literal pattern type`
          )
        )
        break
      }

      case "ConstructorPattern": {
        // コンストラクタパターン: 代数的データ型のコンストラクタ
        const ctorPattern = pattern as AST.ConstructorPattern

        // 環境からコンストラクタの型を取得
        const constructorType = env.get(ctorPattern.constructorName)
        if (!constructorType) {
          this.errors.push(
            new TypeInferenceError(
              `Unknown constructor: ${ctorPattern.constructorName}`,
              pattern.line,
              pattern.column
            )
          )
          return
        }

        // コンストラクタがADT型を返すことを確認
        let adtType = expectedType
        let currentType = constructorType
        const paramTypes: AST.Type[] = []

        // 関数型を辿ってパラメータ型を抽出
        while (currentType instanceof AST.FunctionType) {
          paramTypes.push(currentType.paramType)
          currentType = currentType.returnType
        }

        // 最終的な戻り値型（ADT型）
        adtType = currentType

        // 期待する型とADT型が一致することを確認
        this.addConstraint(
          new TypeConstraint(
            expectedType,
            adtType,
            pattern.line,
            pattern.column,
            `Constructor pattern type`
          )
        )

        // パターンの引数と型パラメータの数が一致することを確認
        if (ctorPattern.patterns.length !== paramTypes.length) {
          this.errors.push(
            new TypeInferenceError(
              `Constructor ${ctorPattern.constructorName} expects ${paramTypes.length} arguments, but got ${ctorPattern.patterns.length}`,
              pattern.line,
              pattern.column
            )
          )
          return
        }

        // ネストしたパターンに対応する型で再帰的に処理
        for (let i = 0; i < ctorPattern.patterns.length; i++) {
          this.generateConstraintsForPattern(
            ctorPattern.patterns[i],
            paramTypes[i],
            env
          )
        }
        break
      }

      case "WildcardPattern":
        // ワイルドカードパターン: 何にでもマッチし、変数をバインドしない
        break

      case "TuplePattern": {
        // タプルパターン: (x, y, z) = tuple
        const tuplePattern = pattern as AST.TuplePattern

        // タプル型の要素数と一致するかチェック
        const expectedElementTypes: AST.Type[] = []
        for (let i = 0; i < tuplePattern.patterns.length; i++) {
          expectedElementTypes.push(
            this.freshTypeVariable(pattern.line, pattern.column)
          )
        }

        const tupleType = new AST.TupleType(
          expectedElementTypes,
          pattern.line,
          pattern.column
        )
        this.addConstraint(
          new TypeConstraint(
            expectedType,
            tupleType,
            pattern.line,
            pattern.column,
            "Tuple pattern structure"
          )
        )

        // 各パターン要素に対応する型で再帰的に処理
        for (let i = 0; i < tuplePattern.patterns.length; i++) {
          this.generateConstraintsForPattern(
            tuplePattern.patterns[i],
            expectedElementTypes[i],
            env
          )
        }
        break
      }

      default:
        this.errors.push(
          new TypeInferenceError(
            `Unsupported pattern type: ${pattern.kind}`,
            pattern.line,
            pattern.column
          )
        )
    }
  }

  private generateConstraintsForRecordExpression(
    record: AST.RecordExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // Mapを使って重複フィールドを避ける（後から来たフィールドで上書き）
    const fieldMap = new Map<string, AST.RecordField>()

    for (const field of record.fields) {
      if (field.kind === "RecordInitField") {
        const initField = field as AST.RecordInitField
        const fieldType = this.generateConstraintsForExpression(
          initField.value,
          env
        )
        // 同名フィールドは上書き
        fieldMap.set(initField.name, new AST.RecordField(
          initField.name,
          fieldType,
          initField.line,
          initField.column
        ))
      } else if (field.kind === "RecordShorthandField") {
        const shorthandField = field as AST.RecordShorthandField
        // 変数名と同じ名前の変数を環境から検索
        const variableType = env.get(shorthandField.name)
        if (!variableType) {
          this.errors.push(
            new TypeInferenceError(
              `Undefined variable '${shorthandField.name}' in shorthand property`,
              shorthandField.line,
              shorthandField.column
            )
          )
          // エラーの場合はTypeVariableをフォールバック
          const fallbackType = this.freshTypeVariable(shorthandField.line, shorthandField.column)
          fieldMap.set(shorthandField.name, new AST.RecordField(
            shorthandField.name,
            fallbackType,
            shorthandField.line,
            shorthandField.column
          ))
        } else {
          fieldMap.set(shorthandField.name, new AST.RecordField(
            shorthandField.name,
            variableType,
            shorthandField.line,
            shorthandField.column
          ))
        }
      } else if (field.kind === "RecordSpreadField") {
        const spreadField = field as AST.RecordSpreadField
        const spreadType = this.generateConstraintsForExpression(
          spreadField.spreadExpression,
          env
        )
        
        // スプレッド元がレコード型であることを確認
        if (spreadType.kind === "RecordType") {
          const recordType = spreadType as AST.RecordType
          // スプレッド元のフィールドをマージ（後から来たフィールドで上書き）
          for (const sourceField of recordType.fields) {
            fieldMap.set(sourceField.name, sourceField)
          }
        } else {
          this.errors.push(
            new TypeInferenceError(
              `Cannot spread non-record type in record literal`,
              spreadField.line,
              spreadField.column
            )
          )
        }
      }
    }

    // Mapから配列に変換
    const fields = Array.from(fieldMap.values())
    return new AST.RecordType(fields, record.line, record.column)
  }

  private generateConstraintsForRecordAccess(
    access: AST.RecordAccess,
    env: Map<string, AST.Type>
  ): AST.Type {
    const recordType = this.generateConstraintsForExpression(access.record, env)

    // まず、recordTypeがStructTypeかどうかを直接チェック
    if (recordType.kind === "StructType") {
      const structType = recordType as AST.StructType
      const field = structType.fields.find((f) => f.name === access.fieldName)
      if (field) {
        return field.type
      } else {
        this.errors.push(
          new TypeInferenceError(
            `Field '${access.fieldName}' does not exist on struct '${structType.name}'`,
            access.line,
            access.column,
            `Field access .${access.fieldName}`
          )
        )
        return this.freshTypeVariable(access.line, access.column)
      }
    }

    // 型変数やその他の場合は、従来の制約ベースのアプローチを使用
    const fieldType = this.freshTypeVariable(access.line, access.column)

    // 構造的制約を常に作成 - unificationプロセスで解決
    // これにより、StructType と RecordType の両方が適切に処理される
    const expectedRecordType = new AST.RecordType(
      [
        new AST.RecordField(
          access.fieldName,
          fieldType,
          access.line,
          access.column
        ),
      ],
      access.line,
      access.column
    )

    // レコードまたは構造体が指定フィールドと互換性があることを制約
    this.addConstraint(
      new TypeConstraint(
        recordType,
        expectedRecordType,
        access.line,
        access.column,
        `Field access .${access.fieldName}`
      )
    )

    return fieldType
  }

  private generateConstraintsForArrayLiteral(
    arrayLiteral: AST.ArrayLiteral,
    env: Map<string, AST.Type>
  ): AST.Type {
    if (arrayLiteral.elements.length === 0) {
      // 空配列の場合、要素型は新しい型変数
      const elementType = this.freshTypeVariable(
        arrayLiteral.line,
        arrayLiteral.column
      )
      return new AST.GenericType(
        "Array",
        [elementType],
        arrayLiteral.line,
        arrayLiteral.column
      )
    }

    // 最初の要素の型を推論
    const firstElementType = this.generateConstraintsForExpression(
      arrayLiteral.elements[0],
      env
    )

    // すべての要素が同じ型であることを制約として追加
    for (let i = 1; i < arrayLiteral.elements.length; i++) {
      const elementType = this.generateConstraintsForExpression(
        arrayLiteral.elements[i],
        env
      )
      this.addConstraint(
        new TypeConstraint(
          firstElementType,
          elementType,
          arrayLiteral.elements[i].line,
          arrayLiteral.elements[i].column,
          `Array element type consistency`
        )
      )
    }

    return new AST.GenericType(
      "Array",
      [firstElementType],
      arrayLiteral.line,
      arrayLiteral.column
    )
  }

  private generateConstraintsForArrayAccess(
    arrayAccess: AST.ArrayAccess,
    env: Map<string, AST.Type>
  ): AST.Type {
    const arrayType = this.generateConstraintsForExpression(
      arrayAccess.array,
      env
    )
    const indexType = this.generateConstraintsForExpression(
      arrayAccess.index,
      env
    )

    // インデックスはInt型でなければならない
    this.addConstraint(
      new TypeConstraint(
        indexType,
        new AST.PrimitiveType(
          "Int",
          arrayAccess.index.line,
          arrayAccess.index.column
        ),
        arrayAccess.index.line,
        arrayAccess.index.column,
        "Array index must be Int"
      )
    )

    // 配列の要素型を取得
    const elementType = this.freshTypeVariable(
      arrayAccess.line,
      arrayAccess.column
    )
    const expectedArrayType = new AST.GenericType(
      "Array",
      [elementType],
      arrayAccess.line,
      arrayAccess.column
    )

    // 配列がArray<T>型であることを制約として追加
    this.addConstraint(
      new TypeConstraint(
        arrayType,
        expectedArrayType,
        arrayAccess.line,
        arrayAccess.column,
        "Array access type"
      )
    )

    return elementType
  }

  private generateConstraintsForListSugar(
    listSugar: AST.ListSugar,
    env: Map<string, AST.Type>
  ): AST.Type {
    if (listSugar.elements.length === 0) {
      // 空リストの場合、要素型は新しい型変数
      const elementType = this.freshTypeVariable(
        listSugar.line,
        listSugar.column
      )
      return new AST.GenericType(
        "List",
        [elementType],
        listSugar.line,
        listSugar.column
      )
    }

    // 最初の要素の型を推論
    const firstElementType = this.generateConstraintsForExpression(
      listSugar.elements[0],
      env
    )

    // すべての要素が同じ型であることを制約として追加
    for (let i = 1; i < listSugar.elements.length; i++) {
      const elementType = this.generateConstraintsForExpression(
        listSugar.elements[i],
        env
      )
      this.addConstraint(
        new TypeConstraint(
          firstElementType,
          elementType,
          listSugar.elements[i].line,
          listSugar.elements[i].column,
          `List element type consistency`
        )
      )
    }

    return new AST.GenericType(
      "List",
      [firstElementType],
      listSugar.line,
      listSugar.column
    )
  }

  private generateConstraintsForConsExpression(
    consExpr: AST.ConsExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // head : tail の型推論
    const headType = this.generateConstraintsForExpression(consExpr.left, env)
    const tailType = this.generateConstraintsForExpression(consExpr.right, env)

    // tailはList<T>型でなければならない
    const expectedTailType = new AST.GenericType(
      "List",
      [headType],
      consExpr.right.line,
      consExpr.right.column
    )

    this.addConstraint(
      new TypeConstraint(
        tailType,
        expectedTailType,
        consExpr.right.line,
        consExpr.right.column,
        "Cons tail must be List type"
      )
    )

    // 結果の型もList<T>
    return new AST.GenericType(
      "List",
      [headType],
      consExpr.line,
      consExpr.column
    )
  }

  private generateConstraintsForRangeLiteral(
    range: AST.RangeLiteral,
    env: Map<string, AST.Type>
  ): AST.Type {
    // 範囲リテラルの開始と終了値の型を推論
    const startType = this.generateConstraintsForExpression(range.start, env)
    const endType = this.generateConstraintsForExpression(range.end, env)

    // 開始と終了は同じ型でなければならない
    this.addConstraint(
      new TypeConstraint(
        startType,
        endType,
        range.line,
        range.column,
        "Range start and end must have same type"
      )
    )

    // 範囲は数値型（Int）のリストを返す
    const intType = new AST.PrimitiveType("Int", range.line, range.column)

    this.addConstraint(
      new TypeConstraint(
        startType,
        intType,
        range.start.line,
        range.start.column,
        "Range values must be integers"
      )
    )

    // 範囲リテラルはArray<Int>を返す
    return new AST.GenericType("Array", [intType], range.line, range.column)
  }

  private generateConstraintsForListComprehension(
    comp: AST.ListComprehension,
    env: Map<string, AST.Type>
  ): AST.Type {
    // 内包表記用の環境を作成
    const compEnv = new Map(env)

    // ジェネレータを処理してスコープに変数を追加
    for (const generator of comp.generators) {
      // ジェネレータのiterableの型を推論
      const iterableType = this.generateConstraintsForExpression(
        generator.iterable,
        compEnv
      )

      // iterableはリスト型またはArray型でなければならない
      const elementType = this.freshTypeVariable(
        generator.line,
        generator.column
      )
      const arrayType = new AST.GenericType(
        "Array",
        [elementType],
        generator.line,
        generator.column
      )

      // 配列内包表記では範囲リテラル（Array型）を直接受け入れる
      this.addConstraint(
        new TypeConstraint(
          iterableType,
          arrayType,
          generator.line,
          generator.column,
          `Generator iterable must be Array type for array comprehensions`
        )
      )

      // ジェネレータ変数をスコープに追加
      compEnv.set(generator.variable, elementType)
    }

    // フィルタ条件の型チェック
    for (const filter of comp.filters) {
      const filterType = this.generateConstraintsForExpression(filter, compEnv)
      const boolType = new AST.PrimitiveType("Bool", filter.line, filter.column)

      this.addConstraint(
        new TypeConstraint(
          filterType,
          boolType,
          filter.line,
          filter.column,
          "Array comprehension filter must be Bool"
        )
      )
    }

    // 内包表記の式の型を推論
    const expressionType = this.generateConstraintsForExpression(
      comp.expression,
      compEnv
    )

    // 結果はArray<expressionType>（配列内包表記なのでArrayを返す）
    return new AST.GenericType(
      "Array",
      [expressionType],
      comp.line,
      comp.column
    )
  }

  private generateConstraintsForListComprehensionSugar(
    comp: AST.ListComprehensionSugar,
    env: Map<string, AST.Type>
  ): AST.Type {
    // ListComprehensionSugarは通常のListComprehensionと同じ型推論を行う
    // 内包表記用の環境を作成
    const compEnv = new Map(env)

    // ジェネレータを処理してスコープに変数を追加
    for (const generator of comp.generators) {
      // ジェネレータのiterableの型を推論
      const iterableType = this.generateConstraintsForExpression(
        generator.iterable,
        compEnv
      )

      // iterableはList型またはArray型を受け入れる（バッククォート記法でも配列を受け入れる）
      const elementType = this.freshTypeVariable(
        generator.line,
        generator.column
      )
      const expectedListType = new AST.GenericType(
        "List",
        [elementType],
        generator.line,
        generator.column
      )

      // iterableがListまたはArrayであることを制約として追加
      // 制約解決システムがArray<->List変換を処理する
      this.addConstraint(
        new TypeConstraint(
          iterableType,
          expectedListType,
          generator.line,
          generator.column,
          `Generator iterable must be List type (Array conversion allowed)`
        )
      )

      // ジェネレータ変数をスコープに追加
      compEnv.set(generator.variable, elementType)
    }

    // フィルタ条件の型チェック
    for (const filter of comp.filters) {
      const filterType = this.generateConstraintsForExpression(filter, compEnv)
      const boolType = new AST.PrimitiveType("Bool", filter.line, filter.column)

      this.addConstraint(
        new TypeConstraint(
          filterType,
          boolType,
          filter.line,
          filter.column,
          "List comprehension filter must be Bool"
        )
      )
    }

    // 内包表記の式の型を推論
    const expressionType = this.generateConstraintsForExpression(
      comp.expression,
      compEnv
    )

    // 結果はList<expressionType>
    return new AST.GenericType("List", [expressionType], comp.line, comp.column)
  }

  private generateConstraintsForSpreadExpression(
    spread: AST.SpreadExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // スプレッド式自体は中身の式の型と同じ
    return this.generateConstraintsForExpression(spread.expression, env)
  }

  private generateConstraintsForRecordDestructuring(
    recordDestr: AST.RecordDestructuring,
    env: Map<string, AST.Type>
  ): void {
    // 初期化式の型を推論
    const initType = this.generateConstraintsForExpression(
      recordDestr.initializer,
      env
    )

    // パターン内の各フィールドを環境に追加し、適切な型制約を設定
    for (const field of recordDestr.pattern.fields) {
      const variableName = field.alias || field.fieldName
      const fieldType = this.freshTypeVariable(field.line, field.column)
      
      // フィールド変数を環境に追加
      env.set(variableName, fieldType)
      this.nodeTypeMap.set(field, fieldType)

      // 初期化式がレコード型で、該当フィールドを持つことを制約として追加
      // レコードフィールドアクセスと同等の制約を作成
      const recordFieldType = this.freshTypeVariable(field.line, field.column)
      const expectedRecordType = new AST.RecordType(
        [new AST.RecordField(field.fieldName, recordFieldType, field.line, field.column)],
        recordDestr.line,
        recordDestr.column
      )

      // 初期化式のレコード型に該当フィールドが存在することを制約として追加
      this.addConstraint(
        new TypeConstraint(
          initType,
          expectedRecordType,
          field.line,
          field.column,
          `Record destructuring field ${field.fieldName}`
        )
      )

      // フィールド変数の型とレコードフィールドの型が一致することを制約として追加
      this.addConstraint(
        new TypeConstraint(
          fieldType,
          recordFieldType,
          field.line,
          field.column,
          `Record destructuring field type ${field.fieldName}`
        )
      )
    }
  }

  private generateConstraintsForStructDestructuring(
    structDestr: AST.StructDestructuring,
    env: Map<string, AST.Type>
  ): void {
    // 初期化式の型を推論
    const initType = this.generateConstraintsForExpression(
      structDestr.initializer,
      env
    )

    // 初期化式が指定された構造体型であることを制約として追加
    const expectedStructType = env.get(structDestr.pattern.structName)
    if (expectedStructType && expectedStructType.kind === "StructType") {
      this.addConstraint(
        new TypeConstraint(
          initType,
          expectedStructType,
          structDestr.line,
          structDestr.column,
          `Struct destructuring of ${structDestr.pattern.structName}`
        )
      )

      // パターン内の各フィールドを環境に追加し、適切な型制約を設定
      const structType = expectedStructType as AST.StructType
      for (const field of structDestr.pattern.fields) {
        const variableName = field.alias || field.fieldName
        
        // 構造体定義から該当フィールドの型を取得
        const structField = structType.fields.find(f => f.name === field.fieldName)
        if (structField) {
          // フィールドの実際の型を使用
          env.set(variableName, structField.type)
          this.nodeTypeMap.set(field, structField.type)
          
          // 分割代入された変数を追跡するために、仮想的な変数宣言ノードを作成してnodeTypeMapに追加
          const virtualVarDecl = {
            kind: "VariableDeclaration",
            name: variableName,
            type: structField.type,
            line: field.line,
            column: field.column,
            isDestructured: true
          }
          this.nodeTypeMap.set(virtualVarDecl, structField.type)
        } else {
          // フィールドが見つからない場合はエラー
          this.errors.push(
            new TypeInferenceError(
              `Field '${field.fieldName}' does not exist in struct ${structDestr.pattern.structName}`,
              field.line,
              field.column
            )
          )
          const fieldType = this.freshTypeVariable(field.line, field.column)
          env.set(variableName, fieldType)
        }
      }
    } else {
      // 構造体型が見つからない場合はfreshTypeVariableで処理
      for (const field of structDestr.pattern.fields) {
        const variableName = field.alias || field.fieldName
        const fieldType = this.freshTypeVariable(field.line, field.column)
        env.set(variableName, fieldType)
      }
    }
  }
}

// Convenience function for type inference
export function infer(statements: AST.Statement[]): InferenceResult {
  const inference = new TypeInferenceSystem()
  const program = new AST.Program(statements)
  const result = inference.infer(program)

  // Transform the result to match InferenceResult interface
  const inferredTypes = new Map<string, AST.Type>()
  const typeEnvironment = new Map<string, AST.Type>()

  // Extract inferred types for variables, functions, and structs
  for (const stmt of statements) {
    if (stmt instanceof AST.VariableDeclaration) {
      const type = result.nodeTypeMap.get(stmt)
      if (type) {
        const resolvedType = result.substitution.apply(type)
        inferredTypes.set(stmt.name, resolvedType)
        typeEnvironment.set(stmt.name, resolvedType)
      }
    } else if (stmt instanceof AST.FunctionDeclaration) {
      const type = result.nodeTypeMap.get(stmt)
      if (type) {
        const resolvedType = result.substitution.apply(type)
        inferredTypes.set(stmt.name, resolvedType)
        typeEnvironment.set(stmt.name, resolvedType)
      }
    } else if (stmt instanceof AST.StructDeclaration) {
      const type = result.nodeTypeMap.get(stmt)
      if (type) {
        const resolvedType = result.substitution.apply(type)
        inferredTypes.set(stmt.name, resolvedType)
        typeEnvironment.set(stmt.name, resolvedType)
      }
    }
  }

  return {
    errors: result.errors,
    inferredTypes,
    typeEnvironment,
  }
}

// MethodCall処理のためにTypeInferenceSystemクラスを拡張
