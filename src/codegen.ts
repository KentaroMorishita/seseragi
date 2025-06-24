import {
  Statement,
  Expression,
  FunctionDeclaration,
  VariableDeclaration,
  TypeDeclaration,
  Literal,
  Identifier,
  BinaryOperation,
  FunctionCall,
  ConditionalExpression,
  MatchExpression,
  Pipeline,
  ReversePipe,
  MonadBind,
  FoldMonoid,
  Type,
  FunctionType,
  PrimitiveType,
  GenericType,
} from "./ast"

/**
 * Seseragi から TypeScript へのコード生成器
 * 関数型言語の機能をJavaScriptの慣用的なコードに変換
 */

export interface CodeGenOptions {
  indent?: string
  useArrowFunctions?: boolean
  generateComments?: boolean
}

const defaultOptions: CodeGenOptions = {
  indent: "  ",
  useArrowFunctions: true,
  generateComments: false,
}

export function generateTypeScript(
  statements: Statement[],
  options: CodeGenOptions = {}
): string {
  const opts = { ...defaultOptions, ...options }
  const generator = new CodeGenerator(opts)
  return generator.generateProgram(statements)
}

class CodeGenerator {
  options: CodeGenOptions
  indentLevel: number

  constructor(options: CodeGenOptions) {
    this.options = options
    this.indentLevel = 0
  }

  // プログラム全体の生成
  generateProgram(statements: Statement[]): string {
    const lines: string[] = []

    if (this.options.generateComments) {
      lines.push("// Generated TypeScript code from Seseragi")
      lines.push("")
    }

    // ヘルパー関数の追加
    lines.push(...this.generateHelperFunctions())
    lines.push("")

    for (const stmt of statements) {
      const code = this.generateStatement(stmt)
      if (code.trim()) {
        lines.push(code)
        lines.push("")
      }
    }

    return lines.join("\n")
  }

  // ヘルパー関数の生成
  generateHelperFunctions(): string[] {
    return [
      "// Seseragi runtime helpers",
      "",
      "// カリー化関数のヘルパー",
      "const curry = (fn: Function) => {",
      "  return function curried(...args: any[]) {",
      "    if (args.length >= fn.length) {",
      "      return fn.apply(this, args);",
      "    } else {",
      "      return function(...args2: any[]) {",
      "        return curried.apply(this, args.concat(args2));",
      "      };",
      "    }",
      "  };",
      "};",
      "",
      "// パイプライン演算子のヘルパー",
      "const pipe = <T, U>(value: T, fn: (arg: T) => U): U => fn(value);",
      "",
      "// 逆パイプ演算子のヘルパー",
      "const reversePipe = <T, U>(fn: (arg: T) => U, value: T): U => fn(value);",
      "",
      "// モナドバインドのヘルパー（Maybe用）",
      "const bind = <T, U>(maybe: T | null | undefined, fn: (value: T) => U | null | undefined): U | null | undefined => {",
      "  return maybe != null ? fn(maybe) : null;",
      "};",
      "",
      "// 畳み込みモノイドのヘルパー",
      "const foldMonoid = <T>(arr: T[], empty: T, combine: (a: T, b: T) => T): T => {",
      "  return arr.reduce(combine, empty);",
      "};",
    ]
  }

  // 文の生成
  generateStatement(stmt: Statement): string {
    if (stmt instanceof FunctionDeclaration) {
      return this.generateFunctionDeclaration(stmt)
    } else if (stmt instanceof VariableDeclaration) {
      return this.generateVariableDeclaration(stmt)
    } else if (stmt instanceof TypeDeclaration) {
      return this.generateTypeDeclaration(stmt)
    }

    return `// Unsupported statement: ${stmt.constructor.name}`
  }

  // 関数宣言の生成
  generateFunctionDeclaration(func: FunctionDeclaration): string {
    const indent = (this.options.indent || '  ').repeat(this.indentLevel)
    const params = func.parameters
      .map((p) => `${p.name}: ${this.generateType(p.type)}`)
      .join(", ")
    const returnType = func.returnType
      ? this.generateType(func.returnType)
      : "any"

    // カリー化された関数として生成
    if (func.parameters.length > 1) {
      const body = this.generateExpression(func.body)
      return `${indent}const ${func.name} = curry((${params}): ${returnType} => ${body});`
    } else {
      const body = this.generateExpression(func.body)
      if (this.options.useArrowFunctions) {
        return `${indent}const ${func.name} = (${params}): ${returnType} => ${body};`
      } else {
        return `${indent}function ${func.name}(${params}): ${returnType} {\n${indent}  return ${body};\n${indent}}`
      }
    }
  }

  // 変数宣言の生成
  generateVariableDeclaration(varDecl: VariableDeclaration): string {
    const indent = (this.options.indent || '  ').repeat(this.indentLevel)
    const type = varDecl.type ? `: ${this.generateType(varDecl.type)}` : ""
    const value = this.generateExpression(varDecl.initializer)

    return `${indent}const ${varDecl.name}${type} = ${value};`
  }

  // 型宣言の生成
  generateTypeDeclaration(typeDecl: TypeDeclaration): string {
    const indent = (this.options.indent || '  ').repeat(this.indentLevel)

    if (typeDecl.fields && typeDecl.fields.length > 0) {
      // 構造体型として生成
      const fields = typeDecl.fields
        .map((f) => `  ${f.name}: ${this.generateType(f.type)}`)
        .join(";\n")

      return `${indent}type ${typeDecl.name} = {\n${fields}\n};`
    } else {
      // 型エイリアスとして生成
      return `${indent}type ${typeDecl.name} = any; // TODO: implement type`
    }
  }

  // 式の生成
  generateExpression(expr: Expression): string {
    if (expr instanceof Literal) {
      return this.generateLiteral(expr)
    } else if (expr instanceof Identifier) {
      return expr.name
    } else if (expr instanceof BinaryOperation) {
      return this.generateBinaryOperation(expr)
    } else if (expr instanceof FunctionCall) {
      return this.generateFunctionCall(expr)
    } else if (expr instanceof ConditionalExpression) {
      return this.generateConditionalExpression(expr)
    } else if (expr instanceof MatchExpression) {
      return this.generateMatchExpression(expr)
    } else if (expr instanceof Pipeline) {
      return this.generatePipeline(expr)
    } else if (expr instanceof ReversePipe) {
      return this.generateReversePipe(expr)
    } else if (expr instanceof MonadBind) {
      return this.generateMonadBind(expr)
    } else if (expr instanceof FoldMonoid) {
      return this.generateFoldMonoid(expr)
    }

    return `/* Unsupported expression: ${expr.constructor.name} */`
  }

  // リテラルの生成
  generateLiteral(literal: Literal): string {
    switch (literal.literalType) {
      case "string":
        return `"${literal.value}"`
      case "integer":
      case "float":
        return literal.value.toString()
      case "boolean":
        return literal.value.toString()
      default:
        return literal.value.toString()
    }
  }

  // 二項演算の生成
  generateBinaryOperation(binOp: BinaryOperation): string {
    const left = this.generateExpression(binOp.left)
    const right = this.generateExpression(binOp.right)

    // 演算子の変換
    let operator = binOp.operator
    if (operator === "==") operator = "==="
    if (operator === "!=") operator = "!=="

    return `(${left} ${operator} ${right})`
  }

  // 関数呼び出しの生成
  generateFunctionCall(call: FunctionCall): string {
    const func = this.generateExpression(call.function)
    const args = call.arguments.map((arg) => this.generateExpression(arg))

    return `${func}(${args.join(", ")})`
  }

  // 条件式の生成
  generateConditionalExpression(cond: ConditionalExpression): string {
    const condition = this.generateExpression(cond.condition)
    const thenBranch = this.generateExpression(cond.thenExpression)
    const elseBranch = this.generateExpression(cond.elseExpression)

    return `(${condition} ? ${thenBranch} : ${elseBranch})`
  }

  // マッチ式の生成
  generateMatchExpression(match: MatchExpression): string {
    const expr = this.generateExpression(match.expression)

    // switch文として生成（簡略版）
    const cases = match.cases
      .map((c) => {
        const pattern = this.generatePattern(c.pattern)
        const body = this.generateExpression(c.expression)
        return `    case ${pattern}: return ${body};`
      })
      .join("\n")

    return `(() => {\n  switch (${expr}) {\n${cases}\n    default: throw new Error('Non-exhaustive pattern match');\n  }\n})()`
  }

  // パターンの生成
  generatePattern(pattern: any): string {
    // 簡易実装：リテラルパターンのみサポート
    if (pattern.value !== undefined) {
      return JSON.stringify(pattern.value)
    }
    return pattern.toString()
  }

  // パイプライン演算子の生成
  generatePipeline(pipeline: Pipeline): string {
    const left = this.generateExpression(pipeline.left)
    const right = this.generateExpression(pipeline.right)

    return `pipe(${left}, ${right})`
  }

  // 逆パイプ演算子の生成
  generateReversePipe(reversePipe: ReversePipe): string {
    const left = this.generateExpression(reversePipe.left)
    const right = this.generateExpression(reversePipe.right)

    return `reversePipe(${left}, ${right})`
  }

  // モナドバインドの生成
  generateMonadBind(bind: MonadBind): string {
    const left = this.generateExpression(bind.left)
    const right = this.generateExpression(bind.right)

    return `bind(${left}, ${right})`
  }

  // 畳み込みモノイドの生成
  generateFoldMonoid(fold: FoldMonoid): string {
    const left = this.generateExpression(fold.left)
    const right = this.generateExpression(fold.right)

    return `foldMonoid(${left}, /* empty */, ${right})`
  }

  // 型の生成
  generateType(type: Type | undefined): string {
    if (!type) return "any"

    if (type instanceof PrimitiveType) {
      switch (type.name) {
        case "Int":
          return "number"
        case "Float":
          return "number"
        case "Bool":
          return "boolean"
        case "String":
          return "string"
        case "Char":
          return "string"
        case "Unit":
          return "void"
        default:
          return type.name
      }
    } else if (type instanceof FunctionType) {
      const paramType = this.generateType(type.paramType)
      const returnType = this.generateType(type.returnType)
      return `(arg: ${paramType}) => ${returnType}`
    } else if (type instanceof GenericType) {
      if (type.typeArguments.length === 0) {
        return this.generateGenericTypeName(type.name)
      }
      const params = type.typeArguments
        .map((p) => this.generateType(p))
        .join(", ")
      return `${this.generateGenericTypeName(type.name)}<${params}>`
    }

    return "any"
  }

  // ジェネリック型名の変換
  generateGenericTypeName(name: string): string {
    switch (name) {
      case "Maybe":
        return "Maybe"
      case "Either":
        return "Either"
      case "IO":
        return "IO"
      case "List":
        return "Array"
      case "Array":
        return "Array"
      default:
        return name
    }
  }
}
