/**
 * Type Checker for Seseragi Language
 *
 * Performs static type checking and type inference on the AST
 */

import * as AST from "./ast"

// Type environment for tracking variable and function types
export class TypeEnvironment {
  private bindings: Map<string, AST.Type>
  private parent: TypeEnvironment | null

  constructor(parent: TypeEnvironment | null = null) {
    this.bindings = new Map()
    this.parent = parent
  }

  // Define a new binding in the current scope
  define(name: string, type: AST.Type): void {
    this.bindings.set(name, type)
  }

  // Look up a type in the environment (including parent scopes)
  lookup(name: string): AST.Type | undefined {
    const type = this.bindings.get(name)
    if (type) return type
    return this.parent?.lookup(name)
  }

  // Create a child environment
  extend(): TypeEnvironment {
    return new TypeEnvironment(this)
  }
}

// Type error class with enhanced error information
export class TypeError {
  constructor(
    public message: string,
    public line: number,
    public column: number,
    public code?: string,
    public suggestion?: string
  ) {}

  // Format error with context and suggestions
  toString(): string {
    let result = `Error at line ${this.line}, column ${this.column}: ${this.message}`

    if (this.code) {
      result += `\n  Code: ${this.code}`
    }

    if (this.suggestion) {
      result += `\n  Suggestion: ${this.suggestion}`
    }

    return result
  }
}

// Type checker main class
export class TypeChecker {
  private errors: TypeError[] = []
  private globalEnv: TypeEnvironment

  constructor() {
    this.globalEnv = new TypeEnvironment()
    this.initializeBuiltins()
  }

  // Initialize built-in functions
  private initializeBuiltins(): void {
    // print: a -> Unit (polymorphic, accepts any type)
    this.globalEnv.define(
      "print",
      new AST.FunctionType(
        new AST.PrimitiveType("a", 0, 0), // Type variable
        new AST.PrimitiveType("Unit", 0, 0),
        0,
        0
      )
    )

    // putStrLn: a -> Unit (polymorphic, accepts any type)
    this.globalEnv.define(
      "putStrLn",
      new AST.FunctionType(
        new AST.PrimitiveType("a", 0, 0), // Type variable
        new AST.PrimitiveType("Unit", 0, 0),
        0,
        0
      )
    )

    // toString: a -> String (polymorphic)
    this.globalEnv.define(
      "toString",
      new AST.FunctionType(
        new AST.PrimitiveType("a", 0, 0), // Type variable
        new AST.PrimitiveType("String", 0, 0),
        0,
        0
      )
    )
  }

  // Main entry point for type checking
  check(program: AST.Program): TypeError[] {
    this.errors = []
    this.checkProgram(program, this.globalEnv)
    return this.errors
  }

  private checkProgram(program: AST.Program, env: TypeEnvironment): void {
    for (const statement of program.statements) {
      this.checkStatement(statement, env)
    }
  }

  private checkStatement(statement: AST.Statement, env: TypeEnvironment): void {
    switch (statement.kind) {
      case "FunctionDeclaration":
        this.checkFunctionDeclaration(statement as AST.FunctionDeclaration, env)
        break
      case "VariableDeclaration":
        this.checkVariableDeclaration(statement as AST.VariableDeclaration, env)
        break
      case "TypeDeclaration":
        this.checkTypeDeclaration(statement as AST.TypeDeclaration, env)
        break
      case "ExpressionStatement":
        this.checkExpression(
          (statement as AST.ExpressionStatement).expression,
          env
        )
        break
      case "ReturnStatement":
        this.checkExpression((statement as AST.ReturnStatement).expression, env)
        break
      default:
        // Handle other statement types later
        break
    }
  }

  private checkFunctionDeclaration(
    func: AST.FunctionDeclaration,
    env: TypeEnvironment
  ): void {
    // Create function type
    let funcType: AST.Type = func.returnType

    // Build function type from parameters (right to left for currying)
    for (let i = func.parameters.length - 1; i >= 0; i--) {
      funcType = new AST.FunctionType(
        func.parameters[i].type,
        funcType,
        func.line,
        func.column
      )
    }

    // Add function to environment
    env.define(func.name, funcType)

    // Check function body in extended environment
    const bodyEnv = env.extend()
    for (const param of func.parameters) {
      bodyEnv.define(param.name, param.type)
    }

    const bodyType = this.checkExpression(func.body, bodyEnv)

    // Verify return type matches
    if (!this.typesEqual(bodyType, func.returnType)) {
      this.addError(
        `Function '${func.name}' return type mismatch`,
        func.body.line,
        func.body.column,
        `Expected: ${this.typeToString(func.returnType)}, but function body returns: ${this.typeToString(bodyType)}`,
        `Check that all return paths return the declared type '${this.typeToString(func.returnType)}'`
      )
    }
  }

  private checkVariableDeclaration(
    varDecl: AST.VariableDeclaration,
    env: TypeEnvironment
  ): void {
    const initType = this.checkExpression(varDecl.initializer, env)

    if (varDecl.type) {
      // If type is explicitly declared, check it matches
      if (!this.typesEqual(initType, varDecl.type)) {
        this.addError(
          `Variable '${varDecl.name}' type mismatch`,
          varDecl.line,
          varDecl.column,
          `Declared as '${this.typeToString(varDecl.type)}' but initialized with '${this.typeToString(initType)}'`,
          this.getTypeMismatchSuggestion(varDecl.type, initType)
        )
      }
      env.define(varDecl.name, varDecl.type)
    } else {
      // Infer type from initializer
      env.define(varDecl.name, initType)
    }
  }

  private checkTypeDeclaration(
    typeDecl: AST.TypeDeclaration,
    env: TypeEnvironment
  ): void {
    // Register custom type (simplified for now)
    // Full implementation would handle type constructors
  }

  private checkExpression(
    expr: AST.Expression,
    env: TypeEnvironment
  ): AST.Type {
    switch (expr.kind) {
      case "Literal":
        return this.checkLiteral(expr as AST.Literal)

      case "Identifier":
        return this.checkIdentifier(expr as AST.Identifier, env)

      case "BinaryOperation":
        return this.checkBinaryOperation(expr as AST.BinaryOperation, env)

      case "FunctionCall":
        return this.checkFunctionCall(expr as AST.FunctionCall, env)

      case "BuiltinFunctionCall":
        return this.checkBuiltinFunctionCall(
          expr as AST.BuiltinFunctionCall,
          env
        )

      case "FunctionApplication":
        return this.checkFunctionApplication(
          expr as AST.FunctionApplication,
          env
        )

      case "Pipeline":
        return this.checkPipeline(expr as AST.Pipeline, env)

      case "ConditionalExpression":
        return this.checkConditional(expr as AST.ConditionalExpression, env)

      case "MatchExpression":
        return this.checkMatch(expr as AST.MatchExpression, env)

      case "ConstructorExpression":
        return this.checkConstructor(expr as AST.ConstructorExpression, env)

      case "BlockExpression":
        return this.checkBlock(expr as AST.BlockExpression, env)

      default:
        this.addError(
          `Unhandled expression type: ${expr.kind}`,
          expr.line,
          expr.column
        )
        return new AST.PrimitiveType("Unknown", expr.line, expr.column)
    }
  }

  private checkLiteral(lit: AST.Literal): AST.Type {
    switch (lit.literalType) {
      case "string":
        return new AST.PrimitiveType("String", lit.line, lit.column)
      case "integer":
        return new AST.PrimitiveType("Int", lit.line, lit.column)
      case "float":
        return new AST.PrimitiveType("Float", lit.line, lit.column)
      case "boolean":
        return new AST.PrimitiveType("Bool", lit.line, lit.column)
    }
  }

  private checkIdentifier(id: AST.Identifier, env: TypeEnvironment): AST.Type {
    const type = env.lookup(id.name)
    if (!type) {
      this.addError(
        `Undefined variable '${id.name}'`,
        id.line,
        id.column,
        `Variable '${id.name}' is not declared in this scope`,
        `Did you mean to declare it with 'let ${id.name} = ...' or is there a typo?`
      )
      return new AST.PrimitiveType("Unknown", id.line, id.column)
    }
    return type
  }

  private checkBinaryOperation(
    binOp: AST.BinaryOperation,
    env: TypeEnvironment
  ): AST.Type {
    const leftType = this.checkExpression(binOp.left, env)
    const rightType = this.checkExpression(binOp.right, env)

    // Type rules for operators
    switch (binOp.operator) {
      case "+":
        // Addition can be numeric or string concatenation
        if (this.isNumericType(leftType) && this.isNumericType(rightType)) {
          // If both are same type, return that type
          if (this.typesEqual(leftType, rightType)) {
            return leftType
          }
          // If mixed Int/Float, promote to Float
          if (
            (this.isIntType(leftType) && this.isFloatType(rightType)) ||
            (this.isFloatType(leftType) && this.isIntType(rightType))
          ) {
            return new AST.PrimitiveType("Float", binOp.line, binOp.column)
          }
        } else if (
          this.isStringType(leftType) &&
          this.isStringType(rightType)
        ) {
          // String concatenation
          return new AST.PrimitiveType("String", binOp.line, binOp.column)
        }
        this.addError(
          `Invalid operands for '${binOp.operator}' operator`,
          binOp.line,
          binOp.column,
          `Cannot apply '${binOp.operator}' to types '${this.typeToString(leftType)}' and '${this.typeToString(rightType)}'`,
          this.getBinaryOperatorSuggestion(binOp.operator, leftType, rightType)
        )
        return new AST.PrimitiveType("Unknown", binOp.line, binOp.column)

      case "-":
      case "*":
      case "/":
      case "%":
        // Numeric operations only
        if (this.isNumericType(leftType) && this.isNumericType(rightType)) {
          // If both are same type, return that type
          if (this.typesEqual(leftType, rightType)) {
            return leftType
          }
          // If mixed Int/Float, promote to Float
          if (
            (this.isIntType(leftType) && this.isFloatType(rightType)) ||
            (this.isFloatType(leftType) && this.isIntType(rightType))
          ) {
            return new AST.PrimitiveType("Float", binOp.line, binOp.column)
          }
        }
        this.addError(
          `Invalid operands for '${binOp.operator}' operator`,
          binOp.line,
          binOp.column,
          `Arithmetic operations require numeric types, got '${this.typeToString(leftType)}' and '${this.typeToString(rightType)}'`,
          `Use numeric types (Int or Float) for arithmetic operations`
        )
        return new AST.PrimitiveType("Unknown", binOp.line, binOp.column)

      case "==":
      case "!=":
      case "<":
      case ">":
      case "<=":
      case ">=":
        // Comparison operations
        if (!this.typesEqual(leftType, rightType)) {
          this.addError(
            `Type error in comparison '${binOp.operator}': types must match, got '${this.typeToString(leftType)}' and '${this.typeToString(rightType)}'`,
            binOp.line,
            binOp.column
          )
        }
        return new AST.PrimitiveType("Bool", binOp.line, binOp.column)

      case "&&":
      case "||":
        // Boolean operations
        if (!this.isBoolType(leftType) || !this.isBoolType(rightType)) {
          this.addError(
            `Type error in boolean operation '${binOp.operator}': both operands must be Bool`,
            binOp.line,
            binOp.column
          )
        }
        return new AST.PrimitiveType("Bool", binOp.line, binOp.column)

      default:
        this.addError(
          `Unknown operator '${binOp.operator}'`,
          binOp.line,
          binOp.column
        )
        return new AST.PrimitiveType("Unknown", binOp.line, binOp.column)
    }
  }

  private checkFunctionCall(
    call: AST.FunctionCall,
    env: TypeEnvironment
  ): AST.Type {
    const funcType = this.checkExpression(call.function, env)

    // Apply arguments one by one (currying)
    let currentType = funcType
    for (const arg of call.arguments) {
      if (currentType.kind !== "FunctionType") {
        this.addError(
          `Cannot call as function`,
          arg.line,
          arg.column,
          `Attempted to call '${this.typeToString(currentType)}' as a function`,
          `Only function types can be called. Check if this is the correct variable name.`
        )
        return new AST.PrimitiveType("Unknown", call.line, call.column)
      }

      const fnType = currentType as AST.FunctionType
      const argType = this.checkExpression(arg, env)

      if (!this.typesCompatible(fnType.paramType, argType)) {
        this.addError(
          `Function argument type mismatch`,
          arg.line,
          arg.column,
          `Expected '${this.typeToString(fnType.paramType)}' but got '${this.typeToString(argType)}'`,
          this.getTypeMismatchSuggestion(fnType.paramType, argType)
        )
      }

      currentType = fnType.returnType
    }

    return currentType
  }

  private checkBuiltinFunctionCall(
    call: AST.BuiltinFunctionCall,
    env: TypeEnvironment
  ): AST.Type {
    switch (call.functionName) {
      case "print":
      case "putStrLn":
        if (call.arguments.length !== 1) {
          this.addError(
            `${call.functionName} expects exactly 1 argument, got ${call.arguments.length}`,
            call.line,
            call.column
          )
        } else {
          // print and putStrLn accept any type (polymorphic)
          this.checkExpression(call.arguments[0], env)
        }
        return new AST.PrimitiveType("Unit", call.line, call.column)

      case "toString":
        if (call.arguments.length !== 1) {
          this.addError(
            `toString expects exactly 1 argument, got ${call.arguments.length}`,
            call.line,
            call.column
          )
        } else {
          // toString accepts any type
          this.checkExpression(call.arguments[0], env)
        }
        return new AST.PrimitiveType("String", call.line, call.column)

      default:
        this.addError(
          `Unknown builtin function '${call.functionName}'`,
          call.line,
          call.column
        )
        return new AST.PrimitiveType("Unknown", call.line, call.column)
    }
  }

  private checkFunctionApplication(
    app: AST.FunctionApplication,
    env: TypeEnvironment
  ): AST.Type {
    const funcType = this.checkExpression(app.function, env)

    if (funcType.kind !== "FunctionType") {
      this.addError(
        `Cannot apply argument to non-function type '${this.typeToString(funcType)}'`,
        app.line,
        app.column
      )
      return new AST.PrimitiveType("Unknown", app.line, app.column)
    }

    const fnType = funcType as AST.FunctionType
    const argType = this.checkExpression(app.argument, env)

    if (!this.typesCompatible(fnType.paramType, argType)) {
      this.addError(
        `Function argument type mismatch`,
        app.argument.line,
        app.argument.column,
        `Expected '${this.typeToString(fnType.paramType)}' but got '${this.typeToString(argType)}'`,
        this.getTypeMismatchSuggestion(fnType.paramType, argType)
      )
    }

    return fnType.returnType
  }

  private checkPipeline(pipe: AST.Pipeline, env: TypeEnvironment): AST.Type {
    const leftType = this.checkExpression(pipe.left, env)
    const rightType = this.checkExpression(pipe.right, env)

    // Right side must be a function that accepts left side's type
    if (rightType.kind !== "FunctionType") {
      this.addError(
        `Right side of pipeline must be a function, got '${this.typeToString(rightType)}'`,
        pipe.right.line,
        pipe.right.column
      )
      return new AST.PrimitiveType("Unknown", pipe.line, pipe.column)
    }

    const fnType = rightType as AST.FunctionType
    if (!this.typesCompatible(fnType.paramType, leftType)) {
      this.addError(
        `Type mismatch in pipeline: function expects '${this.typeToString(fnType.paramType)}', got '${this.typeToString(leftType)}'`,
        pipe.line,
        pipe.column
      )
    }

    return fnType.returnType
  }

  private checkConditional(
    cond: AST.ConditionalExpression,
    env: TypeEnvironment
  ): AST.Type {
    const condType = this.checkExpression(cond.condition, env)
    if (!this.isBoolType(condType)) {
      this.addError(
        `Condition must be Bool, got '${this.typeToString(condType)}'`,
        cond.condition.line,
        cond.condition.column
      )
    }

    const thenType = this.checkExpression(cond.thenExpression, env)
    const elseType = this.checkExpression(cond.elseExpression, env)

    if (!this.typesEqual(thenType, elseType)) {
      this.addError(
        `Conditional branches must have same type: 'then' has '${this.typeToString(thenType)}', 'else' has '${this.typeToString(elseType)}'`,
        cond.line,
        cond.column
      )
      return new AST.PrimitiveType("Unknown", cond.line, cond.column)
    }

    return thenType
  }

  private checkMatch(
    match: AST.MatchExpression,
    env: TypeEnvironment
  ): AST.Type {
    const exprType = this.checkExpression(match.expression, env)

    if (match.cases.length === 0) {
      this.addError(
        "Match expression must have at least one case",
        match.line,
        match.column
      )
      return new AST.PrimitiveType("Unknown", match.line, match.column)
    }

    // Check first case to determine result type
    const firstCase = match.cases[0]
    const caseEnv = this.checkPattern(firstCase.pattern, exprType, env)
    const resultType = this.checkExpression(firstCase.expression, caseEnv)

    // Check remaining cases
    for (let i = 1; i < match.cases.length; i++) {
      const case_ = match.cases[i]
      const caseEnv = this.checkPattern(case_.pattern, exprType, env)
      const caseType = this.checkExpression(case_.expression, caseEnv)

      if (!this.typesEqual(caseType, resultType)) {
        this.addError(
          `All match cases must have same type: case ${i + 1} has '${this.typeToString(caseType)}', but expected '${this.typeToString(resultType)}'`,
          case_.expression.line,
          case_.expression.column
        )
      }
    }

    return resultType
  }

  private checkPattern(
    pattern: AST.Pattern,
    matchedType: AST.Type,
    env: TypeEnvironment
  ): TypeEnvironment {
    const newEnv = env.extend()

    switch (pattern.kind) {
      case "IdentifierPattern":
        const idPat = pattern as AST.IdentifierPattern
        newEnv.define(idPat.name, matchedType)
        break

      case "LiteralPattern":
        // Check literal type matches
        const litPat = pattern as AST.LiteralPattern
        const litType =
          typeof litPat.value === "string"
            ? new AST.PrimitiveType("String", pattern.line, pattern.column)
            : typeof litPat.value === "number"
              ? new AST.PrimitiveType("Int", pattern.line, pattern.column) // Simplified
              : new AST.PrimitiveType("Bool", pattern.line, pattern.column)

        if (!this.typesEqual(litType, matchedType)) {
          this.addError(
            `Pattern type '${this.typeToString(litType)}' does not match expression type '${this.typeToString(matchedType)}'`,
            pattern.line,
            pattern.column
          )
        }
        break

      case "ConstructorPattern":
        // Handle constructor patterns (simplified for now)
        const ctorPat = pattern as AST.ConstructorPattern
        // TODO: Full implementation would check constructor types
        break
    }

    return newEnv
  }

  private checkConstructor(
    ctor: AST.ConstructorExpression,
    env: TypeEnvironment
  ): AST.Type {
    // Handle Maybe and Either constructors
    switch (ctor.constructorName) {
      case "Just":
        if (ctor.arguments.length !== 1) {
          this.addError(
            "Just expects exactly 1 argument",
            ctor.line,
            ctor.column
          )
          return new AST.PrimitiveType("Unknown", ctor.line, ctor.column)
        }
        const argType = this.checkExpression(ctor.arguments[0], env)
        return new AST.GenericType("Maybe", [argType], ctor.line, ctor.column)

      case "Nothing":
        if (ctor.arguments.length !== 0) {
          this.addError("Nothing expects no arguments", ctor.line, ctor.column)
        }
        // Return Maybe<a> with unspecified type parameter
        return new AST.GenericType(
          "Maybe",
          [new AST.PrimitiveType("a", ctor.line, ctor.column)],
          ctor.line,
          ctor.column
        )

      case "Left":
      case "Right":
        if (ctor.arguments.length !== 1) {
          this.addError(
            `${ctor.constructorName} expects exactly 1 argument`,
            ctor.line,
            ctor.column
          )
          return new AST.PrimitiveType("Unknown", ctor.line, ctor.column)
        }
        const valueType = this.checkExpression(ctor.arguments[0], env)
        // Return Either<L,R> with one type parameter known, using consistent type variables
        const leftType =
          ctor.constructorName === "Left"
            ? valueType
            : new AST.PrimitiveType("l", ctor.line, ctor.column)
        const rightType =
          ctor.constructorName === "Right"
            ? valueType
            : new AST.PrimitiveType("r", ctor.line, ctor.column)
        return new AST.GenericType(
          "Either",
          [leftType, rightType],
          ctor.line,
          ctor.column
        )

      default:
        this.addError(
          `Unknown constructor '${ctor.constructorName}'`,
          ctor.line,
          ctor.column
        )
        return new AST.PrimitiveType("Unknown", ctor.line, ctor.column)
    }
  }

  private checkBlock(
    block: AST.BlockExpression,
    env: TypeEnvironment
  ): AST.Type {
    const blockEnv = env.extend()

    // Check all statements in the block
    for (const stmt of block.statements) {
      this.checkStatement(stmt, blockEnv)
    }

    // Return type is the type of the return expression or Unit if none
    if (block.returnExpression) {
      return this.checkExpression(block.returnExpression, blockEnv)
    } else {
      return new AST.PrimitiveType("Unit", block.line, block.column)
    }
  }

  // Helper methods
  private typesEqual(t1: AST.Type, t2: AST.Type): boolean {
    if (t1.kind !== t2.kind) return false

    switch (t1.kind) {
      case "PrimitiveType":
        const p1 = t1 as AST.PrimitiveType
        const p2 = t2 as AST.PrimitiveType
        // Type variables (single lowercase letters) are compatible with any type
        if (this.isTypeVariable(p1.name) || this.isTypeVariable(p2.name)) {
          return true
        }
        return p1.name === p2.name

      case "FunctionType":
        const f1 = t1 as AST.FunctionType
        const f2 = t2 as AST.FunctionType
        return (
          this.typesEqual(f1.paramType, f2.paramType) &&
          this.typesEqual(f1.returnType, f2.returnType)
        )

      case "GenericType":
        const g1 = t1 as AST.GenericType
        const g2 = t2 as AST.GenericType
        if (g1.name !== g2.name) return false
        if (g1.typeArguments.length !== g2.typeArguments.length) return false
        return g1.typeArguments.every((arg, i) =>
          this.typesEqual(arg, g2.typeArguments[i])
        )

      default:
        return false
    }
  }

  // Check if a type name represents a type variable (single lowercase letter)
  private isTypeVariable(name: string): boolean {
    return name.length === 1 && name >= "a" && name <= "z"
  }

  // Check if two types are compatible (including polymorphic types)
  private typesCompatible(expected: AST.Type, actual: AST.Type): boolean {
    // If expected type is a type variable, it can accept any type
    if (
      expected.kind === "PrimitiveType" &&
      this.isTypeVariable((expected as AST.PrimitiveType).name)
    ) {
      return true
    }

    // Otherwise use normal type equality
    return this.typesEqual(expected, actual)
  }

  private typeToString(type: AST.Type): string {
    switch (type.kind) {
      case "PrimitiveType":
        return (type as AST.PrimitiveType).name

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

  private isNumericType(type: AST.Type): boolean {
    return this.isIntType(type) || this.isFloatType(type)
  }

  private isIntType(type: AST.Type): boolean {
    return (
      type.kind === "PrimitiveType" &&
      (type as AST.PrimitiveType).name === "Int"
    )
  }

  private isFloatType(type: AST.Type): boolean {
    return (
      type.kind === "PrimitiveType" &&
      (type as AST.PrimitiveType).name === "Float"
    )
  }

  private isBoolType(type: AST.Type): boolean {
    return (
      type.kind === "PrimitiveType" &&
      (type as AST.PrimitiveType).name === "Bool"
    )
  }

  private isStringType(type: AST.Type): boolean {
    return (
      type.kind === "PrimitiveType" &&
      (type as AST.PrimitiveType).name === "String"
    )
  }

  private addError(
    message: string,
    line: number,
    column: number,
    code?: string,
    suggestion?: string
  ): void {
    this.errors.push(new TypeError(message, line, column, code, suggestion))
  }

  // Helper methods for better error suggestions
  private getTypeMismatchSuggestion(
    expected: AST.Type,
    actual: AST.Type
  ): string {
    const expectedStr = this.typeToString(expected)
    const actualStr = this.typeToString(actual)

    if (expectedStr === "String" && actualStr === "Int") {
      return "Use toString() to convert Int to String, or declare the variable as Int"
    }
    if (expectedStr === "String" && actualStr === "Float") {
      return "Use toString() to convert Float to String, or declare the variable as Float"
    }
    if (expectedStr === "Int" && actualStr === "String") {
      return "Remove quotes to make it a number, or declare the variable as String"
    }
    if (expectedStr === "Float" && actualStr === "Int") {
      return "Add a decimal point (e.g., 42.0) to make it a Float, or declare the variable as Int"
    }

    return `Change the value to match type '${expectedStr}' or change the type annotation to '${actualStr}'`
  }

  private getBinaryOperatorSuggestion(
    operator: string,
    leftType: AST.Type,
    rightType: AST.Type
  ): string {
    const leftStr = this.typeToString(leftType)
    const rightStr = this.typeToString(rightType)

    if (operator === "+") {
      if (leftStr === "String" || rightStr === "String") {
        return "For string concatenation, both operands must be String. Use toString() to convert other types"
      }
      return "For addition, both operands must be numeric (Int or Float)"
    }

    if (["-", "*", "/", "%"].includes(operator)) {
      return "Arithmetic operations require both operands to be numeric (Int or Float)"
    }

    if (["==", "!=", "<", ">", "<=", ">="].includes(operator)) {
      return "Comparison operations require both operands to have the same type"
    }

    if (["&&", "||"].includes(operator)) {
      return "Boolean operations require both operands to be Bool"
    }

    return "Check the types of both operands"
  }
}
