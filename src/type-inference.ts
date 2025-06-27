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
      case "FunctionType":
        const ft = type as AST.FunctionType
        return `(${this.typeToString(ft.paramType)} -> ${this.typeToString(ft.returnType)})`
      case "GenericType":
        const gt = type as AST.GenericType
        const args = gt.typeArguments
          .map((t) => this.typeToString(t))
          .join(", ")
        return `${gt.name}<${args}>`
      case "RecordType":
        const rt = type as AST.RecordType
        const fields = rt.fields
          .map((field) => `${field.name}: ${this.typeToString(field.type)}`)
          .join(", ")
        return `{${fields}}`
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
      case "TypeVariable":
        const tv = type as TypeVariable
        const substituted = this.get(tv.id)
        return substituted ? this.apply(substituted) : type

      case "PolymorphicTypeVariable":
        // 多相型変数は置換しない（常に多相のまま）
        return type

      case "FunctionType":
        const ft = type as AST.FunctionType
        return new AST.FunctionType(
          this.apply(ft.paramType),
          this.apply(ft.returnType),
          ft.line,
          ft.column
        )

      case "GenericType":
        const gt = type as AST.GenericType
        return new AST.GenericType(
          gt.name,
          gt.typeArguments.map((arg) => this.apply(arg)),
          gt.line,
          gt.column
        )

      case "RecordType":
        const rt = type as AST.RecordType
        return new AST.RecordType(
          rt.fields.map((field) => 
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
      case "FunctionType":
        const ft = type as AST.FunctionType
        return `(${this.typeToString(ft.paramType)} -> ${this.typeToString(ft.returnType)})`
      case "GenericType":
        const gt = type as AST.GenericType
        const args = gt.typeArguments
          .map((t) => this.typeToString(t))
          .join(", ")
        return `${gt.name}<${args}>`
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

// 型推論システムのメインクラス
export class TypeInferenceSystem {
  private nextVarId = 0
  private constraints: TypeConstraint[] = []
  private errors: TypeInferenceError[] = []
  private nodeTypeMap: Map<any, AST.Type> = new Map() // Track types for AST nodes

  // 新しい型変数を生成
  freshTypeVariable(line: number, column: number): TypeVariable {
    return new TypeVariable(this.nextVarId++, line, column)
  }

  // 制約を追加
  addConstraint(constraint: TypeConstraint): void {
    this.constraints.push(constraint)
  }

  // 型推論のメインエントリーポイント
  infer(program: AST.Program): {
    substitution: TypeSubstitution
    errors: TypeInferenceError[]
    nodeTypeMap: Map<any, AST.Type>
  } {
    this.constraints = []
    this.errors = []
    this.nextVarId = 0
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

    // Maybe型のコンストラクタを削除（ConstructorExpressionで処理）
    // Either型のコンストラクタを削除（ConstructorExpressionで処理）

    return env
  }

  // 制約生成
  private generateConstraints(
    program: AST.Program,
    env: Map<string, AST.Type>
  ): void {
    for (const statement of program.statements) {
      this.generateConstraintsForStatement(statement, env)
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

    // 関数の型を構築
    let funcType: AST.Type = returnType

    // パラメータから関数型を構築（カリー化）
    if (func.parameters.length === 0) {
      // 引数なしの関数は Unit -> ReturnType
      funcType = new AST.FunctionType(
        new AST.PrimitiveType("Unit", func.line, func.column),
        funcType,
        func.line,
        func.column
      )
    } else {
      // 引数ありの関数は通常のカリー化
      for (let i = func.parameters.length - 1; i >= 0; i--) {
        const paramType =
          func.parameters[i].type ||
          this.freshTypeVariable(
            func.parameters[i].line,
            func.parameters[i].column
          )
        funcType = new AST.FunctionType(
          paramType,
          funcType,
          func.line,
          func.column
        )
      }
    }

    // 関数を環境に追加
    env.set(func.name, funcType)

    // 関数本体の型推論用の環境を作成
    const bodyEnv = new Map(env)
    for (const param of func.parameters) {
      const paramType =
        param.type || this.freshTypeVariable(param.line, param.column)
      bodyEnv.set(param.name, paramType)
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
      // 明示的な型注釈がある場合は制約を追加
      this.addConstraint(
        new TypeConstraint(
          initType,
          varDecl.type,
          varDecl.line,
          varDecl.column,
          `Variable ${varDecl.name} type annotation`
        )
      )
      env.set(varDecl.name, varDecl.type)
      finalType = varDecl.type
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

  private generateConstraintsForTypeDeclaration(
    typeDecl: AST.TypeDeclaration,
    env: Map<string, AST.Type>
  ): void {
    // ADT型を環境に追加
    const adtType = new AST.PrimitiveType(typeDecl.name, typeDecl.line, typeDecl.column)

    // ADT型自体を環境に登録
    env.set(typeDecl.name, adtType)

    // 各バリアント（コンストラクタ）を環境に追加
    for (const field of typeDecl.fields) {
      const constructorType = this.createConstructorType(field, adtType)
      env.set(field.name, constructorType)
    }
  }

  private createConstructorType(field: AST.TypeField, adtType: AST.Type): AST.Type {
    if (field.type instanceof AST.PrimitiveType && field.type.name === "Unit") {
      // データなしのコンストラクタ (Red, Green, Blue)
      return adtType
    } else if (field.type instanceof AST.GenericType && field.type.name === "Tuple") {
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
      return new AST.FunctionType(
        field.type,
        adtType,
        field.line,
        field.column
      )
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
    const name = identifier.name
    
    // Special handling for constructor names used as values
    switch (name) {
      case "Just":
        // Just : 'a -> Maybe<'a>
        const justElemType = new PolymorphicTypeVariable("a", identifier.line, identifier.column)
        const justMaybeType = new AST.GenericType(
          "Maybe",
          [justElemType],
          identifier.line,
          identifier.column
        )
        return new AST.FunctionType(justElemType, justMaybeType, identifier.line, identifier.column)
        
      case "Nothing":
        // Nothing : Maybe<'a>
        return new AST.GenericType(
          "Maybe",
          [new PolymorphicTypeVariable("a", identifier.line, identifier.column)],
          identifier.line,
          identifier.column
        )
        
      case "Left":
        // Left : 'a -> Either<'a, 'b>
        const leftLeftType = new PolymorphicTypeVariable("a", identifier.line, identifier.column)
        const leftRightType = new PolymorphicTypeVariable("b", identifier.line, identifier.column)
        const leftEitherType = new AST.GenericType(
          "Either",
          [leftLeftType, leftRightType],
          identifier.line,
          identifier.column
        )
        return new AST.FunctionType(leftLeftType, leftEitherType, identifier.line, identifier.column)
        
      case "Right":
        // Right : 'b -> Either<'a, 'b>
        const rightLeftType = new PolymorphicTypeVariable("a", identifier.line, identifier.column)
        const rightRightType = new PolymorphicTypeVariable("b", identifier.line, identifier.column)
        const rightEitherType = new AST.GenericType(
          "Either",
          [rightLeftType, rightRightType],
          identifier.line,
          identifier.column
        )
        return new AST.FunctionType(rightRightType, rightEitherType, identifier.line, identifier.column)
        
      case "Empty":
        // Empty : List<'a>
        return new AST.GenericType(
          "List",
          [new PolymorphicTypeVariable("a", identifier.line, identifier.column)],
          identifier.line,
          identifier.column
        )
        
      case "Cons":
        // Cons : 'a -> List<'a> -> List<'a>
        const consElemType = new PolymorphicTypeVariable("a", identifier.line, identifier.column)
        const consListType = new AST.GenericType(
          "List",
          [consElemType],
          identifier.line,
          identifier.column
        )
        return new AST.FunctionType(
          consElemType,
          new AST.FunctionType(consListType, consListType, identifier.line, identifier.column),
          identifier.line,
          identifier.column
        )
    }
    
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
    return type
  }

  private generateConstraintsForBinaryOperation(
    binOp: AST.BinaryOperation,
    env: Map<string, AST.Type>
  ): AST.Type {
    const leftType = this.generateConstraintsForExpression(binOp.left, env)
    const rightType = this.generateConstraintsForExpression(binOp.right, env)

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
      case "%":
        // 数値演算のみ: 両オペランドは Int または Float 型、結果も同じ型
        const numIntType = new AST.PrimitiveType("Int", binOp.line, binOp.column)
        
        // まず Int 型として制約を追加
        this.addConstraint(
          new TypeConstraint(
            leftType,
            numIntType,
            binOp.left.line,
            binOp.left.column,
            `Binary operation ${binOp.operator} left operand`
          )
        )
        this.addConstraint(
          new TypeConstraint(
            rightType,
            numIntType,
            binOp.right.line,
            binOp.right.column,
            `Binary operation ${binOp.operator} right operand`
          )
        )
        return numIntType

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
      case "||":
        // 論理演算: 両オペランドはBool、結果もBool
        const boolType = new AST.PrimitiveType("Bool", binOp.line, binOp.column)
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
    const operandType = this.generateConstraintsForExpression(unaryOp.operand, env)

    switch (unaryOp.operator) {
      case "-":
        // 数値の単項マイナス: Int -> Int, Float -> Float
        const intType = new AST.PrimitiveType("Int", unaryOp.line, unaryOp.column)
        const floatType = new AST.PrimitiveType("Float", unaryOp.line, unaryOp.column)
        
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

      case "!":
        // 論理否定: Bool -> Bool
        const boolType = new AST.PrimitiveType("Bool", unaryOp.line, unaryOp.column)
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
    let resultType = funcType

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
      }
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
          const elemType = new PolymorphicTypeVariable("a", ctor.line, ctor.column)
          const maybeType = new AST.GenericType(
            "Maybe",
            [elemType],
            ctor.line,
            ctor.column
          )
          return new AST.FunctionType(elemType, maybeType, ctor.line, ctor.column)
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
          const leftType = new PolymorphicTypeVariable("a", ctor.line, ctor.column)
          const rightType = new PolymorphicTypeVariable("b", ctor.line, ctor.column)
          const eitherType = new AST.GenericType(
            "Either",
            [leftType, rightType],
            ctor.line,
            ctor.column
          )
          return new AST.FunctionType(rightType, eitherType, ctor.line, ctor.column)
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
          const leftType = new PolymorphicTypeVariable("a", ctor.line, ctor.column)
          const rightType = new PolymorphicTypeVariable("b", ctor.line, ctor.column)
          const eitherType = new AST.GenericType(
            "Either",
            [leftType, rightType],
            ctor.line,
            ctor.column
          )
          return new AST.FunctionType(leftType, eitherType, ctor.line, ctor.column)
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
          const elemType = new PolymorphicTypeVariable("a", ctor.line, ctor.column)
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

      default:
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
      const argType = this.generateConstraintsForExpression(ctor.arguments[i], env)
      
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
      if (paramType.kind === "PrimitiveType" && 
          (paramType as AST.PrimitiveType).name === "_") {
        paramType = this.freshTypeVariable(param.line, param.column)
      }
      
      parameterTypes.push(paramType)
      lambdaEnv.set(param.name, paramType)
    }

    // Infer the type of the lambda body
    const bodyType = this.generateConstraintsForExpression(lambda.body, lambdaEnv)

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
    const substitution = new TypeSubstitution()

    // 同じ型の場合
    if (this.typesEqual(type1, type2)) {
      return substitution
    }

    // 型変数の場合
    if (type1.kind === "TypeVariable") {
      const tv1 = type1 as TypeVariable
      if (this.occursCheck(tv1.id, type2)) {
        throw new Error(
          `Infinite type: ${tv1.name} occurs in ${this.typeToString(type2)}`
        )
      }
      substitution.set(tv1.id, type2)
      return substitution
    }

    if (type2.kind === "TypeVariable") {
      const tv2 = type2 as TypeVariable
      if (this.occursCheck(tv2.id, type1)) {
        throw new Error(
          `Infinite type: ${tv2.name} occurs in ${this.typeToString(type1)}`
        )
      }
      substitution.set(tv2.id, type1)
      return substitution
    }

    // 多相型変数の場合 - これらは常に多相のまま残す
    if (
      type1.kind === "PolymorphicTypeVariable" ||
      type2.kind === "PolymorphicTypeVariable"
    ) {
      // 多相型変数は統一しない（常に多相のまま）
      return substitution
    }

    // プリミティブ型の場合
    if (type1.kind === "PrimitiveType" && type2.kind === "PrimitiveType") {
      const pt1 = type1 as AST.PrimitiveType
      const pt2 = type2 as AST.PrimitiveType
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

      if (
        gt1.name !== gt2.name ||
        gt1.typeArguments.length !== gt2.typeArguments.length
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

    // Record型の場合
    if (type1.kind === "RecordType" && type2.kind === "RecordType") {
      const rt1 = type1 as AST.RecordType
      const rt2 = type2 as AST.RecordType

      // 構造的部分型：一方が他方のサブセットの場合は統一可能
      // 長いレコードの方を基準にして、短いレコードがサブセットかチェック
      const [largerRecord, smallerRecord] = rt1.fields.length >= rt2.fields.length ? [rt1, rt2] : [rt2, rt1]
      const isSubset = this.isRecordSubset(smallerRecord, largerRecord)

      if (isSubset) {
        // サブセット関係がある場合、共通フィールドを統一
        let result = substitution
        for (const smallerField of smallerRecord.fields) {
          const largerField = largerRecord.fields.find(f => f.name === smallerField.name)
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
      const fields1 = [...rt1.fields].sort((a, b) => a.name.localeCompare(b.name))
      const fields2 = [...rt2.fields].sort((a, b) => a.name.localeCompare(b.name))

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

    // Record型と他の型の部分的統一（構造的部分型）
    if (type1.kind === "RecordType" || type2.kind === "RecordType") {
      // どちらか一方がRecordTypeの場合、構造的部分型をチェック
      const recordType = type1.kind === "RecordType" ? type1 as AST.RecordType : type2 as AST.RecordType
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

  // 構造的部分型：小さいレコードが大きいレコードのサブセットかどうかチェック
  private isRecordSubset(smallerRecord: AST.RecordType, largerRecord: AST.RecordType): boolean {
    // 小さいレコードのすべてのフィールドが大きいレコードに存在するかチェック
    for (const smallerField of smallerRecord.fields) {
      const largerField = largerRecord.fields.find(f => f.name === smallerField.name)
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

      case "FunctionType":
        const ft = type as AST.FunctionType
        return (
          this.occursCheck(varId, ft.paramType) ||
          this.occursCheck(varId, ft.returnType)
        )

      case "GenericType":
        const gt = type as AST.GenericType
        return gt.typeArguments.some((arg) => this.occursCheck(varId, arg))

      case "RecordType":
        const rt = type as AST.RecordType
        return rt.fields.some((field) => this.occursCheck(varId, field.type))

      default:
        return false
    }
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

      case "FunctionType":
        const ft1 = type1 as AST.FunctionType
        const ft2 = type2 as AST.FunctionType
        return (
          this.typesEqual(ft1.paramType, ft2.paramType) &&
          this.typesEqual(ft1.returnType, ft2.returnType)
        )

      case "GenericType":
        const gt1 = type1 as AST.GenericType
        const gt2 = type2 as AST.GenericType
        return (
          gt1.name === gt2.name &&
          gt1.typeArguments.length === gt2.typeArguments.length &&
          gt1.typeArguments.every((arg, i) =>
            this.typesEqual(arg, gt2.typeArguments[i])
          )
        )

      case "RecordType":
        const rt1 = type1 as AST.RecordType
        const rt2 = type2 as AST.RecordType
        
        if (rt1.fields.length !== rt2.fields.length) {
          return false
        }

        // フィールド名でソートして比較
        const fields1 = [...rt1.fields].sort((a, b) => a.name.localeCompare(b.name))
        const fields2 = [...rt2.fields].sort((a, b) => a.name.localeCompare(b.name))

        return fields1.every((field1, i) => {
          const field2 = fields2[i]
          return field1.name === field2.name && this.typesEqual(field1.type, field2.type)
        })

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

      case "FunctionType":
        const ft = type as AST.FunctionType
        return `(${this.typeToString(ft.paramType)} -> ${this.typeToString(ft.returnType)})`

      case "GenericType":
        const gt = type as AST.GenericType
        const args = gt.typeArguments
          .map((t) => this.typeToString(t))
          .join(", ")
        return `${gt.name}<${args}>`

      case "RecordType":
        const rt = type as AST.RecordType
        const fields = rt.fields
          .map((field) => `${field.name}: ${this.typeToString(field.type)}`)
          .join(", ")
        return `{${fields}}`

      default:
        return "Unknown"
    }
  }

  private generateConstraintsForMatchExpression(
    match: AST.MatchExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    // マッチ対象の式の型を推論
    const exprType = this.generateConstraintsForExpression(match.expression, env)

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
    const resultType = this.generateConstraintsForExpression(firstCase.expression, patternEnv)

    // 残りのケースの型をチェックし、すべて同じ型になるよう制約を追加
    for (let i = 1; i < match.cases.length; i++) {
      const caseItem = match.cases[i]
      
      // パターンマッチングで新しい変数環境を作成
      const caseEnv = new Map(env)
      this.generateConstraintsForPattern(caseItem.pattern, exprType, caseEnv)
      
      // ケースの結果型を推論
      const caseResultType = this.generateConstraintsForExpression(caseItem.expression, caseEnv)
      
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
      case "IdentifierPattern":
        // 変数パターン: 変数を環境に追加
        const idPattern = pattern as AST.IdentifierPattern
        env.set(idPattern.name, expectedType)
        break

      case "LiteralPattern":
        // リテラルパターン: 期待する型と一致するかチェック
        const litPattern = pattern as AST.LiteralPattern
        let literalType: AST.Type
        
        switch (typeof litPattern.value) {
          case "string":
            literalType = new AST.PrimitiveType("String", pattern.line, pattern.column)
            break
          case "number":
            literalType = new AST.PrimitiveType("Int", pattern.line, pattern.column)
            break
          case "boolean":
            literalType = new AST.PrimitiveType("Bool", pattern.line, pattern.column)
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

      case "ConstructorPattern":
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
    const fields: AST.RecordField[] = []

    for (const initField of record.fields) {
      const fieldType = this.generateConstraintsForExpression(initField.value, env)
      fields.push(
        new AST.RecordField(
          initField.name,
          fieldType,
          initField.line,
          initField.column
        )
      )
    }

    return new AST.RecordType(fields, record.line, record.column)
  }

  private generateConstraintsForRecordAccess(
    access: AST.RecordAccess,
    env: Map<string, AST.Type>
  ): AST.Type {
    const recordType = this.generateConstraintsForExpression(access.record, env)
    const fieldType = this.freshTypeVariable(access.line, access.column)

    // Create a constraint that the record type must have the specified field
    // This is a structural typing constraint
    const expectedRecordType = new AST.RecordType(
      [new AST.RecordField(access.fieldName, fieldType, access.line, access.column)],
      access.line,
      access.column
    )

    // Add constraint that record must be compatible with having this field
    this.addConstraint(
      new TypeConstraint(
        recordType,
        expectedRecordType,
        access.line,
        access.column,
        `Record access .${access.fieldName}`
      )
    )

    return fieldType
  }
}
