/**
 * 使用機能分析器
 * ASTを解析してSeseragiプログラムで使用されている機能を検出
 */

import * as AST from "./ast"

export interface UsageAnalysis {
  needsCurrying: boolean
  needsPipeline: boolean
  needsReversePipe: boolean
  needsFunctionApplication: boolean
  needsMaybe: boolean
  needsEither: boolean
  needsList: boolean
  needsFunctorMap: boolean
  needsApplicativeApply: boolean
  needsMonadBind: boolean
  needsFoldMonoid: boolean
  needsBuiltins: {
    print: boolean
    putStrLn: boolean
    toString: boolean
    toInt: boolean
    toFloat: boolean
    show: boolean
    arrayToList: boolean
    listToArray: boolean
    head: boolean
    tail: boolean
  }
}

export class UsageAnalyzer {
  private analysis: UsageAnalysis = {
    needsCurrying: false,
    needsPipeline: false,
    needsReversePipe: false,
    needsFunctionApplication: false,
    needsMaybe: false,
    needsEither: false,
    needsList: false,
    needsFunctorMap: false,
    needsApplicativeApply: false,
    needsMonadBind: false,
    needsFoldMonoid: false,
    needsBuiltins: {
      print: false,
      putStrLn: false,
      toString: false,
      toInt: false,
      toFloat: false,
      show: false,
      arrayToList: false,
      listToArray: false,
      head: false,
      tail: false,
    },
  }

  analyze(statements: AST.Statement[]): UsageAnalysis {
    if (!statements || !Array.isArray(statements)) {
      return this.analysis
    }
    for (const stmt of statements) {
      this.analyzeStatement(stmt)
    }
    return this.analysis
  }

  private analyzeStatement(stmt: AST.Statement): void {
    if (stmt instanceof AST.FunctionDeclaration) {
      // 複数パラメータの関数はカリー化が必要
      if (stmt.parameters.length > 1) {
        this.analysis.needsCurrying = true
      }
      this.analyzeExpression(stmt.body)
    } else if (stmt instanceof AST.VariableDeclaration) {
      this.analyzeExpression(stmt.initializer)
    } else if (stmt instanceof AST.ExpressionStatement) {
      this.analyzeExpression(stmt.expression)
    } else if (stmt instanceof AST.TypeDeclaration) {
      // 型定義から型の使用状況を分析
      this.analyzeTypeDeclaration(stmt)
    }
  }

  private analyzeExpression(expr: AST.Expression): void {
    // Monadic and pipeline operations
    if (this.analyzeMonadicOperations(expr)) return

    // Constructor and builtin expressions
    if (this.analyzeConstructorAndBuiltins(expr)) return

    // Function applications and calls
    if (this.analyzeFunctionCalls(expr)) return

    // Conditionals and pattern matching
    if (this.analyzeConditionals(expr)) return

    // Data structures
    if (this.analyzeDataStructures(expr)) return

    // Other expressions
    this.analyzeOtherExpressions(expr)
  }

  private analyzeMonadicOperations(expr: AST.Expression): boolean {
    if (expr instanceof AST.Pipeline) {
      this.analysis.needsPipeline = true
      this.analyzeExpression(expr.left)
      this.analyzeExpression(expr.right)
      return true
    }
    if (expr instanceof AST.ReversePipe) {
      this.analysis.needsReversePipe = true
      this.analyzeExpression(expr.left)
      this.analyzeExpression(expr.right)
      return true
    }
    if (expr instanceof AST.MonadBind) {
      this.analysis.needsMonadBind = true
      this.analyzeExpression(expr.left)
      this.analyzeExpression(expr.right)
      return true
    }
    if (expr instanceof AST.FunctorMap) {
      this.analysis.needsFunctorMap = true
      this.analyzeExpression(expr.left)
      this.analyzeExpression(expr.right)
      return true
    }
    if (expr instanceof AST.ApplicativeApply) {
      this.analysis.needsApplicativeApply = true
      this.analyzeExpression(expr.left)
      this.analyzeExpression(expr.right)
      return true
    }
    if (expr instanceof AST.FoldMonoid) {
      this.analysis.needsFoldMonoid = true
      this.analyzeExpression(expr.left)
      this.analyzeExpression(expr.right)
      return true
    }
    return false
  }

  private analyzeConstructorAndBuiltins(expr: AST.Expression): boolean {
    if (expr instanceof AST.ConstructorExpression) {
      this.analyzeConstructor(expr)
      return true
    }
    if (expr instanceof AST.BuiltinFunctionCall) {
      this.analyzeBuiltin(expr.functionName)
      return true
    }
    return false
  }

  private analyzeFunctionCalls(expr: AST.Expression): boolean {
    if (expr instanceof AST.FunctionApplicationOperator) {
      this.analysis.needsFunctionApplication = true
      if (expr.left instanceof AST.Identifier) {
        this.analyzeBuiltin(expr.left.name)
      }
      this.analyzeExpression(expr.left)
      this.analyzeExpression(expr.right)
      return true
    }
    if (expr instanceof AST.FunctionApplication) {
      if (expr.function instanceof AST.Identifier) {
        this.analyzeBuiltin(expr.function.name)
      }
      this.analyzeExpression(expr.function)
      this.analyzeExpression(expr.argument)
      return true
    }
    if (expr instanceof AST.FunctionCall) {
      if (expr.function instanceof AST.Identifier) {
        this.analyzeBuiltin(expr.function.name)
      }
      this.analyzeExpression(expr.function)
      for (const arg of expr.arguments) {
        this.analyzeExpression(arg)
      }
      return true
    }
    return false
  }

  private analyzeConditionals(expr: AST.Expression): boolean {
    if (expr instanceof AST.BinaryOperation) {
      this.analyzeExpression(expr.left)
      this.analyzeExpression(expr.right)
      return true
    }
    if (expr instanceof AST.ConditionalExpression) {
      this.analyzeExpression(expr.condition)
      this.analyzeExpression(expr.thenExpression)
      this.analyzeExpression(expr.elseExpression)
      return true
    }
    if (expr instanceof AST.MatchExpression) {
      this.analyzeExpression(expr.expression)
      for (const matchCase of expr.cases) {
        this.analyzePattern(matchCase.pattern)
        this.analyzeExpression(matchCase.expression)
      }
      return true
    }
    return false
  }

  private analyzeDataStructures(expr: AST.Expression): boolean {
    if (expr instanceof AST.ListSugar) {
      this.analysis.needsList = true
      for (const element of expr.elements) {
        this.analyzeExpression(element)
      }
      return true
    }
    if (expr instanceof AST.ConsExpression) {
      this.analysis.needsList = true
      this.analyzeExpression(expr.left)
      this.analyzeExpression(expr.right)
      return true
    }
    if (expr instanceof AST.RecordExpression) {
      for (const field of expr.fields) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        this.analyzeExpression((field as any).value)
      }
      return true
    }
    if (expr instanceof AST.ArrayLiteral) {
      for (const element of expr.elements) {
        this.analyzeExpression(element)
      }
      return true
    }
    if (expr instanceof AST.ArrayAccess) {
      this.analyzeExpression(expr.array)
      this.analyzeExpression(expr.index)
      this.analysis.needsMaybe = true
      return true
    }
    return false
  }

  private analyzeOtherExpressions(expr: AST.Expression): void {
    if (expr instanceof AST.RecordAccess) {
      this.analyzeExpression(expr.record)
    } else if (expr instanceof AST.LambdaExpression) {
      this.analyzeExpression(expr.body)
    } else if (expr instanceof AST.BlockExpression) {
      for (const stmt of expr.statements) {
        this.analyzeStatement(stmt)
      }
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      if ((expr as any).expression) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        this.analyzeExpression((expr as any).expression)
      }
    } else if (expr instanceof AST.ListComprehensionSugar) {
      this.analysis.needsBuiltins.arrayToList = true
      this.analysis.needsCurrying = true
      this.analysis.needsList = true

      this.analyzeExpression(expr.expression)
      for (const generator of expr.generators) {
        this.analyzeExpression(generator.iterable)
      }
      for (const filter of expr.filters) {
        this.analyzeExpression(filter)
      }
    }
  }

  private analyzeConstructor(expr: AST.ConstructorExpression): void {
    const name = expr.constructorName

    if (name === "Just" || name === "Nothing") {
      this.analysis.needsMaybe = true
    } else if (name === "Left" || name === "Right") {
      this.analysis.needsEither = true
    } else if (name === "Empty" || name === "Cons") {
      this.analysis.needsList = true
    }

    // 引数の解析
    for (const arg of expr.arguments) {
      this.analyzeExpression(arg)
    }
  }

  private analyzePattern(pattern: AST.Pattern): void {
    if (pattern instanceof AST.ConstructorPattern) {
      const name = pattern.constructorName

      if (name === "Just" || name === "Nothing") {
        this.analysis.needsMaybe = true
      } else if (name === "Left" || name === "Right") {
        this.analysis.needsEither = true
      }

      for (const subPattern of pattern.patterns) {
        this.analyzePattern(subPattern)
      }
    }
  }

  private analyzeBuiltin(name: string): void {
    if (name === "print") {
      this.analysis.needsBuiltins.print = true
      // printは内部でtoStringを使うので自動的に必要
      this.analysis.needsBuiltins.toString = true
    } else if (name === "putStrLn") {
      this.analysis.needsBuiltins.putStrLn = true
    } else if (name === "toString") {
      this.analysis.needsBuiltins.toString = true
    } else if (name === "toInt") {
      this.analysis.needsBuiltins.toInt = true
    } else if (name === "toFloat") {
      this.analysis.needsBuiltins.toFloat = true
    } else if (name === "show") {
      this.analysis.needsBuiltins.show = true
      // showは内部でtoStringを使うので自動的に必要
      this.analysis.needsBuiltins.toString = true
    } else if (name === "arrayToList") {
      this.analysis.needsBuiltins.arrayToList = true
      this.analysis.needsList = true // List型も必要
      this.analysis.needsCurrying = true // カリー化も必要
    } else if (name === "listToArray") {
      this.analysis.needsBuiltins.listToArray = true
      this.analysis.needsList = true // List型も必要
    } else if (name === "head") {
      this.analysis.needsBuiltins.head = true
      this.analysis.needsList = true // List型が必要
      this.analysis.needsMaybe = true // Maybe型が必要（戻り値型）
    } else if (name === "tail") {
      this.analysis.needsBuiltins.tail = true
      this.analysis.needsList = true // List型が必要
      this.analysis.needsCurrying = true // カリー化も必要
    }
  }

  private analyzeTypeDeclaration(typeDecl: AST.TypeDeclaration): void {
    for (const field of typeDecl.fields) {
      this.analyzeType(field.type)
    }
  }

  private analyzeType(type: AST.Type): void {
    if (type instanceof AST.GenericType) {
      if (type.name === "Maybe") {
        this.analysis.needsMaybe = true
      } else if (type.name === "Either") {
        this.analysis.needsEither = true
      }

      for (const typeArg of type.typeArguments) {
        this.analyzeType(typeArg)
      }
    } else if (type instanceof AST.FunctionType) {
      this.analyzeType(type.paramType)
      this.analyzeType(type.returnType)
    }
  }
}
