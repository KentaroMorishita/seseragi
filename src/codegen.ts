import {
  Statement,
  Expression,
  ExpressionStatement,
  FunctionDeclaration,
  VariableDeclaration,
  TypeDeclaration,
  Literal,
  Identifier,
  BinaryOperation,
  UnaryOperation,
  FunctionCall,
  FunctionApplication,
  BuiltinFunctionCall,
  ConditionalExpression,
  MatchExpression,
  ConstructorExpression,
  BlockExpression,
  Pipeline,
  ReversePipe,
  FunctorMap,
  ApplicativeApply,
  MonadBind,
  FoldMonoid,
  FunctionApplicationOperator,
  LambdaExpression,
  RecordExpression,
  RecordAccess,
  ArrayLiteral,
  ArrayAccess,
  ListSugar,
  ConsExpression,
  RangeLiteral,
  ListComprehension,
  ListComprehensionSugar,
  Generator,
  Type,
  FunctionType,
  PrimitiveType,
  GenericType,
  RecordType,
} from "./ast"
import { UsageAnalyzer, type UsageAnalysis } from "./usage-analyzer"

/**
 * Seseragi から TypeScript へのコード生成器
 * 関数型言語の機能をJavaScriptの慣用的なコードに変換
 */

export interface CodeGenOptions {
  indent?: string
  useArrowFunctions?: boolean
  generateComments?: boolean
  runtimeMode?: "embedded" | "import" | "minimal"
}

const defaultOptions: CodeGenOptions = {
  indent: "  ",
  useArrowFunctions: true,
  generateComments: false,
  runtimeMode: "import",
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
  usageAnalysis?: UsageAnalysis

  constructor(options: CodeGenOptions) {
    this.options = options
    this.indentLevel = 0
  }

  // プログラム全体の生成
  generateProgram(statements: Statement[]): string {
    const lines: string[] = []

    // 使用分析を実行
    const analyzer = new UsageAnalyzer()
    this.usageAnalysis = analyzer.analyze(statements)

    if (this.options.generateComments) {
      lines.push("// Generated TypeScript code from Seseragi")
      lines.push("")
    }

    // ランタイムの生成
    lines.push(...this.generateRuntime())
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

  // ランタイムの生成
  generateRuntime(): string[] {
    const lines: string[] = []

    if (!this.usageAnalysis) {
      return lines
    }

    switch (this.options.runtimeMode) {
      case "import":
        return this.generateRuntimeImports()
      case "minimal":
        return this.generateMinimalRuntime()
      case "embedded":
      default:
        return this.generateEmbeddedRuntime()
    }
  }

  // 外部ランタイムライブラリからのインポート
  generateRuntimeImports(): string[] {
    const lines: string[] = []
    const imports: string[] = []

    if (!this.usageAnalysis) return lines

    // 必要な機能のみインポート
    if (this.usageAnalysis.needsCurrying) {
      imports.push("curry")
    }
    if (this.usageAnalysis.needsPipeline) {
      imports.push("pipe")
    }
    if (this.usageAnalysis.needsReversePipe) {
      imports.push("reversePipe")
    }
    if (this.usageAnalysis.needsFunctionApplication) {
      imports.push("apply")
    }
    if (this.usageAnalysis.needsMaybe) {
      imports.push("Just", "Nothing", "type Maybe")
    }
    if (this.usageAnalysis.needsEither) {
      imports.push("Left", "Right", "type Either")
    }
    if (this.usageAnalysis.needsFunctorMap) {
      imports.push("map")
    }
    if (this.usageAnalysis.needsApplicativeApply) {
      imports.push("applyWrapped")
    }
    if (this.usageAnalysis.needsMonadBind) {
      imports.push("bind")
    }
    if (this.usageAnalysis.needsFoldMonoid) {
      imports.push("foldMonoid")
    }
    if (this.usageAnalysis.needsBuiltins.print) {
      imports.push("print")
    }
    if (this.usageAnalysis.needsBuiltins.putStrLn) {
      imports.push("putStrLn")
    }
    if (this.usageAnalysis.needsBuiltins.toString) {
      imports.push("toString")
    }
    if (this.usageAnalysis.needsBuiltins.show) {
      imports.push("show")
    }
    if (this.usageAnalysis.needsBuiltins.arrayToList) {
      imports.push("arrayToList")
    }
    if (this.usageAnalysis.needsBuiltins.listToArray) {
      imports.push("listToArray")
    }

    if (imports.length > 0) {
      lines.push(
        `import { ${imports.join(", ")} } from './runtime/seseragi-runtime.js';`
      )
    }

    return lines
  }

  // 最小限のランタイム（使用機能のみ埋め込み）
  generateMinimalRuntime(): string[] {
    const lines: string[] = ["// Seseragi minimal runtime", ""]

    if (!this.usageAnalysis) return lines

    // 型定義
    if (this.usageAnalysis.needsMaybe) {
      lines.push(
        "type Maybe<T> = { tag: 'Just'; value: T } | { tag: 'Nothing' };"
      )
    }
    if (this.usageAnalysis.needsEither) {
      lines.push(
        "type Either<L, R> = { tag: 'Left'; value: L } | { tag: 'Right'; value: R };"
      )
    }
    if (this.usageAnalysis.needsList) {
      lines.push("type List<T> = { tag: 'Empty' } | { tag: 'Cons'; head: T; tail: List<T> };")
    }
    if (this.usageAnalysis.needsMaybe || this.usageAnalysis.needsEither || this.usageAnalysis.needsList) {
      lines.push("")
    }

    // 必要な機能のみ生成
    if (this.usageAnalysis.needsCurrying) {
      lines.push(...this.generateCurryFunction())
      lines.push("")
    }
    if (this.usageAnalysis.needsPipeline) {
      lines.push(
        "const pipe = <T, U>(value: T, fn: (arg: T) => U): U => fn(value);"
      )
      lines.push("")
    }
    if (this.usageAnalysis.needsReversePipe) {
      lines.push(
        "const reversePipe = <T, U>(fn: (arg: T) => U, value: T): U => fn(value);"
      )
      lines.push("")
    }
    if (this.usageAnalysis.needsFunctionApplication) {
      lines.push(
        "const apply = <T, U>(fn: (arg: T) => U, value: T): U => fn(value);"
      )
      lines.push("")
    }
    if (this.usageAnalysis.needsMaybe) {
      lines.push(
        "const Just = <T>(value: T): Maybe<T> => ({ tag: 'Just', value });"
      )
      lines.push("const Nothing: Maybe<never> = { tag: 'Nothing' };")
      lines.push("")
    }
    if (this.usageAnalysis.needsEither) {
      lines.push(
        "const Left = <L>(value: L): Either<L, never> => ({ tag: 'Left', value });"
      )
      lines.push(
        "const Right = <R>(value: R): Either<never, R> => ({ tag: 'Right', value });"
      )
      lines.push("")
    }
    if (this.usageAnalysis.needsList) {
      lines.push("const Empty: List<never> = { tag: 'Empty' };")
      lines.push(
        "const Cons = <T>(head: T, tail: List<T>): List<T> => ({ tag: 'Cons', head, tail });"
      )
      lines.push("")
    }
    if (this.usageAnalysis.needsFunctorMap) {
      lines.push(
        "const map = <T, U>(fn: (value: T) => U, container: Maybe<T> | Either<any, T>): Maybe<U> | Either<any, U> => {"
      )
      lines.push("  if ('tag' in container) {")
      lines.push(
        "    if (container.tag === 'Just') return Just(fn(container.value));"
      )
      lines.push(
        "    if (container.tag === 'Right') return Right(fn(container.value));"
      )
      lines.push("    if (container.tag === 'Nothing') return Nothing;")
      lines.push("    if (container.tag === 'Left') return container;")
      lines.push("  }")
      lines.push("  return Nothing;")
      lines.push("};")
      lines.push("")
    }
    if (this.usageAnalysis.needsApplicativeApply) {
      lines.push(
        "const applyWrapped = <T, U>(wrapped: Maybe<(value: T) => U> | Either<any, (value: T) => U>, container: Maybe<T> | Either<any, T>): Maybe<U> | Either<any, U> => {"
      )
      lines.push("  // Maybe types")
      lines.push(
        "  if (wrapped.tag === 'Nothing' || container.tag === 'Nothing') return Nothing;"
      )
      lines.push(
        "  if (wrapped.tag === 'Just' && container.tag === 'Just') return Just(wrapped.value(container.value));"
      )
      lines.push("  // Either types")
      lines.push("  if (wrapped.tag === 'Left') return wrapped;")
      lines.push("  if (container.tag === 'Left') return container;")
      lines.push(
        "  if (wrapped.tag === 'Right' && container.tag === 'Right') return Right(wrapped.value(container.value));"
      )
      lines.push("  return Nothing;")
      lines.push("};")
      lines.push("")
    }
    if (this.usageAnalysis.needsMonadBind) {
      lines.push(
        "const bind = <T, U>(container: Maybe<T> | Either<any, T>, fn: (value: T) => Maybe<U> | Either<any, U>): Maybe<U> | Either<any, U> => {"
      )
      lines.push("  if (container.tag === 'Just') return fn(container.value);")
      lines.push("  if (container.tag === 'Right') return fn(container.value);")
      lines.push("  if (container.tag === 'Nothing') return Nothing;")
      lines.push("  if (container.tag === 'Left') return container;")
      lines.push("  return Nothing;")
      lines.push("};")
      lines.push("")
    }
    if (this.usageAnalysis.needsFoldMonoid) {
      lines.push(
        "const foldMonoid = <T>(arr: T[], empty: T, combine: (a: T, b: T) => T): T => {"
      )
      lines.push("  return arr.reduce(combine, empty);")
      lines.push("};")
      lines.push("")
    }
    if (this.usageAnalysis.needsBuiltins.print) {
      lines.push(`const print = (value: any): void => {
  // Seseragi型の場合は美しく整形
  if (value && typeof value === 'object' && (
    value.tag === 'Just' || value.tag === 'Nothing' ||
    value.tag === 'Left' || value.tag === 'Right' ||
    value.tag === 'Cons' || value.tag === 'Empty'
  )) {
    console.log(toString(value))
  } 
  // 通常のオブジェクトはそのまま
  else {
    console.log(value)
  }
};`)
    }
    if (this.usageAnalysis.needsBuiltins.putStrLn) {
      lines.push(
        "const putStrLn = (value: string): void => console.log(value);"
      )
    }
    if (this.usageAnalysis.needsBuiltins.toString) {
      lines.push(`const toString = (value: any): string => {
  // Maybe型の美しい表示
  if (value && typeof value === 'object' && value.tag === 'Just') {
    return \`Just(\${toString(value.value)})\`
  }
  if (value && typeof value === 'object' && value.tag === 'Nothing') {
    return 'Nothing'
  }
  
  // Either型の美しい表示
  if (value && typeof value === 'object' && value.tag === 'Left') {
    return \`Left(\${toString(value.value)})\`
  }
  if (value && typeof value === 'object' && value.tag === 'Right') {
    return \`Right(\${toString(value.value)})\`
  }
  
  // List型の美しい表示
  if (value && typeof value === 'object' && value.tag === 'Empty') {
    return '[]'
  }
  if (value && typeof value === 'object' && value.tag === 'Cons') {
    const items = []
    let current = value
    while (current.tag === 'Cons') {
      items.push(toString(current.head))
      current = current.tail
    }
    return \`[\${items.join(', ')}]\`
  }
  
  // 配列の表示
  if (Array.isArray(value)) {
    return \`[\${value.map(toString).join(', ')}]\`
  }
  
  // プリミティブ型
  if (typeof value === 'string') {
    return \`"\${value}"\`
  }
  if (typeof value === 'number' || typeof value === 'boolean') {
    return String(value)
  }
  
  return String(value)
};`)
    }
    if (this.usageAnalysis.needsBuiltins.show) {
      // prettyFormat関数も必要
      lines.push(`// Seseragi型の構造を正規化
function normalizeStructure(obj) {
  if (!obj || typeof obj !== 'object') return obj
  
  // List型 → 配列に変換
  if (obj.tag === 'Empty') return []
  if (obj.tag === 'Cons') {
    const items = []
    let current = obj
    while (current && current.tag === 'Cons') {
      items.push(normalizeStructure(current.head))
      current = current.tail
    }
    return items
  }
  
  // Maybe型
  if (obj.tag === 'Just') {
    return { '@@type': 'Just', value: normalizeStructure(obj.value) }
  }
  if (obj.tag === 'Nothing') {
    return '@@Nothing'
  }
  
  // Either型
  if (obj.tag === 'Right') {
    return { '@@type': 'Right', value: normalizeStructure(obj.value) }
  }
  if (obj.tag === 'Left') {
    return { '@@type': 'Left', value: normalizeStructure(obj.value) }
  }
  
  // 配列
  if (Array.isArray(obj)) {
    return obj.map(normalizeStructure)
  }
  
  // 通常のオブジェクト
  const result = {}
  for (const key in obj) {
    if (obj.hasOwnProperty(key)) {
      result[key] = normalizeStructure(obj[key])
    }
  }
  return result
}

// JSON文字列をSeseragi型の美しい表記に変換
function beautifySeseragiTypes(json) {
  return json
    // Just型
    .replace(/"@@type":\\s*"Just",\\s*"value":\\s*([^}]+)/g, (_, val) => \`Just(\${val.trim()})\`)
    .replace(/\\{\\s*Just\\(([^)]+)\\)\\s*\\}/g, 'Just($1)')
    // Nothing
    .replace(/"@@Nothing"/g, 'Nothing')
    // Right型
    .replace(/"@@type":\\s*"Right",\\s*"value":\\s*([^}]+)/g, (_, val) => \`Right(\${val.trim()})\`)
    .replace(/\\{\\s*Right\\(([^)]+)\\)\\s*\\}/g, 'Right($1)')
    // Left型
    .replace(/"@@type":\\s*"Left",\\s*"value":\\s*([^}]+)/g, (_, val) => \`Left(\${val.trim()})\`)
    .replace(/\\{\\s*Left\\(([^)]+)\\)\\s*\\}/g, 'Left($1)')
}

// 美しくフォーマットする関数
const prettyFormat = (value) => {
  // プリミティブ型
  if (typeof value === 'string') return \`"\${value}"\`
  if (typeof value === 'number' || typeof value === 'boolean') return String(value)
  if (value === null) return 'null'
  if (value === undefined) return 'undefined'
  
  // オブジェクトの場合
  if (typeof value === 'object') {
    // まず構造を正規化
    const normalized = normalizeStructure(value)
    // JSON.stringifyで整形
    const json = JSON.stringify(normalized, null, 2)
    // Seseragi型の表記に変換
    return beautifySeseragiTypes(json)
  }
  
  return String(value)
}

const show = (value) => {
  console.log(prettyFormat(value))
};`)
    }
    if (this.usageAnalysis.needsBuiltins.arrayToList || this.usageAnalysis.needsBuiltins.listToArray) {
      lines.push("")
      if (this.usageAnalysis.needsBuiltins.arrayToList) {
        lines.push("const arrayToList = curry(<T>(arr: T[]): List<T> => {")
        lines.push("  let result: List<T> = Empty;")
        lines.push("  for (let i = arr.length - 1; i >= 0; i--) {")
        lines.push("    result = Cons(arr[i], result);")
        lines.push("  }")
        lines.push("  return result;")
        lines.push("});")
        lines.push("")
      }
      if (this.usageAnalysis.needsBuiltins.listToArray) {
        lines.push("const listToArray = curry(<T>(list: List<T>): T[] => {")
        lines.push("  const result: T[] = [];")
        lines.push("  let current = list;")
        lines.push("  while (current.tag === 'Cons') {")
        lines.push("    result.push(current.head);")
        lines.push("    current = current.tail;")
        lines.push("  }")
        lines.push("  return result;")
        lines.push("});")
      }
    }

    return lines
  }

  // 従来の埋め込み式ランタイム（下位互換性用）
  generateEmbeddedRuntime(): string[] {
    return [
      "// Seseragi runtime helpers",
      "",
      "type Maybe<T> = { tag: 'Just'; value: T } | { tag: 'Nothing' };",
      "type Either<L, R> = { tag: 'Left'; value: L } | { tag: 'Right'; value: R };",
      "type List<T> = { tag: 'Empty' } | { tag: 'Cons'; head: T; tail: List<T> };",
      "",
      ...this.generateCurryFunction(),
      "",
      "const pipe = <T, U>(value: T, fn: (arg: T) => U): U => fn(value);",
      "",
      "const reversePipe = <T, U>(fn: (arg: T) => U, value: T): U => fn(value);",
      "",
      "const map = <T, U>(fn: (value: T) => U, container: Maybe<T> | Either<any, T>): Maybe<U> | Either<any, U> => {",
      "  if ('tag' in container) {",
      "    if (container.tag === 'Just') return Just(fn(container.value));",
      "    if (container.tag === 'Right') return Right(fn(container.value));",
      "    if (container.tag === 'Nothing') return Nothing;",
      "    if (container.tag === 'Left') return container;",
      "  }",
      "  return Nothing;",
      "};",
      "",
      "const applyWrapped = <T, U>(wrapped: Maybe<(value: T) => U> | Either<any, (value: T) => U>, container: Maybe<T> | Either<any, T>): Maybe<U> | Either<any, U> => {",
      "  // Maybe types",
      "  if (wrapped.tag === 'Nothing' || container.tag === 'Nothing') return Nothing;",
      "  if (wrapped.tag === 'Just' && container.tag === 'Just') return Just(wrapped.value(container.value));",
      "  // Either types",
      "  if (wrapped.tag === 'Left') return wrapped;",
      "  if (container.tag === 'Left') return container;",
      "  if (wrapped.tag === 'Right' && container.tag === 'Right') return Right(wrapped.value(container.value));",
      "  return Nothing;",
      "};",
      "",
      "const bind = <T, U>(container: Maybe<T> | Either<any, T>, fn: (value: T) => Maybe<U> | Either<any, U>): Maybe<U> | Either<any, U> => {",
      "  if (container.tag === 'Just') return fn(container.value);",
      "  if (container.tag === 'Right') return fn(container.value);",
      "  if (container.tag === 'Nothing') return Nothing;",
      "  if (container.tag === 'Left') return container;",
      "  return Nothing;",
      "};",
      "",
      "const foldMonoid = <T>(arr: T[], empty: T, combine: (a: T, b: T) => T): T => {",
      "  return arr.reduce(combine, empty);",
      "};",
      "",
      "const Just = <T>(value: T): Maybe<T> => ({ tag: 'Just', value });",
      "const Nothing: Maybe<never> = { tag: 'Nothing' };",
      "",
      "const Left = <L>(value: L): Either<L, never> => ({ tag: 'Left', value });",
      "const Right = <R>(value: R): Either<never, R> => ({ tag: 'Right', value });",
      "",
      "const Empty: List<never> = { tag: 'Empty' };",
      "const Cons = <T>(head: T, tail: List<T>): List<T> => ({ tag: 'Cons', head, tail });",
      "",
      `const print = (value: any): void => {
  // Seseragi型の場合は美しく整形
  if (value && typeof value === 'object' && (
    value.tag === 'Just' || value.tag === 'Nothing' ||
    value.tag === 'Left' || value.tag === 'Right' ||
    value.tag === 'Cons' || value.tag === 'Empty'
  )) {
    console.log(toString(value))
  } 
  // 通常のオブジェクトはそのまま
  else {
    console.log(value)
  }
};`,
      "const putStrLn = (value: string): void => console.log(value);",
      `const toString = (value: any): string => {
  // Maybe型の美しい表示
  if (value && typeof value === 'object' && value.tag === 'Just') {
    return \`Just(\${toString(value.value)})\`
  }
  if (value && typeof value === 'object' && value.tag === 'Nothing') {
    return 'Nothing'
  }
  
  // Either型の美しい表示
  if (value && typeof value === 'object' && value.tag === 'Left') {
    return \`Left(\${toString(value.value)})\`
  }
  if (value && typeof value === 'object' && value.tag === 'Right') {
    return \`Right(\${toString(value.value)})\`
  }
  
  // List型の美しい表示
  if (value && typeof value === 'object' && value.tag === 'Empty') {
    return '[]'
  }
  if (value && typeof value === 'object' && value.tag === 'Cons') {
    const items = []
    let current = value
    while (current.tag === 'Cons') {
      items.push(toString(current.head))
      current = current.tail
    }
    return \`[\${items.join(', ')}]\`
  }
  
  // 配列の表示
  if (Array.isArray(value)) {
    return \`[\${value.map(toString).join(', ')}]\`
  }
  
  // プリミティブ型
  if (typeof value === 'string') {
    return \`"\${value}"\`
  }
  if (typeof value === 'number' || typeof value === 'boolean') {
    return String(value)
  }
  
  return String(value)
};`,
      `const show = (value: any): void => {
  console.log(toString(value))
};`,
      "",
      "const arrayToList = curry(<T>(arr: T[]): List<T> => {",
      "  let result: List<T> = Empty;",
      "  for (let i = arr.length - 1; i >= 0; i--) {",
      "    result = Cons(arr[i], result);",
      "  }",
      "  return result;",
      "});",
      "",
      "const listToArray = curry(<T>(list: List<T>): T[] => {",
      "  const result: T[] = [];",
      "  let current = list;",
      "  while (current.tag === 'Cons') {",
      "    result.push(current.head);",
      "    current = current.tail;",
      "  }",
      "  return result;",
      "});",
    ]
  }

  private generateCurryFunction(): string[] {
    return [
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
    } else if (stmt instanceof ExpressionStatement) {
      return this.generateExpressionStatement(stmt)
    }

    return `// Unsupported statement: ${stmt.constructor.name}`
  }

  // 式ステートメントの生成
  generateExpressionStatement(stmt: ExpressionStatement): string {
    const expr = this.generateExpression(stmt.expression)
    return `${expr};`
  }

  // 関数宣言の生成
  generateFunctionDeclaration(func: FunctionDeclaration): string {
    const indent = (this.options.indent || "  ").repeat(this.indentLevel)
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
    const indent = (this.options.indent || "  ").repeat(this.indentLevel)
    const type = varDecl.type ? `: ${this.generateType(varDecl.type)}` : ""
    const value = this.generateExpression(varDecl.initializer)

    return `${indent}const ${varDecl.name}${type} = ${value};`
  }

  // 型宣言の生成
  generateTypeDeclaration(typeDecl: TypeDeclaration): string {
    const indent = (this.options.indent || "  ").repeat(this.indentLevel)

    if (typeDecl.fields && typeDecl.fields.length > 0) {
      // Check if this is a union type (ADT) or struct type
      const isUnionType = typeDecl.fields.some(f => 
        f.type instanceof PrimitiveType && f.type.name === "Unit" ||
        f.type instanceof GenericType && f.type.name === "Tuple"
      )

      if (isUnionType) {
        // Union type (ADT) - generate TypeScript discriminated union
        const variants = typeDecl.fields.map(field => {
          if (field.type instanceof PrimitiveType && field.type.name === "Unit") {
            // Simple variant without data
            return `{ type: '${field.name}' }`
          } else if (field.type instanceof GenericType && field.type.name === "Tuple") {
            // Variant with associated data
            const dataTypes = field.type.typeArguments
              .map((t, i) => `data${i}: ${this.generateType(t)}`)
              .join(', ')
            return `{ type: '${field.name}', ${dataTypes} }`
          } else {
            // Fallback
            return `{ type: '${field.name}', data: ${this.generateType(field.type)} }`
          }
        }).join(' | ')

        // Also generate constructor functions
        const constructors = typeDecl.fields.map(field => {
          if (field.type instanceof PrimitiveType && field.type.name === "Unit") {
            return `${indent}const ${field.name} = { type: '${field.name}' as const };`
          } else if (field.type instanceof GenericType && field.type.name === "Tuple") {
            const params = field.type.typeArguments
              .map((t, i) => `data${i}: ${this.generateType(t)}`)
              .join(', ')
            const dataFields = field.type.typeArguments
              .map((_, i) => `data${i}`)
              .join(', ')
            return `${indent}const ${field.name} = (${params}) => ({ type: '${field.name}' as const, ${dataFields} });`
          } else {
            return `${indent}const ${field.name} = (data: ${this.generateType(field.type)}) => ({ type: '${field.name}' as const, data });`
          }
        }).join('\n')

        return `${indent}type ${typeDecl.name} = ${variants};\n\n${constructors}`
      } else {
        // Struct type - generate interface
        const fields = typeDecl.fields
          .map((f) => `  ${f.name}: ${this.generateType(f.type)}`)
          .join(";\n")

        return `${indent}type ${typeDecl.name} = {\n${fields}\n};`
      }
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
    } else if (expr instanceof UnaryOperation) {
      return this.generateUnaryOperation(expr)
    } else if (expr instanceof FunctionCall) {
      return this.generateFunctionCall(expr)
    } else if (expr instanceof FunctionApplication) {
      return this.generateFunctionApplication(expr)
    } else if (expr instanceof BuiltinFunctionCall) {
      return this.generateBuiltinFunctionCall(expr)
    } else if (expr instanceof ConditionalExpression) {
      return this.generateConditionalExpression(expr)
    } else if (expr instanceof MatchExpression) {
      return this.generateMatchExpression(expr)
    } else if (expr instanceof Pipeline) {
      return this.generatePipeline(expr)
    } else if (expr instanceof ReversePipe) {
      return this.generateReversePipe(expr)
    } else if (expr instanceof FunctorMap) {
      return this.generateFunctorMap(expr)
    } else if (expr instanceof ApplicativeApply) {
      return this.generateApplicativeApply(expr)
    } else if (expr instanceof MonadBind) {
      return this.generateMonadBind(expr)
    } else if (expr instanceof FoldMonoid) {
      return this.generateFoldMonoid(expr)
    } else if (expr instanceof FunctionApplicationOperator) {
      return this.generateFunctionApplicationOperator(expr)
    } else if (expr instanceof ConstructorExpression) {
      return this.generateConstructorExpression(expr)
    } else if (expr instanceof BlockExpression) {
      return this.generateBlockExpression(expr)
    } else if (expr instanceof LambdaExpression) {
      return this.generateLambdaExpression(expr)
    } else if (expr instanceof RecordExpression) {
      return this.generateRecordExpression(expr)
    } else if (expr instanceof RecordAccess) {
      return this.generateRecordAccess(expr)
    } else if (expr instanceof ArrayLiteral) {
      return this.generateArrayLiteral(expr)
    } else if (expr instanceof ArrayAccess) {
      return this.generateArrayAccess(expr)
    } else if (expr instanceof ListSugar) {
      return this.generateListSugar(expr)
    } else if (expr instanceof ConsExpression) {
      return this.generateConsExpression(expr)
    } else if (expr instanceof RangeLiteral) {
      return this.generateRangeLiteral(expr)
    } else if (expr instanceof ListComprehension) {
      return this.generateListComprehension(expr)
    } else if (expr instanceof ListComprehensionSugar) {
      return this.generateListComprehensionSugar(expr)
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

    // CONS演算子の特別処理
    if (binOp.operator === ":") {
      return `Cons(${left}, ${right})`
    }

    // 演算子の変換
    let operator = binOp.operator
    if (operator === "==") operator = "==="
    if (operator === "!=") operator = "!=="

    return `(${left} ${operator} ${right})`
  }

  // 単項演算の生成
  generateUnaryOperation(unaryOp: UnaryOperation): string {
    const operand = this.generateExpression(unaryOp.operand)
    
    // 演算子をそのまま使用（TypeScriptと同じ）
    return `(${unaryOp.operator}${operand})`
  }

  // 関数呼び出しの生成
  generateFunctionCall(call: FunctionCall): string {
    const func = this.generateExpression(call.function)
    const args = call.arguments.map((arg) => this.generateExpression(arg))

    return `${func}(${args.join(", ")})`
  }

  // 関数適用の生成
  generateFunctionApplication(app: FunctionApplication): string {
    const func = this.generateExpression(app.function)
    const arg = this.generateExpression(app.argument)

    // ビルトイン関数の特別処理
    if (app.function instanceof Identifier) {
      const funcName = app.function.name
      if (funcName === "print" || funcName === "putStrLn") {
        return `console.log(${arg})`
      } else if (funcName === "toString") {
        return `toString(${arg})`
      } else if (funcName === "show") {
        return `show(${arg})`
      }
    }

    // ネストした関数適用の処理
    if (app.function instanceof FunctionApplication) {
      const nestedFunc = app.function.function
      if (nestedFunc instanceof Identifier) {
        const funcName = nestedFunc.name
        if (funcName === "print" || funcName === "putStrLn") {
          const firstArg = this.generateExpression(app.function.argument)
          return `console.log(${firstArg}, ${arg})`
        }
      }
    }

    // 通常の関数適用 - wrap lambda expressions in parentheses
    if (app.function instanceof LambdaExpression) {
      return `(${func})(${arg})`
    }
    return `${func}(${arg})`
  }

  // ビルトイン関数呼び出しの生成
  generateBuiltinFunctionCall(call: BuiltinFunctionCall): string {
    const args = call.arguments.map((arg) => this.generateExpression(arg))

    switch (call.functionName) {
      case "print":
        return `console.log(${args.join(", ")})`
      case "putStrLn":
        return `console.log(${args.join(", ")})`
      case "toString":
        if (args.length !== 1) {
          throw new Error("toString requires exactly one argument")
        }
        return `toString(${args[0]})`
      case "show":
        if (args.length !== 1) {
          throw new Error("show requires exactly one argument")
        }
        return `show(${args[0]})`
      default:
        throw new Error(`Unknown builtin function: ${call.functionName}`)
    }
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

    // if-else チェーンとして生成（柔軟性を向上）
    const cases = match.cases
    let result = "(() => {\n  const matchValue = " + expr + ";\n"
    
    for (let i = 0; i < cases.length; i++) {
      const c = cases[i]
      const condition = this.generatePatternCondition(c.pattern, "matchValue")
      const body = this.generateExpression(c.expression)
      
      if (i === 0) {
        result += `  if (${condition}) {\n    return ${body};\n  }`
      } else {
        result += ` else if (${condition}) {\n    return ${body};\n  }`
      }
    }
    
    result += " else {\n    throw new Error('Non-exhaustive pattern match');\n  }\n})()"
    return result
  }

  // パターン条件の生成（if-else用）
  generatePatternCondition(pattern: any, valueVar: string): string {
    if (!pattern) {
      return "true" // ワイルドカードパターン
    }
    
    // ASTパターンの種類に基づいて処理
    switch (pattern.kind) {
      case "LiteralPattern":
        if (typeof pattern.value === "string") {
          return `${valueVar} === ${JSON.stringify(pattern.value)}`
        } else {
          return `${valueVar} === ${pattern.value}`
        }
      
      case "IdentifierPattern":
        if (pattern.name === "_") {
          return "true" // ワイルドカードパターン
        }
        // 変数にバインドする場合（常に true）
        return "true"
      
      case "ConstructorPattern":
        // ADTコンストラクタパターン
        return `${valueVar}.type === '${pattern.constructorName}'`
      
      default:
        // 後方互換性のための古い形式をチェック
        if (pattern.value !== undefined) {
          if (typeof pattern.value === "string") {
            return `${valueVar} === ${JSON.stringify(pattern.value)}`
          } else {
            return `${valueVar} === ${pattern.value}`
          }
        }
        
        if (pattern.name) {
          if (pattern.name === "_") {
            return "true"
          }
          return "true"
        }
        
        if (pattern.constructor) {
          return `${valueVar}.type === '${pattern.constructor}'`
        }
        
        return `${valueVar} === ${JSON.stringify(pattern.toString())}`
    }
  }

  // パターンの生成（旧メソッド、下位互換性のため保持）
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

  // ファンクターマップの生成
  generateFunctorMap(map: FunctorMap): string {
    const func = this.generateExpression(map.left)
    const value = this.generateExpression(map.right)

    return `map(${func}, ${value})`
  }

  // アプリカティブ適用の生成
  generateApplicativeApply(apply: ApplicativeApply): string {
    const wrapped = this.generateExpression(apply.left)
    const value = this.generateExpression(apply.right)

    return `applyWrapped(${wrapped}, ${value})`
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

  // 関数適用演算子の生成
  generateFunctionApplicationOperator(
    app: FunctionApplicationOperator
  ): string {
    const left = this.generateExpression(app.left)
    const right = this.generateExpression(app.right)

    // ビルトイン関数の特別処理
    if (app.left instanceof Identifier) {
      const funcName = app.left.name
      if (funcName === "print" || funcName === "putStrLn") {
        return `console.log(${right})`
      } else if (funcName === "toString") {
        return `toString(${right})`
      } else if (funcName === "show") {
        return `show(${right})`
      }
    }

    // $ は右結合で、基本的には関数呼び出しと同じ
    // f $ x → f(x)
    return `${left}(${right})`
  }

  // コンストラクタ式の生成
  generateConstructorExpression(expr: ConstructorExpression): string {
    const name = expr.constructorName
    const args = expr.arguments.map((arg) => this.generateExpression(arg))

    switch (name) {
      case "Nothing":
        return "Nothing"
      case "Just":
        return args.length > 0 ? `Just(${args[0]})` : "Just"
      case "Left":
        return args.length > 0 ? `Left(${args[0]})` : "Left"
      case "Right":
        return args.length > 0 ? `Right(${args[0]})` : "Right"
      case "Empty":
        return "Empty"
      case "Cons":
        return args.length === 2 ? `Cons(${args[0]}, ${args[1]})` : "Cons"
      default:
        // 一般的なコンストラクタ
        return args.length > 0 ? `${name}(${args.join(", ")})` : name
    }
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
        case "_":
          return "any"  // Placeholder type for inference
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
    } else if (type instanceof RecordType) {
      if (type.fields.length === 0) {
        return "{}"
      }
      const fields = type.fields
        .map((field) => `${field.name}: ${this.generateType(field.type)}`)
        .join(", ")
      return `{ ${fields} }`
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
        return "List"
      case "Array":
        return "Array"
      default:
        return name
    }
  }

  // ブロック式の生成
  generateBlockExpression(expr: BlockExpression): string {
    const lines: string[] = []

    // ブロック内の文を生成
    for (const stmt of expr.statements) {
      const code = this.generateStatement(stmt)
      if (code.trim()) {
        lines.push(code)
      }
    }

    // 最後の式/return文を追加
    if (expr.returnExpression) {
      lines.push(`return ${this.generateExpression(expr.returnExpression)};`)
    }

    // IIFEとして生成（即座に実行される関数式）
    return `(() => {\n${lines.map((line) => `  ${line}`).join("\n")}\n})()`
  }

  // ラムダ式の生成
  generateLambdaExpression(expr: LambdaExpression): string {
    const body = this.generateExpression(expr.body)

    // For single parameter lambdas, we can use arrow function syntax
    if (expr.parameters.length === 1) {
      const param = expr.parameters[0]
      const paramWithType = `${param.name}: ${this.generateType(param.type)}`
      return `(${paramWithType}) => ${body}`
    }

    // For multi-parameter lambdas, we need to curry them
    // \a -> \b -> expr becomes (a) => (b) => expr
    let result = body
    for (let i = expr.parameters.length - 1; i >= 0; i--) {
      const param = expr.parameters[i]
      const paramWithType = `${param.name}: ${this.generateType(param.type)}`
      result = `(${paramWithType}) => ${result}`
    }

    return result
  }

  // Record式の生成
  generateRecordExpression(record: RecordExpression): string {
    if (record.fields.length === 0) {
      return "{}"
    }

    const fields = record.fields.map(field => {
      const value = this.generateExpression(field.value)
      return `${field.name}: ${value}`
    })

    return `{ ${fields.join(", ")} }`
  }

  // Recordアクセスの生成
  generateRecordAccess(access: RecordAccess): string {
    const record = this.generateExpression(access.record)
    return `${record}.${access.fieldName}`
  }

  // 配列リテラルの生成
  generateArrayLiteral(arrayLiteral: ArrayLiteral): string {
    if (arrayLiteral.elements.length === 0) {
      return "[]"
    }

    const elements = arrayLiteral.elements.map(element => 
      this.generateExpression(element)
    )

    return `[${elements.join(", ")}]`
  }

  // 配列アクセスの生成
  generateArrayAccess(arrayAccess: ArrayAccess): string {
    const array = this.generateExpression(arrayAccess.array)
    const index = this.generateExpression(arrayAccess.index)
    return `${array}[${index}]`
  }

  // リストシュガーの生成 [1, 2, 3] -> Cons(1, Cons(2, Cons(3, Empty)))
  generateListSugar(listSugar: ListSugar): string {
    if (listSugar.elements.length === 0) {
      return "Empty"
    }

    // リストを右からConsで構築
    let result = "Empty"
    for (let i = listSugar.elements.length - 1; i >= 0; i--) {
      const element = this.generateExpression(listSugar.elements[i])
      result = `Cons(${element}, ${result})`
    }

    return result
  }

  // Cons式の生成 left : right -> Cons(left, right)
  generateConsExpression(consExpr: ConsExpression): string {
    const left = this.generateExpression(consExpr.left)
    const right = this.generateExpression(consExpr.right)
    return `Cons(${left}, ${right})`
  }

  // 範囲リテラルの生成
  generateRangeLiteral(range: RangeLiteral): string {
    const start = this.generateExpression(range.start)
    const end = this.generateExpression(range.end)
    
    if (range.inclusive) {
      // 1..=5 -> Array.from({length: 5 - 1 + 1}, (_, i) => i + 1)
      return `Array.from({length: ${end} - ${start} + 1}, (_, i) => i + ${start})`
    } else {
      // 1..5 -> Array.from({length: 5 - 1}, (_, i) => i + 1)
      return `Array.from({length: ${end} - ${start}}, (_, i) => i + ${start})`
    }
  }

  // リスト内包表記の生成
  generateListComprehension(comp: ListComprehension): string {
    // [x * 2 | x <- range, x % 2 == 0] -> 
    // range.filter(x => x % 2 == 0).map(x => x * 2)
    
    let result = ""
    
    // 最初のジェネレータから開始
    if (comp.generators.length > 0) {
      const firstGenerator = comp.generators[0]
      result = this.generateExpression(firstGenerator.iterable)
      
      // 追加のジェネレータがある場合はflatMapを使用
      for (let i = 1; i < comp.generators.length; i++) {
        const generator = comp.generators[i]
        const iterable = this.generateExpression(generator.iterable)
        result = `${result}.flatMap(${firstGenerator.variable} => ${iterable}.map(${generator.variable} => [${firstGenerator.variable}, ${generator.variable}]))`
      }
      
      // フィルタを適用
      for (const filter of comp.filters) {
        const filterExpr = this.generateExpression(filter)
        // ジェネレータ変数を適切に置換
        if (comp.generators.length === 1) {
          result = `${result}.filter(${comp.generators[0].variable} => ${filterExpr})`
        } else {
          // 複数ジェネレータの場合は複雑になるので簡略化
          result = `${result}.filter(tuple => {
            const [${comp.generators.map(g => g.variable).join(', ')}] = tuple;
            return ${filterExpr};
          })`
        }
      }
      
      // 最終的な式をマップ
      const expression = this.generateExpression(comp.expression)
      if (comp.generators.length === 1) {
        result = `${result}.map(${comp.generators[0].variable} => ${expression})`
      } else {
        result = `${result}.map(tuple => {
          const [${comp.generators.map(g => g.variable).join(', ')}] = tuple;
          return ${expression};
        })`
      }
    }
    
    return result || "[]"
  }

  // リスト内包表記（Sugar版）の生成 - Seseragiリストを返す
  generateListComprehensionSugar(comp: ListComprehensionSugar): string {
    // まず通常の配列内包表記を生成
    let arrayResult = ""
    
    // 最初のジェネレータから開始
    if (comp.generators.length > 0) {
      const firstGenerator = comp.generators[0]
      arrayResult = this.generateExpression(firstGenerator.iterable)
      
      // 追加のジェネレータがある場合はflatMapを使用
      for (let i = 1; i < comp.generators.length; i++) {
        const generator = comp.generators[i]
        const iterable = this.generateExpression(generator.iterable)
        arrayResult = `${arrayResult}.flatMap(${firstGenerator.variable} => ${iterable}.map(${generator.variable} => [${firstGenerator.variable}, ${generator.variable}]))`
      }
      
      // フィルタを適用
      for (const filter of comp.filters) {
        const filterExpr = this.generateExpression(filter)
        // ジェネレータ変数を適切に置換
        if (comp.generators.length === 1) {
          arrayResult = `${arrayResult}.filter(${comp.generators[0].variable} => ${filterExpr})`
        } else {
          // 複数ジェネレータの場合は複雑になるので簡略化
          arrayResult = `${arrayResult}.filter(tuple => {
            const [${comp.generators.map(g => g.variable).join(', ')}] = tuple;
            return ${filterExpr};
          })`
        }
      }
      
      // 最終的な式をマップ
      const expression = this.generateExpression(comp.expression)
      if (comp.generators.length === 1) {
        arrayResult = `${arrayResult}.map(${comp.generators[0].variable} => ${expression})`
      } else {
        arrayResult = `${arrayResult}.map(tuple => {
          const [${comp.generators.map(g => g.variable).join(', ')}] = tuple;
          return ${expression};
        })`
      }
    }
    
    if (!arrayResult) {
      return "Empty"
    }
    
    // 配列をSeseragiリストに変換
    return `arrayToList(${arrayResult})`
  }
}
