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
      case "FunctionType":
        const ft = type as AST.FunctionType
        return `(${this.typeToString(ft.paramType)} -> ${this.typeToString(ft.returnType)})`
      case "GenericType":
        const gt = type as AST.GenericType
        const args = gt.typeArguments.map(t => this.typeToString(t)).join(", ")
        return `${gt.name}<${args}>`
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
          gt.typeArguments.map(arg => this.apply(arg)),
          gt.line,
          gt.column
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
      case "FunctionType":
        const ft = type as AST.FunctionType
        return `(${this.typeToString(ft.paramType)} -> ${this.typeToString(ft.returnType)})`
      case "GenericType":
        const gt = type as AST.GenericType
        const args = gt.typeArguments.map(t => this.typeToString(t)).join(", ")
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
    substitution: TypeSubstitution, 
    errors: TypeInferenceError[] 
  } {
    this.constraints = []
    this.errors = []
    this.nextVarId = 0

    // 型環境の初期化
    const env = this.createInitialEnvironment()

    // 制約生成
    this.generateConstraints(program, env)

    // 制約解決（単一化）
    const substitution = this.solveConstraints()

    return { substitution, errors: this.errors }
  }

  // 初期型環境を作成
  private createInitialEnvironment(): Map<string, AST.Type> {
    const env = new Map<string, AST.Type>()
    
    // 組み込み関数の型を定義
    
    const toStringType = new AST.FunctionType(
      this.freshTypeVariable(0, 0),
      new AST.PrimitiveType("String", 0, 0),
      0, 0
    )
    env.set("toString", toStringType)

    // Maybe型のコンストラクタを削除（ConstructorExpressionで処理）
    // Either型のコンストラクタを削除（ConstructorExpressionで処理）

    return env
  }

  // 制約生成
  private generateConstraints(program: AST.Program, env: Map<string, AST.Type>): void {
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
        this.generateConstraintsForFunctionDeclaration(statement as AST.FunctionDeclaration, env)
        break
      case "VariableDeclaration":
        this.generateConstraintsForVariableDeclaration(statement as AST.VariableDeclaration, env)
        break
      case "ExpressionStatement":
        this.generateConstraintsForExpression((statement as AST.ExpressionStatement).expression, env)
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
    const returnType = func.returnType || this.freshTypeVariable(func.line, func.column)
    
    // 関数の型を構築
    let funcType: AST.Type = returnType
    
    // パラメータから関数型を構築（カリー化）
    for (let i = func.parameters.length - 1; i >= 0; i--) {
      const paramType = func.parameters[i].type || this.freshTypeVariable(func.parameters[i].line, func.parameters[i].column)
      funcType = new AST.FunctionType(
        paramType,
        funcType,
        func.line,
        func.column
      )
    }

    // 関数を環境に追加
    env.set(func.name, funcType)

    // 関数本体の型推論用の環境を作成
    const bodyEnv = new Map(env)
    for (const param of func.parameters) {
      const paramType = param.type || this.freshTypeVariable(param.line, param.column)
      bodyEnv.set(param.name, paramType)
    }

    // 関数本体の型を推論
    const bodyType = this.generateConstraintsForExpression(func.body, bodyEnv)

    // 関数本体の型と戻り値型が一致することを制約として追加
    this.addConstraint(new TypeConstraint(
      bodyType,
      returnType,
      func.body.line,
      func.body.column,
      `Function ${func.name} body type`
    ))
  }

  private generateConstraintsForVariableDeclaration(
    varDecl: AST.VariableDeclaration,
    env: Map<string, AST.Type>
  ): AST.Type {
    // 初期化式の型を推論
    const initType = this.generateConstraintsForExpression(varDecl.initializer, env)

    if (varDecl.type) {
      // 明示的な型注釈がある場合は制約を追加
      this.addConstraint(new TypeConstraint(
        initType,
        varDecl.type,
        varDecl.line,
        varDecl.column,
        `Variable ${varDecl.name} type annotation`
      ))
      env.set(varDecl.name, varDecl.type)
      return varDecl.type
    } else {
      // 型注釈がない場合は推論された型を使用
      // ここで型変数の場合も正しく処理
      const inferredType = initType.kind === "TypeVariable" ? initType : initType
      env.set(varDecl.name, inferredType)
      return inferredType
    }
  }

  private generateConstraintsForExpression(
    expr: AST.Expression,
    env: Map<string, AST.Type>
  ): AST.Type {
    switch (expr.kind) {
      case "Literal":
        return this.generateConstraintsForLiteral(expr as AST.Literal)
      
      case "Identifier":
        return this.generateConstraintsForIdentifier(expr as AST.Identifier, env)
      
      case "BinaryOperation":
        return this.generateConstraintsForBinaryOperation(expr as AST.BinaryOperation, env)
      
      case "FunctionCall":
        return this.generateConstraintsForFunctionCall(expr as AST.FunctionCall, env)
      
      case "FunctionApplication":
        return this.generateConstraintsForFunctionApplication(expr as AST.FunctionApplication, env)
      
      case "Pipeline":
        return this.generateConstraintsForPipeline(expr as AST.Pipeline, env)
      
      case "ConditionalExpression":
        return this.generateConstraintsForConditional(expr as AST.ConditionalExpression, env)
      
      case "BlockExpression":
        return this.generateConstraintsForBlockExpression(expr as AST.BlockExpression, env)
      
      case "ConstructorExpression":
        return this.generateConstraintsForConstructorExpression(expr as AST.ConstructorExpression, env)
      
      default:
        this.errors.push(new TypeInferenceError(
          `Unhandled expression type: ${expr.kind}`,
          expr.line,
          expr.column
        ))
        return this.freshTypeVariable(expr.line, expr.column)
    }
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
    const type = env.get(identifier.name)
    if (!type) {
      this.errors.push(new TypeInferenceError(
        `Undefined variable: ${identifier.name}`,
        identifier.line,
        identifier.column
      ))
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
      case "-":
      case "*":
      case "/":
      case "%":
        // 数値演算: 両オペランドは Int または Float 型、結果も同じ型
        const intType = new AST.PrimitiveType("Int", binOp.line, binOp.column)
        const floatType = new AST.PrimitiveType("Float", binOp.line, binOp.column)
        
        // まず Int 型として制約を追加
        this.addConstraint(new TypeConstraint(
          leftType,
          intType,
          binOp.left.line,
          binOp.left.column,
          `Binary operation ${binOp.operator} left operand`
        ))
        this.addConstraint(new TypeConstraint(
          rightType,
          intType,
          binOp.right.line,
          binOp.right.column,
          `Binary operation ${binOp.operator} right operand`
        ))
        return intType

      case "==":
      case "!=":
      case "<":
      case ">":
      case "<=":
      case ">=":
        // 比較演算: 両オペランドは同じ型、結果はBool
        this.addConstraint(new TypeConstraint(
          leftType,
          rightType,
          binOp.line,
          binOp.column,
          `Comparison ${binOp.operator} operands must match`
        ))
        return new AST.PrimitiveType("Bool", binOp.line, binOp.column)

      case "&&":
      case "||":
        // 論理演算: 両オペランドはBool、結果もBool
        const boolType = new AST.PrimitiveType("Bool", binOp.line, binOp.column)
        this.addConstraint(new TypeConstraint(
          leftType,
          boolType,
          binOp.left.line,
          binOp.left.column,
          `Logical operation ${binOp.operator} left operand`
        ))
        this.addConstraint(new TypeConstraint(
          rightType,
          boolType,
          binOp.right.line,
          binOp.right.column,
          `Logical operation ${binOp.operator} right operand`
        ))
        return boolType

      default:
        this.errors.push(new TypeInferenceError(
          `Unknown binary operator: ${binOp.operator}`,
          binOp.line,
          binOp.column
        ))
        return this.freshTypeVariable(binOp.line, binOp.column)
    }
  }

  private generateConstraintsForFunctionCall(
    call: AST.FunctionCall,
    env: Map<string, AST.Type>
  ): AST.Type {
    // print関数の特別処理
    if (call.function.kind === "Identifier" && (call.function as AST.Identifier).name === "print") {
      if (call.arguments.length === 1) {
        // print関数は任意の型を受け取り、Unit型を返す
        this.generateConstraintsForExpression(call.arguments[0], env)
        return new AST.PrimitiveType("Unit", call.line, call.column)
      }
    }
    
    const funcType = this.generateConstraintsForExpression(call.function, env)
    
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
      
      this.addConstraint(new TypeConstraint(
        resultType,
        expectedFuncType,
        call.line,
        call.column,
        `Function application`
      ))
      
      resultType = newResultType
    }
    
    return resultType
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
    
    this.addConstraint(new TypeConstraint(
      funcType,
      expectedFuncType,
      app.line,
      app.column,
      `Function application`
    ))
    
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
    
    this.addConstraint(new TypeConstraint(
      rightType,
      expectedFuncType,
      pipe.line,
      pipe.column,
      `Pipeline operator`
    ))
    
    return resultType
  }

  private generateConstraintsForConditional(
    cond: AST.ConditionalExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    const condType = this.generateConstraintsForExpression(cond.condition, env)
    const thenType = this.generateConstraintsForExpression(cond.thenExpression, env)
    const elseType = this.generateConstraintsForExpression(cond.elseExpression, env)
    
    // 条件はBool型でなければならない
    this.addConstraint(new TypeConstraint(
      condType,
      new AST.PrimitiveType("Bool", cond.condition.line, cond.condition.column),
      cond.condition.line,
      cond.condition.column,
      `Conditional expression condition`
    ))
    
    // thenとelseの分岐は同じ型でなければならない
    this.addConstraint(new TypeConstraint(
      thenType,
      elseType,
      cond.line,
      cond.column,
      `Conditional expression branches`
    ))
    
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
      return this.generateConstraintsForExpression(block.returnExpression, blockEnv)
    }
    
    // If no explicit return expression, check if the last statement is an expression
    if (block.statements.length > 0) {
      const lastStatement = block.statements[block.statements.length - 1]
      if (lastStatement.kind === "ExpressionStatement") {
        const exprStmt = lastStatement as AST.ExpressionStatement
        return this.generateConstraintsForExpression(exprStmt.expression, blockEnv)
      }
    }
    
    // If no return expression and last statement is not an expression, return Unit
    return new AST.PrimitiveType("Unit", block.line, block.column)
  }

  private generateConstraintsForConstructorExpression(
    ctor: AST.ConstructorExpression,
    env: Map<string, AST.Type>
  ): AST.Type {
    const constructorName = ctor.constructorName
    
    switch (constructorName) {
      case "Just":
        if (ctor.arguments && ctor.arguments.length > 0) {
          const argType = this.generateConstraintsForExpression(ctor.arguments[0], env)
          return new AST.GenericType(
            "Maybe",
            [argType],
            ctor.line,
            ctor.column
          )
        }
        // Just without arguments - should be error
        this.errors.push(new TypeInferenceError(
          "Just constructor requires exactly one argument",
          ctor.line,
          ctor.column
        ))
        return new AST.GenericType(
          "Maybe",
          [this.freshTypeVariable(ctor.line, ctor.column)],
          ctor.line,
          ctor.column
        )
      
      case "Nothing":
        // Nothing doesn't take arguments
        if (ctor.arguments && ctor.arguments.length > 0) {
          this.errors.push(new TypeInferenceError(
            "Nothing constructor does not take any arguments",
            ctor.line,
            ctor.column
          ))
        }
        return new AST.GenericType(
          "Maybe",
          [this.freshTypeVariable(ctor.line, ctor.column)],
          ctor.line,
          ctor.column
        )
      
      case "Right":
        if (ctor.arguments && ctor.arguments.length > 0) {
          const argType = this.generateConstraintsForExpression(ctor.arguments[0], env)
          return new AST.GenericType(
            "Either",
            [this.freshTypeVariable(ctor.line, ctor.column), argType],
            ctor.line,
            ctor.column
          )
        }
        this.errors.push(new TypeInferenceError(
          "Right constructor requires exactly one argument",
          ctor.line,
          ctor.column
        ))
        return new AST.GenericType(
          "Either",
          [this.freshTypeVariable(ctor.line, ctor.column), this.freshTypeVariable(ctor.line, ctor.column)],
          ctor.line,
          ctor.column
        )
      
      case "Left":
        if (ctor.arguments && ctor.arguments.length > 0) {
          const argType = this.generateConstraintsForExpression(ctor.arguments[0], env)
          return new AST.GenericType(
            "Either",
            [argType, this.freshTypeVariable(ctor.line, ctor.column)],
            ctor.line,
            ctor.column
          )
        }
        this.errors.push(new TypeInferenceError(
          "Left constructor requires exactly one argument",
          ctor.line,
          ctor.column
        ))
        return new AST.GenericType(
          "Either",
          [this.freshTypeVariable(ctor.line, ctor.column), this.freshTypeVariable(ctor.line, ctor.column)],
          ctor.line,
          ctor.column
        )
      
      default:
        this.errors.push(new TypeInferenceError(
          `Unknown constructor: ${constructorName}`,
          ctor.line,
          ctor.column
        ))
        return this.freshTypeVariable(ctor.line, ctor.column)
    }
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
        this.errors.push(new TypeInferenceError(
          `Cannot unify types: ${error}`,
          constraint.line,
          constraint.column,
          constraint.context
        ))
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
        throw new Error(`Infinite type: ${tv1.name} occurs in ${this.typeToString(type2)}`)
      }
      substitution.set(tv1.id, type2)
      return substitution
    }
    
    if (type2.kind === "TypeVariable") {
      const tv2 = type2 as TypeVariable
      if (this.occursCheck(tv2.id, type1)) {
        throw new Error(`Infinite type: ${tv2.name} occurs in ${this.typeToString(type1)}`)
      }
      substitution.set(tv2.id, type1)
      return substitution
    }
    
    // プリミティブ型の場合
    if (type1.kind === "PrimitiveType" && type2.kind === "PrimitiveType") {
      const pt1 = type1 as AST.PrimitiveType
      const pt2 = type2 as AST.PrimitiveType
      if (pt1.name === pt2.name) {
        return substitution
      }
      throw new Error(`Cannot unify ${this.typeToString(type1)} with ${this.typeToString(type2)}`)
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
      
      if (gt1.name !== gt2.name || gt1.typeArguments.length !== gt2.typeArguments.length) {
        throw new Error(`Cannot unify ${this.typeToString(type1)} with ${this.typeToString(type2)}`)
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
    
    throw new Error(`Cannot unify ${this.typeToString(type1)} with ${this.typeToString(type2)}`)
  }

  // Occurs Check: 型変数が型の中に現れるかチェック
  private occursCheck(varId: number, type: AST.Type): boolean {
    switch (type.kind) {
      case "TypeVariable":
        return (type as TypeVariable).id === varId
      
      case "FunctionType":
        const ft = type as AST.FunctionType
        return this.occursCheck(varId, ft.paramType) || this.occursCheck(varId, ft.returnType)
      
      case "GenericType":
        const gt = type as AST.GenericType
        return gt.typeArguments.some(arg => this.occursCheck(varId, arg))
      
      default:
        return false
    }
  }

  // 型の等価性チェック
  private typesEqual(type1: AST.Type, type2: AST.Type): boolean {
    if (type1.kind !== type2.kind) return false
    
    switch (type1.kind) {
      case "PrimitiveType":
        return (type1 as AST.PrimitiveType).name === (type2 as AST.PrimitiveType).name
      
      case "TypeVariable":
        return (type1 as TypeVariable).id === (type2 as TypeVariable).id
      
      case "FunctionType":
        const ft1 = type1 as AST.FunctionType
        const ft2 = type2 as AST.FunctionType
        return this.typesEqual(ft1.paramType, ft2.paramType) && 
               this.typesEqual(ft1.returnType, ft2.returnType)
      
      case "GenericType":
        const gt1 = type1 as AST.GenericType
        const gt2 = type2 as AST.GenericType
        return gt1.name === gt2.name &&
               gt1.typeArguments.length === gt2.typeArguments.length &&
               gt1.typeArguments.every((arg, i) => this.typesEqual(arg, gt2.typeArguments[i]))
      
      default:
        return false
    }
  }

  // 型を文字列に変換
  private typeToString(type: AST.Type): string {
    switch (type.kind) {
      case "PrimitiveType":
        return (type as AST.PrimitiveType).name
      
      case "TypeVariable":
        return (type as TypeVariable).name
      
      case "FunctionType":
        const ft = type as AST.FunctionType
        return `(${this.typeToString(ft.paramType)} -> ${this.typeToString(ft.returnType)})`
      
      case "GenericType":
        const gt = type as AST.GenericType
        const args = gt.typeArguments.map(t => this.typeToString(t)).join(", ")
        return `${gt.name}<${args}>`
      
      default:
        return "Unknown"
    }
  }
}