/**
 * Type Checker for Seseragi Language
 *
 * Performs static type checking and type inference on the AST
 */

import * as AST from "./ast"
import { ModuleResolver } from "./module-resolver"

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
export class SeseragiTypeError {
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
  private errors: SeseragiTypeError[] = []
  private globalEnv: TypeEnvironment
  private moduleResolver: ModuleResolver
  private currentFilePath: string = ""

  constructor(typeEnvironment?: Map<string, AST.Type>, filePath?: string) {
    this.globalEnv = new TypeEnvironment()
    this.initializeBuiltins()

    this.moduleResolver = new ModuleResolver()
    this.currentFilePath = filePath || ""

    // Add types from type inference system if provided
    if (typeEnvironment) {
      for (const [name, type] of typeEnvironment) {
        this.globalEnv.define(name, type)
      }
    }
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

    // show: a -> Unit (polymorphic)
    this.globalEnv.define(
      "show",
      new AST.FunctionType(
        new AST.PrimitiveType("a", 0, 0), // Type variable
        new AST.PrimitiveType("Unit", 0, 0),
        0,
        0
      )
    )
  }

  // Main entry point for type checking
  check(program: AST.Program): SeseragiTypeError[] {
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
      case "ImportDeclaration":
        this.checkImportDeclaration(statement as AST.ImportDeclaration, env)
        break
      case "FunctionDeclaration":
        this.checkFunctionDeclaration(statement as AST.FunctionDeclaration, env)
        break
      case "VariableDeclaration":
        this.checkVariableDeclaration(statement as AST.VariableDeclaration, env)
        break
      case "TypeDeclaration":
        this.checkTypeDeclaration(statement as AST.TypeDeclaration, env)
        break
      case "StructDeclaration":
        this.checkStructDeclaration(statement as AST.StructDeclaration, env)
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
      // Resolve parameter type aliases
      let paramType = func.parameters[i].type
      if (paramType.kind === "PrimitiveType") {
        const typeName = (paramType as AST.PrimitiveType).name
        const resolvedType = env.lookup(typeName)
        if (resolvedType && resolvedType.kind === "StructType") {
          paramType = resolvedType
        }
      }

      funcType = new AST.FunctionType(
        paramType,
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
      // Resolve type aliases for parameters
      let resolvedType = param.type
      if (param.type.kind === "PrimitiveType") {
        const typeName = (param.type as AST.PrimitiveType).name

        // Try to resolve it to a struct type from the environment
        const structType = env.lookup(typeName)
        if (structType && structType.kind === "StructType") {
          resolvedType = structType
        }
      }

      bodyEnv.define(param.name, resolvedType)
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
    _typeDecl: AST.TypeDeclaration,
    _env: TypeEnvironment
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

      case "UnaryOperation":
        return this.checkUnaryOperation(expr as AST.UnaryOperation, env)

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

      case "LambdaExpression":
        return this.checkLambda(expr as AST.LambdaExpression, env)

      case "RecordExpression":
        return this.checkRecordExpression(expr as AST.RecordExpression, env)

      case "RecordAccess":
        return this.checkRecordAccess(expr as AST.RecordAccess, env)

      case "StructExpression":
        return this.checkStructExpression(expr as AST.StructExpression, env)

      case "TemplateExpression":
        return this.checkTemplateExpression(expr as AST.TemplateExpression, env)

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
      case "unit":
        return new AST.PrimitiveType("Unit", lit.line, lit.column)
      default:
        return new AST.PrimitiveType("Unknown", lit.line, lit.column)
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

    // Check for struct operator overloading first
    if (leftType.kind === "StructType") {
      return leftType
    }

    return this.checkBinaryOperatorType(binOp, leftType, rightType)
  }

  private checkBinaryOperatorType(
    binOp: AST.BinaryOperation,
    leftType: AST.Type,
    rightType: AST.Type
  ): AST.Type {
    switch (binOp.operator) {
      case "+":
        return this.checkAdditionOperator(binOp, leftType, rightType)

      case "-":
      case "*":
      case "/":
      case "%":
      case "**":
        return this.checkArithmeticOperator(binOp, leftType, rightType)

      case "==":
      case "!=":
      case "<":
      case ">":
      case "<=":
      case ">=":
        return this.checkComparisonOperator(binOp, leftType, rightType)

      case "&&":
      case "||":
        return this.checkBooleanOperator(binOp, leftType, rightType)

      default:
        this.addError(
          `Unknown operator '${binOp.operator}'`,
          binOp.line,
          binOp.column
        )
        return new AST.PrimitiveType("Unknown", binOp.line, binOp.column)
    }
  }

  private checkAdditionOperator(
    binOp: AST.BinaryOperation,
    leftType: AST.Type,
    rightType: AST.Type
  ): AST.Type {
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
    } else if (this.isStringType(leftType) && this.isStringType(rightType)) {
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
  }

  private checkArithmeticOperator(
    binOp: AST.BinaryOperation,
    leftType: AST.Type,
    rightType: AST.Type
  ): AST.Type {
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
      `Cannot apply '${binOp.operator}' to types '${this.typeToString(leftType)}' and '${this.typeToString(rightType)}'`,
      this.getBinaryOperatorSuggestion(binOp.operator, leftType, rightType)
    )
    return new AST.PrimitiveType("Unknown", binOp.line, binOp.column)
  }

  private checkComparisonOperator(
    binOp: AST.BinaryOperation,
    leftType: AST.Type,
    rightType: AST.Type
  ): AST.Type {
    // Comparison operations
    if (!this.typesEqual(leftType, rightType)) {
      this.addError(
        `Type error in comparison '${binOp.operator}': types must match, got '${this.typeToString(leftType)}' and '${this.typeToString(rightType)}'`,
        binOp.line,
        binOp.column
      )
    }
    return new AST.PrimitiveType("Bool", binOp.line, binOp.column)
  }

  private checkBooleanOperator(
    binOp: AST.BinaryOperation,
    leftType: AST.Type,
    rightType: AST.Type
  ): AST.Type {
    // Boolean operations
    if (!this.isBoolType(leftType) || !this.isBoolType(rightType)) {
      this.addError(
        `Type error in boolean operation '${binOp.operator}': both operands must be Bool`,
        binOp.line,
        binOp.column
      )
    }
    return new AST.PrimitiveType("Bool", binOp.line, binOp.column)
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

  private checkUnaryOperation(
    unaryOp: AST.UnaryOperation,
    env: TypeEnvironment
  ): AST.Type {
    const operandType = this.checkExpression(unaryOp.operand, env)

    switch (unaryOp.operator) {
      case "-":
        // Unary minus: only works on numeric types
        if (this.isNumericType(operandType)) {
          return operandType // Return same type (Int -> Int, Float -> Float)
        }
        this.addError(
          `Invalid operand for unary '-' operator`,
          unaryOp.line,
          unaryOp.column,
          `Cannot apply unary '-' to type '${this.typeToString(operandType)}'`,
          `Unary '-' can only be applied to Int or Float types`
        )
        return new AST.PrimitiveType("Int", unaryOp.line, unaryOp.column) // fallback

      case "!":
        // Logical negation: only works on Bool
        if (this.isBoolType(operandType)) {
          return operandType // Bool -> Bool
        }
        this.addError(
          `Invalid operand for logical negation '!' operator`,
          unaryOp.line,
          unaryOp.column,
          `Cannot apply '!' to type '${this.typeToString(operandType)}'`,
          `Logical negation '!' can only be applied to Bool type`
        )
        return new AST.PrimitiveType("Bool", unaryOp.line, unaryOp.column) // fallback

      default:
        this.addError(
          `Unknown unary operator: ${unaryOp.operator}`,
          unaryOp.line,
          unaryOp.column
        )
        return new AST.PrimitiveType("Int", unaryOp.line, unaryOp.column) // fallback
    }
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
          // print and putStrLn accept any type, so we just check the expression
          // without enforcing a specific type on it.
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
          // toString also accepts any type.
          this.checkExpression(call.arguments[0], env)
        }
        return new AST.PrimitiveType("String", call.line, call.column)

      case "show":
        if (call.arguments.length !== 1) {
          this.addError(
            `show expects exactly 1 argument, got ${call.arguments.length}`,
            call.line,
            call.column
          )
        } else {
          // show also accepts any type.
          this.checkExpression(call.arguments[0], env)
        }
        return new AST.PrimitiveType("Unit", call.line, call.column)

      case "toInt":
        if (call.arguments.length !== 1) {
          this.addError(
            `toInt expects exactly 1 argument, got ${call.arguments.length}`,
            call.line,
            call.column
          )
        } else {
          // toInt accepts any type and converts to Int
          this.checkExpression(call.arguments[0], env)
        }
        return new AST.PrimitiveType("Int", call.line, call.column)

      case "toFloat":
        if (call.arguments.length !== 1) {
          this.addError(
            `toFloat expects exactly 1 argument, got ${call.arguments.length}`,
            call.line,
            call.column
          )
        } else {
          // toFloat accepts any type and converts to Float
          this.checkExpression(call.arguments[0], env)
        }
        return new AST.PrimitiveType("Float", call.line, call.column)

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
      case "IdentifierPattern": {
        const idPat = pattern as AST.IdentifierPattern
        newEnv.define(idPat.name, matchedType)
        break
      }

      case "LiteralPattern": {
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
      }

      case "ConstructorPattern": {
        // Handle constructor patterns (simplified for now)
        const _ctorPat = pattern as AST.ConstructorPattern
        // TODO: Full implementation would check constructor types
        break
      }
    }

    return newEnv
  }

  private checkConstructor(
    ctor: AST.ConstructorExpression,
    env: TypeEnvironment
  ): AST.Type {
    // Handle Maybe and Either constructors
    switch (ctor.constructorName) {
      case "Just": {
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
      }

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
      case "Right": {
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
      }

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

  private checkLambda(
    lambda: AST.LambdaExpression,
    env: TypeEnvironment
  ): AST.Type {
    // Create a new environment for the lambda body
    const lambdaEnv = env.extend()

    // Add parameters to the lambda environment
    for (const param of lambda.parameters) {
      let paramType = param.type

      // If parameter type is placeholder "_", we'll use type inference
      if (
        paramType.kind === "PrimitiveType" &&
        (paramType as AST.PrimitiveType).name === "_"
      ) {
        // Create a type variable for inference
        paramType = new AST.PrimitiveType("a", param.line, param.column)
      }

      lambdaEnv.define(param.name, paramType)
    }

    // Check the lambda body
    const bodyType = this.checkExpression(lambda.body, lambdaEnv)

    // Build the function type from right to left (currying)
    let resultType: AST.Type = bodyType
    for (let i = lambda.parameters.length - 1; i >= 0; i--) {
      let paramType = lambda.parameters[i].type

      // Handle placeholder types for inference
      if (
        paramType.kind === "PrimitiveType" &&
        (paramType as AST.PrimitiveType).name === "_"
      ) {
        paramType = new AST.PrimitiveType(
          "a",
          lambda.parameters[i].line,
          lambda.parameters[i].column
        )
      }

      resultType = new AST.FunctionType(
        paramType,
        resultType,
        lambda.line,
        lambda.column
      )
    }

    return resultType
  }

  // Helper methods
  private typesEqual(t1: AST.Type, t2: AST.Type): boolean {
    if (t1.kind !== t2.kind) return false

    switch (t1.kind) {
      case "PrimitiveType": {
        const p1 = t1 as AST.PrimitiveType
        const p2 = t2 as AST.PrimitiveType
        // Type variables (single lowercase letters) are compatible with any type
        if (this.isTypeVariable(p1.name) || this.isTypeVariable(p2.name)) {
          return true
        }
        return p1.name === p2.name
      }

      case "FunctionType": {
        const f1 = t1 as AST.FunctionType
        const f2 = t2 as AST.FunctionType
        return (
          this.typesEqual(f1.paramType, f2.paramType) &&
          this.typesEqual(f1.returnType, f2.returnType)
        )
      }

      case "GenericType": {
        const g1 = t1 as AST.GenericType
        const g2 = t2 as AST.GenericType
        if (g1.name !== g2.name) return false
        if (g1.typeArguments.length !== g2.typeArguments.length) return false
        return g1.typeArguments.every((arg, i) =>
          this.typesEqual(arg, g2.typeArguments[i])
        )
      }

      case "StructType": {
        const s1 = t1 as AST.StructType
        const s2 = t2 as AST.StructType
        return s1.name === s2.name
      }

      case "RecordType": {
        const r1 = t1 as AST.RecordType
        const r2 = t2 as AST.RecordType
        if (r1.fields.length !== r2.fields.length) return false
        return r1.fields.every(
          (field, i) =>
            field.name === r2.fields[i].name &&
            this.typesEqual(field.type, r2.fields[i].type)
        )
      }

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
    this.errors.push(
      new SeseragiTypeError(message, line, column, code, suggestion)
    )
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

  private checkRecordExpression(
    record: AST.RecordExpression,
    env: TypeEnvironment
  ): AST.Type {
    const fields: AST.RecordField[] = []
    const fieldNames = new Set<string>()

    for (const field of record.fields) {
      this.processRecordField(field, fields, fieldNames, env)
    }

    return new AST.RecordType(fields, record.line, record.column)
  }

  private processRecordField(
    field:
      | AST.RecordInitField
      | AST.RecordShorthandField
      | AST.RecordSpreadField,
    fields: AST.RecordField[],
    fieldNames: Set<string>,
    env: TypeEnvironment
  ): void {
    if (field.kind === "RecordInitField") {
      this.processRecordInitField(
        field as AST.RecordInitField,
        fields,
        fieldNames,
        env
      )
    } else if (field.kind === "RecordSpreadField") {
      this.processRecordSpreadField(
        field as AST.RecordSpreadField,
        fields,
        fieldNames,
        env
      )
    }
  }

  private processRecordInitField(
    initField: AST.RecordInitField,
    fields: AST.RecordField[],
    fieldNames: Set<string>,
    env: TypeEnvironment
  ): void {
    // Check for duplicate field names
    if (fieldNames.has(initField.name)) {
      this.addError(
        `Duplicate field name '${initField.name}' in record`,
        initField.line,
        initField.column,
        `Field '${initField.name}' is defined multiple times`,
        `Remove the duplicate field definition`
      )
    }
    fieldNames.add(initField.name)

    const fieldType = this.checkExpression(initField.value, env)
    fields.push(
      new AST.RecordField(
        initField.name,
        fieldType,
        initField.line,
        initField.column
      )
    )
  }

  private processRecordSpreadField(
    spreadField: AST.RecordSpreadField,
    fields: AST.RecordField[],
    fieldNames: Set<string>,
    env: TypeEnvironment
  ): void {
    const spreadType = this.checkExpression(
      spreadField.spreadExpression.expression,
      env
    )

    // Handle spread fields - add their types to the current record
    if (spreadType.kind === "RecordType") {
      const recordType = spreadType as AST.RecordType
      for (const sourceField of recordType.fields) {
        if (!fieldNames.has(sourceField.name)) {
          fields.push(sourceField)
          fieldNames.add(sourceField.name)
        }
      }
    }
  }

  private checkRecordAccess(
    access: AST.RecordAccess,
    env: TypeEnvironment
  ): AST.Type {
    const recordType = this.checkExpression(access.record, env)

    // Support both RecordType and StructType for field access
    if (recordType.kind === "RecordType") {
      const rt = recordType as AST.RecordType
      const field = rt.fields.find((f) => f.name === access.fieldName)

      if (!field) {
        const availableFields = rt.fields.map((f) => f.name).join(", ")
        this.addError(
          `Record does not have field '${access.fieldName}'`,
          access.line,
          access.column,
          `Field '${access.fieldName}' is not defined in this record type`,
          `Available fields: ${availableFields}`
        )
        return new AST.PrimitiveType("Unknown", access.line, access.column)
      }

      return field.type
    } else if (recordType.kind === "StructType") {
      const st = recordType as AST.StructType
      const field = st.fields.find((f) => f.name === access.fieldName)

      if (!field) {
        const availableFields = st.fields.map((f) => f.name).join(", ")
        this.addError(
          `Struct does not have field '${access.fieldName}'`,
          access.line,
          access.column,
          `Field '${access.fieldName}' is not defined in this struct type`,
          `Available fields: ${availableFields}`
        )
        return new AST.PrimitiveType("Unknown", access.line, access.column)
      }

      return field.type
    } else {
      this.addError(
        `Cannot access field '${access.fieldName}' on non-record type '${this.typeToString(recordType)}'`,
        access.line,
        access.column,
        `Type '${this.typeToString(recordType)}' is not a record or struct`,
        `Only record and struct types support field access with dot notation`
      )
      return new AST.PrimitiveType("Unknown", access.line, access.column)
    }
  }

  private checkStructExpression(
    structExpr: AST.StructExpression,
    env: TypeEnvironment
  ): AST.Type {
    const structType = this.validateStructType(structExpr, env)
    if (!structType) {
      return new AST.PrimitiveType(
        "Unknown",
        structExpr.line,
        structExpr.column
      )
    }

    const st = structType as AST.StructType
    this.validateRequiredFields(structExpr, st)
    this.validateProvidedFields(structExpr, st, env)

    return structType
  }

  private validateStructType(
    structExpr: AST.StructExpression,
    env: TypeEnvironment
  ): AST.Type | null {
    const structType = env.lookup(structExpr.structName)

    if (!structType) {
      this.addError(
        `Unknown struct type: ${structExpr.structName}`,
        structExpr.line,
        structExpr.column,
        `Struct '${structExpr.structName}' is not defined`,
        `Check that the struct is declared before use`
      )
      return null
    }

    if (structType.kind !== "StructType") {
      this.addError(
        `'${structExpr.structName}' is not a struct type`,
        structExpr.line,
        structExpr.column,
        `Expected a struct type, but got ${structType.kind}`,
        `Only struct types can be used in struct expressions`
      )
      return null
    }

    return structType
  }

  private validateRequiredFields(
    structExpr: AST.StructExpression,
    st: AST.StructType
  ): void {
    // Check that all required fields are provided
    for (const structField of st.fields) {
      const providedField = structExpr.fields.find(
        (f) =>
          f.kind === "RecordInitField" &&
          (f as AST.RecordInitField).name === structField.name
      )
      if (!providedField) {
        this.addError(
          `Missing field '${structField.name}' in struct expression`,
          structExpr.line,
          structExpr.column,
          `Struct '${structExpr.structName}' requires field '${structField.name}'`,
          `Add the missing field: ${structField.name}: <value>`
        )
      }
    }
  }

  private validateProvidedFields(
    structExpr: AST.StructExpression,
    st: AST.StructType,
    env: TypeEnvironment
  ): void {
    // Check that all provided fields exist and have correct types
    for (const field of structExpr.fields) {
      if (field.kind === "RecordInitField") {
        this.validateStructInitField(
          field as AST.RecordInitField,
          structExpr,
          st,
          env
        )
      } else if (field.kind === "RecordSpreadField") {
        this.validateStructSpreadField(field as AST.RecordSpreadField, env)
      }
    }
  }

  private validateStructInitField(
    providedField: AST.RecordInitField,
    structExpr: AST.StructExpression,
    st: AST.StructType,
    env: TypeEnvironment
  ): void {
    const structField = st.fields.find((f) => f.name === providedField.name)
    if (!structField) {
      this.addError(
        `Unknown field '${providedField.name}' in struct '${structExpr.structName}'`,
        providedField.line,
        providedField.column,
        `Field '${providedField.name}' is not defined in struct '${structExpr.structName}'`,
        `Available fields: ${st.fields.map((f) => f.name).join(", ")}`
      )
      return
    }

    // Check field type
    const valueType = this.checkExpression(providedField.value, env)
    if (!this.typesEqual(valueType, structField.type)) {
      this.addError(
        `Field '${providedField.name}' type mismatch`,
        providedField.line,
        providedField.column,
        `Expected '${this.typeToString(structField.type)}', but got '${this.typeToString(valueType)}'`,
        `Ensure the field value matches the declared type`
      )
    }
  }

  private validateStructSpreadField(
    spreadField: AST.RecordSpreadField,
    env: TypeEnvironment
  ): void {
    // Handle spread fields
    const spreadType = this.checkExpression(
      spreadField.spreadExpression.expression,
      env
    )

    if (spreadType.kind !== "StructType" && spreadType.kind !== "RecordType") {
      this.addError(
        `Cannot spread non-struct/record type in struct literal`,
        spreadField.line,
        spreadField.column,
        `Expected struct or record type for spread operator`,
        `Use a struct or record expression instead`
      )
    }
  }

  private checkStructDeclaration(
    structDecl: AST.StructDeclaration,
    env: TypeEnvironment
  ): void {
    // Create StructType and add it to the environment
    const structType = new AST.StructType(
      structDecl.name,
      structDecl.fields,
      structDecl.line,
      structDecl.column
    )

    env.define(structDecl.name, structType)
  }

  private typeToString(type: AST.Type): string {
    switch (type.kind) {
      case "PrimitiveType":
        return (type as AST.PrimitiveType).name
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
      case "StructType": {
        const st = type as AST.StructType
        return st.name
      }
      default:
        return "Unknown"
    }
  }

  private checkTemplateExpression(
    expr: AST.TemplateExpression,
    env: TypeEnvironment
  ): AST.Type {
    // テンプレートリテラルの各埋め込み式の型をチェック
    for (const part of expr.parts) {
      if (typeof part !== "string") {
        // 埋め込み式の型をチェック
        this.checkExpression(part, env)
        // 現在は全ての型をtoString可能と仮定
        // 将来的にはtoString制約を追加できる
      }
    }

    // テンプレートリテラルの結果型は常にString
    return new AST.PrimitiveType("String", expr.line, expr.column)
  }

  private checkImportDeclaration(
    importDecl: AST.ImportDeclaration,
    env: TypeEnvironment
  ): void {
    // モジュールを解決
    const resolvedModule = this.moduleResolver.resolve(
      importDecl.module,
      this.currentFilePath
    )

    if (!resolvedModule) {
      this.addError(
        `Cannot resolve module '${importDecl.module}'`,
        importDecl.line,
        importDecl.column,
        "MODULE_NOT_FOUND",
        `Check that the file '${importDecl.module}.ssrg' exists`
      )
      return
    }

    // インポートした項目を環境に追加
    for (const item of importDecl.items) {
      const exportedFunction = resolvedModule.exports.functions.get(item.name)
      const exportedType = resolvedModule.exports.types.get(item.name)

      if (exportedFunction) {
        // 関数をインポート
        const funcType = this.createFunctionType(exportedFunction)
        const importName = item.alias || item.name
        env.define(importName, funcType)
      } else if (exportedType) {
        // 型をインポート
        const importName = item.alias || item.name
        env.define(importName, exportedType as AST.Type)
      } else {
        // エクスポートされていないアイテム
        this.addError(
          `Module '${importDecl.module}' does not export '${item.name}'`,
          importDecl.line,
          importDecl.column,
          "EXPORT_NOT_FOUND",
          `Check available exports in '${importDecl.module}.ssrg'`
        )
      }
    }
  }

  private createFunctionType(funcDecl: AST.FunctionDeclaration): AST.Type {
    // 関数の型を作成（パラメータ → 戻り値）
    let resultType = funcDecl.returnType

    // カリー化された関数型を構築（右結合）
    for (let i = funcDecl.parameters.length - 1; i >= 0; i--) {
      const param = funcDecl.parameters[i]
      resultType = new AST.FunctionType(
        param.type,
        resultType,
        funcDecl.line,
        funcDecl.column
      )
    }

    return resultType
  }
}
