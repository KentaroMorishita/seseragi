import {
  type ApplicativeApply,
  type ArrayAccess,
  type ArrayLiteral,
  type ArrayPattern,
  type BinaryOperation,
  BlockExpression,
  type BuiltinFunctionCall,
  type ConditionalExpression,
  type ConsExpression,
  type ConstructorExpression,
  type ConstructorPattern,
  type Expression,
  ExpressionStatement,
  type FoldMonoid,
  FunctionApplication,
  type FunctionApplicationOperator,
  FunctionCall,
  FunctionDeclaration,
  FunctionType,
  type FunctorMap,
  GenericType,
  type GuardPattern,
  Identifier,
  type IdentifierPattern,
  ImplBlock,
  IntersectionType,
  LambdaExpression,
  ListComprehension,
  type ListComprehensionSugar,
  type ListSugar,
  type ListSugarPattern,
  type Literal,
  type LiteralPattern,
  type MatchExpression,
  type MethodCall,
  type MethodDeclaration,
  type MonadBind,
  type NullishCoalescingExpression,
  type MonoidDeclaration,
  type OperatorDeclaration,
  type OrPattern,
  type Pattern,
  type Pipeline,
  PrimitiveType,
  type RangeLiteral,
  type RecordAccess,
  RecordDestructuring,
  type RecordExpression,
  type RecordInitField,
  type RecordShorthandField,
  type RecordSpreadField,
  RecordType,
  type ReversePipe,
  type SpreadExpression,
  type Statement,
  StructDeclaration,
  StructDestructuring,
  StructExpression,
  StructType,
  type TemplateExpression,
  type TernaryExpression,
  TupleDestructuring,
  type TupleExpression,
  type TuplePattern,
  TupleType,
  type Type,
  TypeAliasDeclaration,
  type TypeAssertion,
  TypeDeclaration,
  type UnaryOperation,
  UnionType,
  VariableDeclaration,
} from "./ast"
import type { TypeInferenceSystemResult } from "./type-inference"
import { type UsageAnalysis, UsageAnalyzer } from "./usage-analyzer"

/**
 * Seseragi から TypeScript へのコード生成器
 * SeseragiをTypeScriptコードに変換
 */

export interface CodeGenOptions {
  indent?: string
  useArrowFunctions?: boolean
  generateComments?: boolean
  runtimeMode?: "embedded" | "import"
  filePath?: string // ファイルパス（ハッシュ生成用）
  typeInferenceResult?: TypeInferenceSystemResult // 型推論結果
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

export class CodeGenerator {
  options: CodeGenOptions
  indentLevel: number
  usageAnalysis?: UsageAnalysis
  wildcardCounter: number
  filePrefix: string
  currentStructContext: string | null = null // 現在処理中の構造体名
  structMethods: Map<string, Set<string>> = new Map() // 構造体名 → メソッド名のセット
  structOperators: Map<string, Set<string>> = new Map() // 構造体名 → 演算子のセット
  typeInferenceResult: TypeInferenceSystemResult | null = null // 型推論結果

  constructor(options: CodeGenOptions) {
    this.options = options
    this.indentLevel = 0
    this.wildcardCounter = 1
    this.typeInferenceResult = options.typeInferenceResult
    this.filePrefix = this.generateFilePrefix(options.filePath || "unknown")
  }

  // ビルトインコンストラクタかどうかを判定
  private isBuiltinConstructor(name: string): boolean {
    return ["Just", "Nothing", "Left", "Right", "Empty", "Cons"].includes(name)
  }

  // パターンから変数バインディングを生成
  private generatePatternBindings(pattern: Pattern, valueVar: string): string {
    if (!pattern) return ""

    switch (pattern.kind) {
      case "IdentifierPattern":
        return this.generateIdentifierPatternBindings(
          pattern as IdentifierPattern,
          valueVar
        )
      case "ConstructorPattern":
        return this.generateConstructorPatternBindings(
          pattern as ConstructorPattern,
          valueVar
        )
      case "TuplePattern":
        return this.generateTuplePatternBindings(
          pattern as TuplePattern,
          valueVar
        )

      case "OrPattern":
        return this.generateOrPatternBindings(pattern as OrPattern, valueVar)
      case "GuardPattern":
        return this.generateGuardPatternBindings(
          pattern as GuardPattern,
          valueVar
        )
      case "ListSugarPattern":
        return this.generateListSugarPatternBindings(
          pattern as ListSugarPattern,
          valueVar
        )
      case "ArrayPattern":
        return this.generateArrayPatternBindings(
          pattern as ArrayPattern,
          valueVar
        )

      default:
        return ""
    }
  }

  // ファイルパスからハッシュベースのプレフィックスを生成
  private generateFilePrefix(filePath: string): string {
    // ファイルパスの簡易ハッシュ（名前衝突回避用）
    let hash = 0
    for (let i = 0; i < filePath.length; i++) {
      const char = filePath.charCodeAt(i)
      hash = (hash << 5) - hash + char
      hash = hash & hash // Convert to 32-bit integer
    }
    return `f${Math.abs(hash).toString(36).slice(0, 6)}`
  }

  // プログラム全体の生成
  generateProgram(statements: Statement[]): string {
    // statementsのガード
    if (!statements || !Array.isArray(statements)) {
      return ""
    }

    // 使用分析を実行
    this.performUsageAnalysis(statements)

    const lines: string[] = []

    // コメントとランタイムの生成
    this.addProgramHeader(lines)

    // 構造体前処理とディスパッチテーブル生成
    this.processStructuresAndDispatch(statements, lines)

    // 文を分類して生成
    this.generateStatementsByType(statements, lines)

    return lines.join("\n")
  }

  private performUsageAnalysis(statements: Statement[]): void {
    const analyzer = new UsageAnalyzer()
    this.usageAnalysis = analyzer.analyze(statements)
  }

  private addProgramHeader(lines: string[]): void {
    if (this.options.generateComments) {
      lines.push("// Generated TypeScript code from Seseragi")
      lines.push("")
    }

    // ランタイムの生成
    lines.push(...this.generateRuntime())
    lines.push("")
  }

  private processStructuresAndDispatch(
    statements: Statement[],
    lines: string[]
  ): void {
    // まず構造体を処理してディスパッチテーブルを準備
    for (const stmt of statements) {
      if (stmt instanceof ImplBlock) {
        this.preProcessImplBlock(stmt)
      }
    }

    // ディスパッチテーブルが必要かどうかを判定
    const needsDispatchTables = this.shouldGenerateDispatchTables(statements)

    if (needsDispatchTables) {
      lines.push(this.generateDispatchTables())
      lines.push("")
    }
  }

  private shouldGenerateDispatchTables(statements: Statement[]): boolean {
    const usesOperatorDispatch = this.checkUsesOperatorDispatch(statements)
    const hasBinaryOperations = this.hasBinaryOperations(statements)
    const hasStructs = this.hasStructureRelatedStatements(statements)
    const hasDispatchOperatorUsage = this.hasDispatchOperatorUsage(statements)

    return (
      hasStructs ||
      hasBinaryOperations ||
      usesOperatorDispatch ||
      hasDispatchOperatorUsage
    )
  }

  private hasStructureRelatedStatements(statements: Statement[]): boolean {
    return statements.some(
      (stmt) =>
        stmt instanceof StructDeclaration ||
        stmt instanceof ImplBlock ||
        (stmt instanceof ExpressionStatement &&
          stmt.expression instanceof StructExpression)
    )
  }

  private generateStatementsByType(
    statements: Statement[],
    lines: string[]
  ): void {
    const { structStatements, implStatements, otherStatements } =
      this.categorizeStatements(statements)

    // 構造体定義
    this.generateStatementsOfType(structStatements, lines)

    // 実装ブロック
    this.generateStatementsOfType(implStatements, lines)

    // ディスパッチテーブル初期化
    this.generateDispatchTableInit(lines)

    // 残りの文
    this.generateStatementsOfType(otherStatements, lines)
  }

  private categorizeStatements(statements: Statement[]): {
    structStatements: Statement[]
    implStatements: Statement[]
    otherStatements: Statement[]
  } {
    const structStatements: Statement[] = []
    const implStatements: Statement[] = []
    const otherStatements: Statement[] = []

    for (const stmt of statements) {
      if (stmt instanceof StructDeclaration) {
        structStatements.push(stmt)
      } else if (stmt instanceof ImplBlock) {
        implStatements.push(stmt)
      } else {
        otherStatements.push(stmt)
      }
    }

    return { structStatements, implStatements, otherStatements }
  }

  private generateStatementsOfType(
    statements: Statement[],
    lines: string[]
  ): void {
    for (const stmt of statements) {
      const code = this.generateStatement(stmt)
      if (code.trim()) {
        lines.push(code)
        lines.push("")
      }
    }
  }

  private generateDispatchTableInit(lines: string[]): void {
    if (this.structMethods.size > 0 || this.structOperators.size > 0) {
      lines.push("// Initialize dispatch tables immediately")
      lines.push("(() => {")
      const initCode = this.generateDispatchTableInitialization()
      lines.push(
        ...initCode.split("\n").map((line) => (line ? `  ${line}` : line))
      )
      lines.push("})();")
      lines.push("")
    }
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
    this.collectBasicImports(imports)
    this.collectTypeImports(imports)
    this.collectFunctionalImports(imports)
    this.collectBuiltinImports(imports)

    if (imports.length > 0) {
      lines.push(
        `import { ${imports.join(", ")} } from './runtime/seseragi-runtime.js';`
      )
    }

    return lines
  }

  private collectBasicImports(imports: string[]): void {
    if (!this.usageAnalysis) return

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
  }

  private collectTypeImports(imports: string[]): void {
    if (!this.usageAnalysis) return

    if (this.usageAnalysis.needsMaybe) {
      imports.push("Just", "Nothing", "type Maybe")
    }
    if (this.usageAnalysis.needsEither) {
      imports.push("Left", "Right", "type Either")
    }
  }

  private collectFunctionalImports(imports: string[]): void {
    if (!this.usageAnalysis) return

    if (this.usageAnalysis.needsFunctorMap) {
      imports.push("mapMaybe", "mapEither", "mapList", "mapArray")
    }
    if (this.usageAnalysis.needsApplicativeApply) {
      imports.push("applyMaybe", "applyEither", "applyList", "applyArray")
    }
    if (this.usageAnalysis.needsMonadBind) {
      imports.push("bindMaybe", "bindEither", "bindList", "bindArray")
    }
    if (this.usageAnalysis.needsFoldMonoid) {
      imports.push("foldMonoid")
    }
  }

  private collectBuiltinImports(imports: string[]): void {
    if (!this.usageAnalysis) return

    const builtins = this.usageAnalysis.needsBuiltins
    const builtinMap: Record<string, string> = {
      print: "print",
      putStrLn: "putStrLn",
      toString: "toString",
      toInt: "toInt",
      toFloat: "toFloat",
      show: "show",
      arrayToList: "arrayToList",
      listToArray: "listToArray",
      head: "headList",
      tail: "tailList",
    }

    for (const [key, value] of Object.entries(builtinMap)) {
      if (builtins[key as keyof typeof builtins]) {
        imports.push(value)
      }
    }
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
      "// Array monadic functions",
      "const mapArray = <T, U>(fa: T[], f: (a: T) => U): U[] => {",
      "  return fa.map(f);",
      "};",
      "",
      "const applyArray = <T, U>(ff: ((a: T) => U)[], fa: T[]): U[] => {",
      "  const result: U[] = [];",
      "  for (const func of ff) {",
      "    for (const value of fa) {",
      "      result.push(func(value));",
      "    }",
      "  }",
      "  return result;",
      "};",
      "",
      "const bindArray = <T, U>(ma: T[], f: (value: T) => U[]): U[] => {",
      "  const result: U[] = [];",
      "  for (const value of ma) {",
      "    result.push(...f(value));",
      "  }",
      "  return result;",
      "};",
      "",
      "// List monadic functions",
      "const mapList = <T, U>(fa: any, f: (a: T) => U): any => {",
      "  if (fa.tag === 'Empty') return { tag: 'Empty' };",
      "  return { tag: 'Cons', head: f(fa.head), tail: mapList(fa.tail, f) };",
      "};",
      "",
      "const applyList = <T, U>(ff: any, fa: any): any => {",
      "  if (ff.tag === 'Empty') return { tag: 'Empty' };",
      "  const mappedValues = mapList(fa, ff.head);",
      "  const restApplied = applyList(ff.tail, fa);",
      "  return concatList(mappedValues, restApplied);",
      "};",
      "",
      "const concatList = <T>(list1: any, list2: any): any => {",
      "  if (list1.tag === 'Empty') return list2;",
      "  return { tag: 'Cons', head: list1.head, tail: concatList(list1.tail, list2) };",
      "};",
      "",
      "const bindList = <T, U>(ma: any, f: (value: T) => any): any => {",
      "  if (ma.tag === 'Empty') return { tag: 'Empty' };",
      "  const headResult = f(ma.head);",
      "  const tailResult = bindList(ma.tail, f);",
      "  return concatList(headResult, tailResult);",
      "};",
      "",
      "// Maybe monadic functions",
      "const mapMaybe = <T, U>(fa: Maybe<T>, f: (a: T) => U): Maybe<U> => {",
      "  return fa.tag === 'Just' ? Just(f(fa.value)) : Nothing;",
      "};",
      "",
      "const applyMaybe = <T, U>(ff: Maybe<(a: T) => U>, fa: Maybe<T>): Maybe<U> => {",
      "  return ff.tag === 'Just' && fa.tag === 'Just' ? Just(ff.value(fa.value)) : Nothing;",
      "};",
      "",
      "const bindMaybe = <T, U>(ma: Maybe<T>, f: (value: T) => Maybe<U>): Maybe<U> => {",
      "  return ma.tag === 'Just' ? f(ma.value) : Nothing;",
      "};",
      "",
      "// Either monadic functions",
      "const mapEither = <L, R, U>(ea: Either<L, R>, f: (value: R) => U): Either<L, U> => {",
      "  return ea.tag === 'Right' ? Right(f(ea.value)) : ea;",
      "};",
      "",
      "const applyEither = <L, R, U>(ef: Either<L, (value: R) => U>, ea: Either<L, R>): Either<L, U> => {",
      "  return ef.tag === 'Right' && ea.tag === 'Right' ? Right(ef.value(ea.value)) :",
      "         ef.tag === 'Left' ? ef : ea;",
      "};",
      "",
      "const bindEither = <L, R, U>(ea: Either<L, R>, f: (value: R) => Either<L, U>): Either<L, U> => {",
      "  return ea.tag === 'Right' ? f(ea.value) : ea;",
      "};",
      "",
      "const Just = <T>(value: T): Maybe<T> => ({ tag: 'Just', value });",
      "const Nothing: Maybe<never> = { tag: 'Nothing' };",
      "",
      "const Left = <L>(value: L): Either<L, never> => ({ tag: 'Left', value });",
      "const Right = <R>(value: R): Either<never, R> => ({ tag: 'Right', value });",
      "",
      "// Nullish coalescing helper functions",
      "const fromMaybe = <T>(defaultValue: T, maybe: Maybe<T>): T => {",
      "  return maybe.tag === 'Just' ? maybe.value : defaultValue;",
      "};",
      "",
      "const fromRight = <L, R>(defaultValue: R, either: Either<L, R>): R => {",
      "  return either.tag === 'Right' ? either.value : defaultValue;",
      "};",
      "",
      "const fromLeft = <L, R>(defaultValue: L, either: Either<L, R>): L => {",
      "  return either.tag === 'Left' ? either.value : defaultValue;",
      "};",
      "",
      "const Empty: List<never> = { tag: 'Empty' };",
      "const Cons = <T>(head: T, tail: List<T>): List<T> => ({ tag: 'Cons', head, tail });",
      "",
      "const headList = <T>(list: List<T>): Maybe<T> => list.tag === 'Cons' ? { tag: 'Just', value: list.head } : { tag: 'Nothing' };",
      "const tailList = <T>(list: List<T>): List<T> => list.tag === 'Cons' ? list.tail : Empty;",
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
    return "\`[]"
  }
  if (value && typeof value === 'object' && value.tag === 'Cons') {
    const items = []
    let current = value
    while (current.tag === 'Cons') {
      items.push(toString(current.head))
      current = current.tail
    }
    return "\`[" + items.join(', ') + "]"
  }

  // Tuple型の美しい表示
  if (value && typeof value === 'object' && value.tag === 'Tuple') {
    return \`(\${value.elements.map(toString).join(', ')})\`
  }

  // 配列の表示
  if (Array.isArray(value)) {
    return \`[\${value.map(toString).join(', ')}]\`
  }

  // プリミティブ型
  if (typeof value === 'string') {
    return \`"\${value}"\`
  }
  if (typeof value === 'number') {
    return String(value)
  }
  if (typeof value === 'boolean') {
    return value ? 'True' : 'False'
  }

  // 普通のオブジェクト（構造体など）
  if (typeof value === 'object' && value !== null) {
    const pairs = []
    for (const key in value) {
      if (value.hasOwnProperty(key)) {
        pairs.push(\`\${key}: \${toString(value[key])}\`)
      }
    }

    // 構造体名を取得（constructor.nameを使用）
    const structName = value.constructor && value.constructor.name !== 'Object'
      ? value.constructor.name
      : ''

    // 複数フィールドがある場合はインデント表示
    if (pairs.length > 2) {
      return \`\${structName} {\\n  \${pairs.join(',\\n  ')}\\n}\`
    } else {
      return \`\${structName} { \${pairs.join(', ')} }\`
    }
  }

  return String(value)
};`,
      `const toInt = (value: any): number => {
  if (typeof value === 'number') {
    return Math.trunc(value)
  }
  if (typeof value === 'string') {
    const n = parseInt(value, 10)
    if (isNaN(n)) {
      throw new Error(\`Cannot convert "\${value}" to Int\`)
    }
    return n
  }
  throw new Error(\`Cannot convert \${typeof value} to Int\`)
};`,
      `const toFloat = (value: any): number => {
  if (typeof value === 'number') {
    return value
  }
  if (typeof value === 'string') {
    const n = parseFloat(value)
    if (isNaN(n)) {
      throw new Error(\`Cannot convert "\${value}" to Float\`)
    }
    return n
  }
  throw new Error(\`Cannot convert \${typeof value} to Float\`)
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
    } else if (stmt instanceof TypeAliasDeclaration) {
      return this.generateTypeAliasDeclaration(stmt)
    } else if (stmt instanceof ExpressionStatement) {
      return this.generateExpressionStatement(stmt)
    } else if (stmt instanceof TupleDestructuring) {
      return this.generateTupleDestructuring(stmt)
    } else if (stmt instanceof RecordDestructuring) {
      return this.generateRecordDestructuring(stmt)
    } else if (stmt instanceof StructDestructuring) {
      return this.generateStructDestructuring(stmt)
    } else if (stmt instanceof StructDeclaration) {
      return this.generateStructDeclaration(stmt)
    } else if (stmt instanceof ImplBlock) {
      return this.generateImplBlock(stmt)
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

    // TypeScriptジェネリクス型パラメータを生成
    const typeParams =
      func.typeParameters && func.typeParameters.length > 0
        ? `<${func.typeParameters.map((tp) => tp.name).join(", ")}>`
        : ""

    const params = func.parameters
      .map((p) => `${p.name}: ${this.generateType(p.type)}`)
      .join(", ")
    const returnType = func.returnType
      ? this.generateType(func.returnType)
      : "any"

    // カリー化された関数として生成
    if (func.parameters.length > 1) {
      const body = this.generateExpression(func.body)
      // オブジェクトリテラルの場合は括弧で囲む
      const wrappedBody = body.startsWith("{") ? `(${body})` : body
      return `${indent}const ${this.sanitizeIdentifier(func.name)} = curry(${typeParams}(${params}): ${returnType} => ${wrappedBody});`
    } else {
      const body = this.generateExpression(func.body)
      if (this.options.useArrowFunctions) {
        // オブジェクトリテラルの場合は括弧で囲む
        const wrappedBody = body.startsWith("{") ? `(${body})` : body
        return `${indent}const ${this.sanitizeIdentifier(func.name)} = ${typeParams}(${params}): ${returnType} => ${wrappedBody};`
      } else {
        return `${indent}function ${this.sanitizeIdentifier(func.name)}${typeParams}(${params}): ${returnType} {\n${indent}  return ${body};\n${indent}}`
      }
    }
  }

  // 変数宣言の生成
  generateVariableDeclaration(varDecl: VariableDeclaration): string {
    const indent = (this.options.indent || "  ").repeat(this.indentLevel)
    const type = varDecl.type ? `: ${this.generateType(varDecl.type)}` : ""
    const value = this.generateExpression(varDecl.initializer)

    return `${indent}const ${this.sanitizeIdentifier(varDecl.name)}${type} = ${value};`
  }

  // 型宣言の生成
  generateTypeDeclaration(typeDecl: TypeDeclaration): string {
    const indent = (this.options.indent || "  ").repeat(this.indentLevel)

    if (typeDecl.fields && typeDecl.fields.length > 0) {
      // Check if this is a union type (ADT) or struct type
      const isUnionType = typeDecl.fields.some(
        (f) =>
          (f.type instanceof PrimitiveType && f.type.name === "Unit") ||
          (f.type instanceof GenericType && f.type.name === "Tuple")
      )

      if (isUnionType) {
        // Union type (ADT) - generate TypeScript discriminated union
        const variants = typeDecl.fields
          .map((field) => {
            if (
              field.type instanceof PrimitiveType &&
              field.type.name === "Unit"
            ) {
              // Simple variant without data
              return `{ type: '${field.name}' }`
            } else if (
              field.type instanceof GenericType &&
              field.type.name === "Tuple"
            ) {
              // Variant with associated data - use List format
              const dataTypes = field.type.typeArguments
                .map((t) => this.generateType(t))
                .join(" | ")
              return `{ type: '${field.name}', data: Array<${dataTypes}> }`
            } else {
              // Fallback - single data as array
              return `{ type: '${field.name}', data: [${this.generateType(field.type)}] }`
            }
          })
          .join(" | ")

        // Also generate constructor functions
        const constructors = typeDecl.fields
          .map((field) => {
            if (
              field.type instanceof PrimitiveType &&
              field.type.name === "Unit"
            ) {
              return `${indent}const ${field.name} = { type: '${field.name}' as const };`
            } else if (
              field.type instanceof GenericType &&
              field.type.name === "Tuple"
            ) {
              const params = field.type.typeArguments
                .map((t, i) => `data${i}: ${this.generateType(t)}`)
                .join(", ")
              const dataArray = field.type.typeArguments
                .map((_, i) => `data${i}`)
                .join(", ")
              return `${indent}const ${field.name} = (${params}) => ({ type: '${field.name}' as const, data: [${dataArray}] });`
            } else {
              return `${indent}const ${field.name} = (data: ${this.generateType(field.type)}) => ({ type: '${field.name}' as const, data: [data] });`
            }
          })
          .join("\n")

        return `${indent}type ${typeDecl.name} = ${variants};\n\n${constructors}`
      } else {
        // Struct type - generate interface
        const fields = typeDecl.fields
          .map((f) => `  ${f.name}: ${this.generateType(f.type)}`)
          .join(";\n")

        return `${indent}type ${typeDecl.name} = {\n${fields}\n};`
      }
    } else {
      // Empty type declaration fallback
      return `${indent}type ${typeDecl.name} = never; // Empty type declaration`
    }
  }

  // 型エイリアス宣言の生成
  generateTypeAliasDeclaration(typeAlias: TypeAliasDeclaration): string {
    const indent = (this.options.indent || "  ").repeat(this.indentLevel)
    const aliasedType = this.generateType(typeAlias.aliasedType)

    // ジェネリック型パラメータがある場合は追加
    let typeParametersStr = ""
    if (typeAlias.typeParameters && typeAlias.typeParameters.length > 0) {
      const paramNames = typeAlias.typeParameters.map((param) => param.name)
      typeParametersStr = `<${paramNames.join(", ")}>`
    }

    return `${indent}type ${typeAlias.name}${typeParametersStr} = ${aliasedType};`
  }

  // 構造体宣言の生成
  generateStructDeclaration(structDecl: StructDeclaration): string {
    const indent = (this.options.indent || "  ").repeat(this.indentLevel)

    // フィールド定義
    const fieldDeclarations = structDecl.fields
      .map((f) => `${indent}  ${f.name}: ${this.generateType(f.type)};`)
      .join("\n")

    // コンストラクタ引数の型定義
    const constructorParamType = structDecl.fields
      .map((f) => {
        // Maybe型フィールドはオプショナルにする
        const isOptional = this.isMaybeType(f.type) ? "?" : ""
        return `${f.name}${isOptional}: ${this.generateType(f.type)}`
      })
      .join(", ")

    // コンストラクタ本体でデフォルト値を適用
    const fieldAssignments = structDecl.fields
      .map((f) => {
        if (this.isMaybeType(f.type)) {
          return `${indent}    this.${f.name} = fields.${f.name} ?? Nothing;`
        } else {
          return `${indent}    this.${f.name} = fields.${f.name};`
        }
      })
      .join("\n")

    return `${indent}class ${structDecl.name} {
${fieldDeclarations}

${indent}  constructor(fields: { ${constructorParamType} }) {
${fieldAssignments}
${indent}  }
${indent}}`
  }

  // Maybe型かどうかをチェック（コード生成用）
  private isMaybeType(type: Type | undefined): boolean {
    if (!type) return false

    // 型推論結果がある場合は置換を適用
    if (this.typeInferenceResult?.substitution) {
      const resolvedType = this.typeInferenceResult.substitution.apply(type)
      if (
        resolvedType &&
        resolvedType.kind === "GenericType" &&
        (resolvedType as GenericType).name === "Maybe"
      ) {
        return true
      }
    }

    // 直接GenericTypeの場合もチェック
    if (type.kind === "GenericType" && (type as GenericType).name === "Maybe") {
      return true
    }

    return false
  }

  // Either型かどうかをチェック
  private isEitherType(type: Type | undefined): boolean {
    if (!type) return false

    // 型推論結果がある場合は置換を適用
    if (this.typeInferenceResult?.substitution) {
      const resolvedType = this.typeInferenceResult.substitution.apply(type)
      if (
        resolvedType &&
        resolvedType.kind === "GenericType" &&
        (resolvedType as GenericType).name === "Either"
      ) {
        return true
      }
    }

    // 直接GenericTypeの場合もチェック
    if (
      type.kind === "GenericType" &&
      (type as GenericType).name === "Either"
    ) {
      return true
    }

    return false
  }

  // impl ブロックの生成
  generateImplBlock(implBlock: ImplBlock): string {
    const indent = (this.options.indent || "  ").repeat(this.indentLevel)
    const lines: string[] = []

    // 構造体コンテキストを設定
    const oldContext = this.currentStructContext
    this.currentStructContext = implBlock.typeName

    // 構造体のメソッドと演算子を登録
    if (!this.structMethods.has(implBlock.typeName)) {
      this.structMethods.set(implBlock.typeName, new Set())
    }
    if (!this.structOperators.has(implBlock.typeName)) {
      this.structOperators.set(implBlock.typeName, new Set())
    }

    const methodSet = this.structMethods.get(implBlock.typeName)!
    const operatorSet = this.structOperators.get(implBlock.typeName)!

    // グローバル関数として生成（namespaceを使わない）
    lines.push(`${indent}// ${implBlock.typeName} implementation`)

    // メソッドの生成
    for (const method of implBlock.methods) {
      const methodCode = this.generateMethodDeclaration(method)
      lines.push(methodCode)
      methodSet.add(method.name)
    }

    // 演算子の生成
    for (const operator of implBlock.operators) {
      const operatorCode = this.generateOperatorDeclaration(operator)
      lines.push(operatorCode)
      operatorSet.add(operator.operator)
    }

    // モノイドの生成
    if (implBlock.monoid) {
      const monoidCode = this.generateMonoidDeclaration(implBlock.monoid)
      lines.push(monoidCode)
    }

    // コンテキストを復元
    this.currentStructContext = oldContext

    return lines.join("\n")
  }

  // impl ブロックを事前処理してディスパッチテーブル情報を収集
  preProcessImplBlock(implBlock: ImplBlock): void {
    // 構造体のメソッドと演算子を登録
    if (!this.structMethods.has(implBlock.typeName)) {
      this.structMethods.set(implBlock.typeName, new Set())
    }
    if (!this.structOperators.has(implBlock.typeName)) {
      this.structOperators.set(implBlock.typeName, new Set())
    }

    const methodSet = this.structMethods.get(implBlock.typeName)!
    const operatorSet = this.structOperators.get(implBlock.typeName)!

    // メソッドを登録
    for (const method of implBlock.methods) {
      methodSet.add(method.name)
    }

    // 演算子を登録
    for (const operator of implBlock.operators) {
      operatorSet.add(operator.operator)
    }
  }

  // 構造体のメソッド・演算子ディスパッチテーブルを生成
  generateDispatchTables(): string {
    const lines: string[] = []

    lines.push("// Struct method and operator dispatch tables")

    // 空のディスパッチテーブルを先に定義
    lines.push(
      "let __structMethods: Record<string, Record<string, Function>> = {};"
    )
    lines.push(
      "let __structOperators: Record<string, Record<string, Function>> = {};"
    )
    lines.push("")

    // ディスパッチヘルパー関数を定義
    lines.push("// Method dispatch helper")
    lines.push(
      "function __dispatchMethod(obj: any, methodName: string, ...args: any[]): any {"
    )
    lines.push("  // 構造体のフィールドアクセスの場合は直接返す")
    lines.push("  if (args.length === 0 && obj.hasOwnProperty(methodName)) {")
    lines.push("    return obj[methodName];")
    lines.push("  }")
    lines.push("  const structName = obj.constructor.name;")
    lines.push("  const structMethods = __structMethods[structName];")
    lines.push("  if (structMethods && structMethods[methodName]) {")
    lines.push("    return structMethods[methodName](obj, ...args);")
    lines.push("  }")
    lines.push(
      `  throw new Error(\`Method '\${methodName}' not found for struct '\${structName}'\`);`
    )
    lines.push("}")
    lines.push("")

    lines.push("// Operator dispatch helper")
    lines.push(
      "function __dispatchOperator(left: any, operator: string, right: any): any {"
    )
    lines.push("  const structName = left.constructor.name;")
    lines.push("  const structOperators = __structOperators[structName];")
    lines.push("  if (structOperators && structOperators[operator]) {")
    lines.push("    return structOperators[operator](left, right);")
    lines.push("  }")
    lines.push("  // Fall back to native JavaScript operator")
    lines.push("  switch (operator) {")
    lines.push("    case '+': return left + right;")
    lines.push("    case '-': return left - right;")
    lines.push("    case '*': return left * right;")
    lines.push("    case '/': return left / right;")
    lines.push("    case '%': return left % right;")
    lines.push("    case '==': return left == right;")
    lines.push("    case '!=': return left != right;")
    lines.push("    case '<': return left < right;")
    lines.push("    case '>': return left > right;")
    lines.push("    case '<=': return left <= right;")
    lines.push("    case '>=': return left >= right;")
    lines.push("    case '&&': return left && right;")
    lines.push("    case '||': return left || right;")
    lines.push(
      `    default: throw new Error(\`Unknown operator: \${operator}\`);`
    )
    lines.push("  }")
    lines.push("}")
    lines.push("")

    return lines.join("\n")
  }

  // ディスパッチテーブル初期化コードを生成
  generateDispatchTableInitialization(): string {
    const lines: string[] = []

    // メソッドディスパッチテーブル初期化
    if (this.structMethods.size > 0) {
      lines.push("// Initialize method dispatch table")
      lines.push("__structMethods = {")
      for (const [structName, methods] of Array.from(this.structMethods)) {
        const methodEntries = Array.from(methods)
          .map((methodName) => {
            const funcName = `__ssrg_${structName}_${this.filePrefix}_${methodName}`
            return `    "${methodName}": ${funcName}`
          })
          .join(",\n")
        lines.push(`  "${structName}": {\n${methodEntries}\n  },`)
      }
      lines.push("};")
      lines.push("")
    }

    // 演算子ディスパッチテーブル初期化
    if (this.structOperators.size > 0) {
      lines.push("// Initialize operator dispatch table")
      lines.push("__structOperators = {")
      for (const [structName, operators] of Array.from(this.structOperators)) {
        const operatorEntries = Array.from(operators)
          .map((op) => {
            const opMethodName = this.operatorToMethodName(op as string)
            const funcName = `__ssrg_${structName}_${this.filePrefix}_op_${opMethodName}`
            return `    "${op}": ${funcName}`
          })
          .join(",\n")
        lines.push(`  "${structName}": {\n${operatorEntries}\n  },`)
      }
      lines.push("};")
      lines.push("")
    }

    return lines.join("\n")
  }

  // メソッド宣言の生成（ファイルハッシュベース命名）
  generateMethodDeclaration(method: MethodDeclaration): string {
    const indent = (this.options.indent || "  ").repeat(this.indentLevel)
    const params = method.parameters
      .map((p) => `${p.name}: ${this.generateType(p.type)}`)
      .join(", ")
    const returnType = this.generateType(method.returnType)
    const body = this.generateExpression(method.body)

    // 構造体名とファイルハッシュベースの一意な名前を生成
    const structPrefix = this.currentStructContext
      ? `${this.currentStructContext}_`
      : ""
    const methodName = `__ssrg_${structPrefix}${this.filePrefix}_${method.name}`

    return `${indent}function ${methodName}(${params}): ${returnType} {
${indent}  return ${body};
${indent}}`
  }

  // 演算子宣言の生成（ファイルハッシュベース命名）
  generateOperatorDeclaration(operator: OperatorDeclaration): string {
    const indent = (this.options.indent || "  ").repeat(this.indentLevel)
    const params = operator.parameters
      .map((p) => `${p.name}: ${this.generateType(p.type)}`)
      .join(", ")
    const returnType = this.generateType(operator.returnType)
    const body = this.generateExpression(operator.body)

    // 演算子名を安全な識別子に変換
    const opMethodName = this.operatorToMethodName(operator.operator)
    const structPrefix = this.currentStructContext
      ? `${this.currentStructContext}_`
      : ""
    const operatorName = `__ssrg_${structPrefix}${this.filePrefix}_op_${opMethodName}`

    return `${indent}function ${operatorName}(${params}): ${returnType} {
${indent}  return ${body};
${indent}}`
  }

  // 演算子をメソッド名に変換
  private operatorToMethodName(op: string): string {
    const opMap: Record<string, string> = {
      "+": "add",
      "-": "sub",
      "*": "mul",
      "/": "div",
      "%": "mod",
      "==": "eq",
      "!=": "ne",
      "<": "lt",
      ">": "gt",
      "<=": "le",
      ">=": "ge",
      "&&": "and",
      "||": "or",
      "!": "not",
    }
    return opMap[op] || op.replace(/[^a-zA-Z0-9]/g, "_")
  }

  // 識別子をサニタイズ（アポストロフィを変換）
  private sanitizeIdentifier(name: string): string {
    // アポストロフィを_primeに変換
    // 例: x' -> x_prime, f'' -> f_prime_prime
    return name.replace(/'/g, "_prime")
  }

  // モノイド宣言の生成
  generateMonoidDeclaration(monoid: MonoidDeclaration): string {
    const indent = (this.options.indent || "  ").repeat(this.indentLevel)
    const lines: string[] = []

    lines.push(`${indent}// Monoid implementation`)
    lines.push(
      `${indent}export const identity = ${this.generateExpression(monoid.identity)};`
    )
    lines.push(this.generateOperatorDeclaration(monoid.operator))

    return lines.join("\n")
  }

  // 式の生成
  generateExpression(expr: Expression): string {
    switch (expr.kind) {
      case "Literal":
        return this.generateLiteral(expr as Literal)
      case "Identifier":
        return this.sanitizeIdentifier((expr as Identifier).name)
      case "TemplateExpression":
        return this.generateTemplateExpression(expr as TemplateExpression)
      case "BinaryOperation":
        return this.generateBinaryOperation(expr as BinaryOperation)
      case "NullishCoalescingExpression":
        return this.generateNullishCoalescing(
          expr as NullishCoalescingExpression
        )
      case "UnaryOperation":
        return this.generateUnaryOperation(expr as UnaryOperation)
      case "FunctionCall":
        return this.generateFunctionCall(expr as FunctionCall)
      case "MethodCall":
        return this.generateMethodCall(expr as MethodCall)
      case "FunctionApplication":
        return this.generateFunctionApplication(expr as FunctionApplication)
      case "BuiltinFunctionCall":
        return this.generateBuiltinFunctionCall(expr as BuiltinFunctionCall)
      case "ConditionalExpression":
        return this.generateConditionalExpression(expr as ConditionalExpression)
      case "TernaryExpression":
        return this.generateTernaryExpression(expr as TernaryExpression)
      case "MatchExpression":
        return this.generateMatchExpression(expr as MatchExpression)
      case "Pipeline":
        return this.generatePipeline(expr as Pipeline)
      case "ReversePipe":
        return this.generateReversePipe(expr as ReversePipe)
      case "FunctorMap":
        return this.generateFunctorMap(expr as FunctorMap)
      case "ApplicativeApply":
        return this.generateApplicativeApply(expr as ApplicativeApply)
      case "MonadBind":
        return this.generateMonadBind(expr as MonadBind)
      case "FoldMonoid":
        return this.generateFoldMonoid(expr as FoldMonoid)
      case "FunctionApplicationOperator":
        return this.generateFunctionApplicationOperator(
          expr as FunctionApplicationOperator
        )
      case "ConstructorExpression":
        return this.generateConstructorExpression(expr as ConstructorExpression)
      case "BlockExpression":
        return this.generateBlockExpression(expr as BlockExpression)
      case "LambdaExpression":
        return this.generateLambdaExpression(expr as LambdaExpression)
      case "RecordExpression":
        return this.generateRecordExpression(expr as RecordExpression)
      case "RecordAccess":
        return this.generateRecordAccess(expr as RecordAccess)
      case "ArrayLiteral":
        return this.generateArrayLiteral(expr as ArrayLiteral)
      case "ArrayAccess":
        return this.generateArrayAccess(expr as ArrayAccess)
      case "ListSugar":
        return this.generateListSugar(expr as ListSugar)
      case "ConsExpression":
        return this.generateConsExpression(expr as ConsExpression)
      case "RangeLiteral":
        return this.generateRangeLiteral(expr as RangeLiteral)
      case "ListComprehension":
        return this.generateListComprehension(expr as ListComprehension)
      case "ListComprehensionSugar":
        return this.generateListComprehensionSugar(
          expr as ListComprehensionSugar
        )
      case "TupleExpression":
        return this.generateTupleExpression(expr as TupleExpression)
      case "StructExpression":
        return this.generateStructExpression(expr as StructExpression)
      case "SpreadExpression":
        return this.generateSpreadExpression(expr as SpreadExpression)
      case "TypeAssertion":
        return this.generateTypeAssertion(expr as TypeAssertion)
      default:
        return `/* Unsupported expression: ${expr.constructor.name} */`
    }
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

    // 解決済みの型を取得
    const leftType = this.getResolvedType(binOp.left)
    const rightType = this.getResolvedType(binOp.right)

    // 両辺がプリミブ型の場合は直接演算子を使用
    if (
      this.isBasicOperator(binOp.operator) &&
      this.isPrimitiveType(leftType) &&
      this.isPrimitiveType(rightType)
    ) {
      let operator = binOp.operator
      if (operator === "==") operator = "==="
      if (operator === "!=") operator = "!=="

      // Int/Int除算の特別処理 - Math.trunc()で切り捨て
      if (
        operator === "/" &&
        this.isIntType(leftType) &&
        this.isIntType(rightType)
      ) {
        return `Math.trunc(${left} / ${right})`
      }

      return `(${left} ${operator} ${right})`
    }

    // 構造体の演算子オーバーロードの可能性がある場合は演算子ディスパッチを使用
    return this.generateOperatorDispatch(binOp.operator, left, right)
  }

  // Nullish Coalescing演算子の生成
  generateNullishCoalescing(
    nullishCoalescing: NullishCoalescingExpression
  ): string {
    const left = this.generateExpression(nullishCoalescing.left)
    const right = this.generateExpression(nullishCoalescing.right)

    // 左辺の型を取得して適切なランタイム関数を選択
    const leftType = this.getResolvedType(nullishCoalescing.left)
    console.log(`[DEBUG] Nullish coalescing left type:`, leftType)

    // デバッグ: 左辺の型情報を出力
    if (nullishCoalescing.left.kind === "FunctionApplication") {
      console.log("[DEBUG] FunctionApplication left type:", leftType)
      console.log("[DEBUG] Left expr kind:", nullishCoalescing.left.kind)
      console.log(
        "[DEBUG] Left expr type from expr:",
        nullishCoalescing.left.type
      )
      if (this.typeInferenceResult?.nodeTypeMap) {
        const mappedType = this.typeInferenceResult.nodeTypeMap.get(
          nullishCoalescing.left
        )
        console.log("[DEBUG] Type from nodeTypeMap:", mappedType)

        // 関数名を取得
        const funcApp = nullishCoalescing.left as FunctionApplication
        if (funcApp.function.kind === "Identifier") {
          const funcName = (funcApp.function as Identifier).name
          console.log("[DEBUG] Function name:", funcName)
          console.log("[DEBUG] Function expr type:", funcApp.function.type)
          const funcType = this.typeInferenceResult.nodeTypeMap.get(
            funcApp.function
          )
          console.log("[DEBUG] Function type from nodeTypeMap:", funcType)
        }
      }
    }

    if (this.isMaybeType(leftType)) {
      const rightType = this.getResolvedType(nullishCoalescing.right)

      // 右辺もMaybe型の場合: 特別な処理が必要
      if (this.isMaybeType(rightType)) {
        // Maybe<T> ?? Maybe<U> の場合、左辺がJustなら左辺の値、そうでなければ右辺の値
        return `(${left}.tag === 'Just' ? ${left}.value : (${right}.tag === 'Just' ? ${right}.value : undefined))`
      }

      // Maybe型の場合: fromMaybe(defaultValue, maybe)
      return `fromMaybe(${right}, ${left})`
    } else if (this.isEitherType(leftType)) {
      const rightType = this.getResolvedType(nullishCoalescing.right)

      // 右辺もEither型の場合: 特別な処理が必要
      if (this.isEitherType(rightType)) {
        // Either<L, R> ?? Either<L2, R2> の場合、左辺がRightなら左辺の値、そうでなければ右辺の値
        return `(${left}.tag === 'Right' ? ${left}.value : (${right}.tag === 'Right' ? ${right}.value : undefined))`
      }

      // Either型の場合: fromRight(defaultValue, either)
      return `fromRight(${right}, ${left})`
    } else {
      // その他の場合: TypeScriptのnull合体演算子を使用
      return `(${left} ?? ${right})`
    }
  }

  // 基本演算子かどうかをチェック
  private isBasicOperator(op: string): boolean {
    // プリミティブ型で直接使用できる演算子
    const basicOps = [
      "+",
      "-",
      "*",
      "/",
      "%",
      "**",
      "==",
      "!=",
      "<",
      ">",
      "<=",
      ">=",
      "&&",
      "||",
    ]
    return basicOps.includes(op)
  }

  // 型推論結果から解決済みの型を取得
  private getResolvedType(expr: Expression): Type | undefined {
    if (this.typeInferenceResult?.nodeTypeMap) {
      const resolvedType = this.typeInferenceResult.nodeTypeMap.get(expr)
      if (resolvedType) {
        return resolvedType
      }
    }
    return expr.type
  }

  // プリミティブ型かどうかをチェック
  private isPrimitiveType(type: Type | undefined): boolean {
    if (!type || type.kind !== "PrimitiveType") {
      return false
    }
    const primitiveTypes = ["Int", "Float", "Bool", "String", "Char", "Unit"]
    return primitiveTypes.includes((type as PrimitiveType).name)
  }

  // Int型かどうかをチェック
  private isIntType(type: Type | undefined): boolean {
    return (
      type?.kind === "PrimitiveType" && (type as PrimitiveType).name === "Int"
    )
  }

  // 演算子ディスパッチを使用しているかチェック
  private checkUsesOperatorDispatch(statements: Statement[]): boolean {
    for (const stmt of statements) {
      if (this.checkStatementUsesOperatorDispatch(stmt)) {
        return true
      }
    }
    return false
  }

  // BinaryOperationが存在するかチェック
  private hasBinaryOperations(statements: Statement[]): boolean {
    for (const stmt of statements) {
      if (this.statementHasBinaryOperations(stmt)) {
        return true
      }
    }
    return false
  }

  private statementHasBinaryOperations(stmt: Statement): boolean {
    if (stmt instanceof ExpressionStatement) {
      return this.expressionHasBinaryOperations(stmt.expression)
    } else if (stmt instanceof VariableDeclaration && stmt.initializer) {
      return this.expressionHasBinaryOperations(stmt.initializer)
    } else if (stmt instanceof FunctionDeclaration && stmt.body) {
      return this.expressionHasBinaryOperations(stmt.body)
    }
    return false
  }

  private expressionHasBinaryOperations(expr: Expression): boolean {
    switch (expr.kind) {
      case "BinaryOperation":
        return true

      case "BlockExpression":
        return this.blockExpressionHasBinaryOperations(expr as BlockExpression)

      case "ListComprehension":
        return this.listComprehensionHasBinaryOperations(
          expr as ListComprehension
        )

      case "FunctionCall":
        return this.functionCallHasBinaryOperations(expr as FunctionCall)

      case "LambdaExpression":
        return this.expressionHasBinaryOperations(
          (expr as LambdaExpression).body
        )

      case "FunctorMap":
        return this.expressionHasBinaryOperations((expr as FunctorMap).left)

      case "MonadBind":
        return this.expressionHasBinaryOperations((expr as MonadBind).left)

      case "ApplicativeApply":
        return this.applicativeApplyHasBinaryOperations(
          expr as ApplicativeApply
        )

      case "FunctionApplication":
        return this.functionApplicationHasBinaryOperations(
          expr as FunctionApplication
        )

      default:
        return false
    }
  }

  private blockExpressionHasBinaryOperations(
    blockExpr: BlockExpression
  ): boolean {
    for (const stmt of blockExpr.statements) {
      if (this.statementHasBinaryOperations(stmt)) {
        return true
      }
    }
    if (blockExpr.returnExpression) {
      return this.expressionHasBinaryOperations(blockExpr.returnExpression)
    }
    return false
  }

  private listComprehensionHasBinaryOperations(
    listComp: ListComprehension
  ): boolean {
    if (this.expressionHasBinaryOperations(listComp.expression)) {
      return true
    }
    for (const filter of listComp.filters || []) {
      if (this.expressionHasBinaryOperations(filter)) {
        return true
      }
    }
    return false
  }

  private functionCallHasBinaryOperations(funcCall: FunctionCall): boolean {
    for (const arg of funcCall.arguments) {
      if (this.expressionHasBinaryOperations(arg)) {
        return true
      }
    }
    return false
  }

  private applicativeApplyHasBinaryOperations(
    applyExpr: ApplicativeApply
  ): boolean {
    if (this.expressionHasBinaryOperations(applyExpr.left)) {
      return true
    }
    return this.expressionHasBinaryOperations(applyExpr.right)
  }

  private functionApplicationHasBinaryOperations(
    appExpr: FunctionApplication
  ): boolean {
    if (this.expressionHasBinaryOperations(appExpr.function)) {
      return true
    }
    return this.expressionHasBinaryOperations(appExpr.argument)
  }

  // __dispatchOperatorの使用があるかチェック
  private hasDispatchOperatorUsage(statements: Statement[]): boolean {
    // 簡単なアプローチ: 生成されるコードに __dispatchOperator が含まれるかチェック
    for (const stmt of statements) {
      const code = this.generateStatement(stmt)
      if (code.includes("__dispatchOperator")) {
        return true
      }
    }
    return false
  }

  private statementHasListComprehensions(stmt: Statement): boolean {
    if (stmt instanceof ExpressionStatement) {
      return this.expressionHasListComprehensions(stmt.expression)
    } else if (stmt instanceof VariableDeclaration && stmt.initializer) {
      return this.expressionHasListComprehensions(stmt.initializer)
    } else if (stmt instanceof FunctionDeclaration && stmt.body) {
      return this.expressionHasListComprehensions(stmt.body)
    }
    return false
  }

  private expressionHasListComprehensions(expr: Expression): boolean {
    if (expr instanceof ListComprehension) {
      return true
    } else if (expr instanceof BlockExpression) {
      for (const stmt of expr.statements) {
        if (this.statementHasListComprehensions(stmt)) {
          return true
        }
      }
      if (expr.returnExpression) {
        return this.expressionHasListComprehensions(expr.returnExpression)
      }
    } else if (expr instanceof FunctionCall) {
      for (const arg of expr.arguments) {
        if (this.expressionHasListComprehensions(arg)) {
          return true
        }
      }
    }
    return false
  }

  private checkStatementUsesOperatorDispatch(stmt: Statement): boolean {
    if (stmt instanceof ExpressionStatement) {
      return this.checkExpressionUsesOperatorDispatch(stmt.expression)
    } else if (stmt instanceof VariableDeclaration && stmt.initializer) {
      return this.checkExpressionUsesOperatorDispatch(stmt.initializer)
    } else if (stmt instanceof FunctionDeclaration && stmt.body) {
      return this.checkExpressionUsesOperatorDispatch(stmt.body)
    }
    return false
  }

  private checkExpressionUsesOperatorDispatch(expr: Expression): boolean {
    switch (expr.kind) {
      case "BinaryOperation":
        return this.binaryOperationUsesDispatch(expr as BinaryOperation)

      case "BlockExpression":
        return this.blockExpressionUsesDispatch(expr as BlockExpression)

      case "ListComprehension":
        return this.listComprehensionUsesDispatch(expr as ListComprehension)

      case "FunctionCall":
        return this.functionCallUsesDispatch(expr as FunctionCall)

      default:
        return false
    }
  }

  private binaryOperationUsesDispatch(binOp: BinaryOperation): boolean {
    // プリミティブ型同士の演算ではない場合のみ__dispatchOperatorを使用
    if (
      !this.isBasicOperator(binOp.operator) ||
      !this.isPrimitiveType(binOp.left.type) ||
      !this.isPrimitiveType(binOp.right.type)
    ) {
      return true
    }
    return false
  }

  private blockExpressionUsesDispatch(blockExpr: BlockExpression): boolean {
    for (const stmt of blockExpr.statements) {
      if (this.checkStatementUsesOperatorDispatch(stmt)) {
        return true
      }
    }
    if (blockExpr.returnExpression) {
      return this.checkExpressionUsesOperatorDispatch(
        blockExpr.returnExpression
      )
    }
    return false
  }

  private listComprehensionUsesDispatch(listComp: ListComprehension): boolean {
    if (this.checkExpressionUsesOperatorDispatch(listComp.expression)) {
      return true
    }
    for (const filter of listComp.filters || []) {
      if (this.checkExpressionUsesOperatorDispatch(filter)) {
        return true
      }
    }
    return false
  }

  private functionCallUsesDispatch(funcCall: FunctionCall): boolean {
    for (const arg of funcCall.arguments) {
      if (this.checkExpressionUsesOperatorDispatch(arg)) {
        return true
      }
    }
    return false
  }

  // 演算子ディスパッチの生成
  private generateOperatorDispatch(
    operator: string,
    left: string,
    right: string
  ): string {
    // ディスパッチテーブルを使用した演算子呼び出し
    return `__dispatchOperator(${left}, "${operator}", ${right})`
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

    // 型引数がある場合の処理
    if (call.typeArguments && call.typeArguments.length > 0) {
      const typeArgs = call.typeArguments
        .map((type) => this.generateType(type))
        .join(", ")
      return `${func}<${typeArgs}>(${args.join(", ")})`
    }

    return `${func}(${args.join(", ")})`
  }

  // 関数適用の生成
  generateFunctionApplication(app: FunctionApplication): string {
    const func = this.generateExpression(app.function)
    const arg = this.generateExpression(app.argument)

    // ビルトイン関数の特別処理
    const builtinResult = this.tryGenerateBuiltinApplication(app, arg)
    if (builtinResult) {
      return builtinResult
    }

    // ネストした関数適用の処理
    const nestedResult = this.tryGenerateNestedApplication(app, arg)
    if (nestedResult) {
      return nestedResult
    }

    // 通常の関数適用
    return this.generateRegularApplication(app, func, arg)
  }

  private tryGenerateBuiltinApplication(
    app: FunctionApplication,
    arg: string
  ): string | null {
    if (app.function instanceof Identifier) {
      const funcName = app.function.name
      const builtinMap: Record<string, string> = {
        print: `console.log(${arg})`,
        putStrLn: `console.log(${arg})`,
        toString: `toString(${arg})`,
        toInt: `toInt(${arg})`,
        toFloat: `toFloat(${arg})`,
        head: `headList(${arg})`,
        tail: `tailList(${arg})`,
        show: `show(${arg})`,
      }
      return builtinMap[funcName] || null
    }
    return null
  }

  private tryGenerateNestedApplication(
    app: FunctionApplication,
    arg: string
  ): string | null {
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
    return null
  }

  private generateRegularApplication(
    app: FunctionApplication,
    func: string,
    arg: string
  ): string {
    // wrap lambda expressions in parentheses
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
      case "toInt":
        if (args.length !== 1) {
          throw new Error("toInt requires exactly one argument")
        }
        return `toInt(${args[0]})`
      case "toFloat":
        if (args.length !== 1) {
          throw new Error("toFloat requires exactly one argument")
        }
        return `toFloat(${args[0]})`
      case "head":
        if (args.length !== 1) {
          throw new Error("head requires exactly one argument")
        }
        return `headList(${args[0]})`
      case "tail":
        if (args.length !== 1) {
          throw new Error("tail requires exactly one argument")
        }
        return `tailList(${args[0]})`
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

  // 三項演算子の生成
  generateTernaryExpression(ternary: TernaryExpression): string {
    const condition = this.generateExpression(ternary.condition)
    const trueBranch = this.generateExpression(ternary.trueExpression)
    const falseBranch = this.generateExpression(ternary.falseExpression)

    return `(${condition} ? ${trueBranch} : ${falseBranch})`
  }

  // マッチ式の生成
  generateMatchExpression(match: MatchExpression): string {
    const expr = this.generateExpression(match.expression)

    // if-else チェーンとして生成（柔軟性を向上）
    const cases = match.cases
    let result = `(() => {\n  const matchValue = ${expr};\n`

    for (let i = 0; i < cases.length; i++) {
      const c = cases[i]
      if (!c) continue

      // GuardPatternの場合は特別な処理が必要
      if (c.pattern.kind === "GuardPattern") {
        const guardPattern = c.pattern as GuardPattern
        const baseCondition = this.generatePatternCondition(
          guardPattern.pattern,
          "matchValue"
        )
        const bindings = this.generatePatternBindings(
          guardPattern.pattern,
          "matchValue"
        )
        const guardCondition = this.generateExpression(guardPattern.guard)
        const body = this.generateExpression(c.expression)

        // GuardPatternでは常にifを使用（else ifではなく）
        // これにより、ガード条件が失敗したときに次のパターンへ続行できる
        result += `  if (${baseCondition}) {\n    ${bindings}if (${guardCondition}) {\n      return ${body};\n    }\n  }`
      } else {
        const condition = this.generatePatternCondition(c.pattern, "matchValue")
        const bindings = this.generatePatternBindings(c.pattern, "matchValue")
        const body = this.generateExpression(c.expression)

        // 通常のパターンは常に'if'を使用（ガードパターンと同様の連続的チェック）
        result += `  if (${condition}) {\n    ${bindings}return ${body};\n  }`
      }
    }

    result +=
      " else {\n    throw new Error('Non-exhaustive pattern match');\n  }\n})()"
    return result
  }

  // パターン条件の生成（if-else用）
  generatePatternCondition(pattern: Pattern, valueVar: string): string {
    if (!pattern) {
      return "true" // ワイルドカードパターン
    }

    // ASTパターンの種類に基づいて処理
    switch (pattern.kind) {
      case "LiteralPattern":
        return this.generateLiteralPatternCondition(
          pattern as LiteralPattern,
          valueVar
        )
      case "IdentifierPattern":
        return this.generateIdentifierPatternCondition(
          pattern as IdentifierPattern,
          valueVar
        )
      case "ConstructorPattern":
        return this.generateConstructorPatternCondition(
          pattern as ConstructorPattern,
          valueVar
        )
      case "TuplePattern":
        return this.generateTuplePatternCondition(
          pattern as TuplePattern,
          valueVar
        )
      case "WildcardPattern":
        return "true" // ワイルドカードパターン
      case "OrPattern":
        return this.generateOrPatternCondition(pattern as OrPattern, valueVar)

      case "GuardPattern":
        return this.generateGuardPatternCondition(
          pattern as GuardPattern,
          valueVar
        )
      case "ListSugarPattern":
        return this.generateListSugarPatternCondition(
          pattern as ListSugarPattern,
          valueVar
        )
      case "ArrayPattern":
        return this.generateArrayPatternCondition(
          pattern as ArrayPattern,
          valueVar
        )

      default:
        // 後方互換性のための古い形式をチェック
        if (pattern.kind === "LiteralPattern") {
          const literalPattern = pattern as LiteralPattern
          if (typeof literalPattern.value === "string") {
            return `${valueVar} === ${JSON.stringify(literalPattern.value)}`
          } else {
            return `${valueVar} === ${literalPattern.value}`
          }
        }

        if (pattern.kind === "IdentifierPattern") {
          const identifierPattern = pattern as IdentifierPattern
          if (identifierPattern.name === "_") {
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

  // リテラルパターンの条件生成
  private generateLiteralPatternCondition(
    pattern: LiteralPattern,
    valueVar: string
  ): string {
    if (typeof pattern.value === "string") {
      return `${valueVar} === ${JSON.stringify(pattern.value)}`
    } else {
      return `${valueVar} === ${pattern.value}`
    }
  }

  // 識別子パターンの条件生成
  private generateIdentifierPatternCondition(
    pattern: IdentifierPattern,
    _valueVar: string
  ): string {
    if (pattern.name === "_") {
      return "true" // ワイルドカードパターン
    }
    // 変数にバインドする場合（常に true）
    return "true"
  }

  // コンストラクタパターンの条件生成
  private generateConstructorPatternCondition(
    pattern: ConstructorPattern,
    valueVar: string
  ): string {
    const constructorCondition = this.generateConstructorCondition(
      pattern,
      valueVar
    )
    const subConditions = this.generateSubPatternConditions(pattern, valueVar)

    if (subConditions.length > 0) {
      return `${constructorCondition} && ${subConditions.join(" && ")}`
    }

    return constructorCondition
  }

  // コンストラクタ条件の生成
  private generateConstructorCondition(
    pattern: ConstructorPattern,
    valueVar: string
  ): string {
    if (this.isBuiltinConstructor(pattern.constructorName)) {
      return `${valueVar}.tag === '${pattern.constructorName}'`
    } else {
      return `${valueVar}.type === '${pattern.constructorName}'`
    }
  }

  // サブパターン条件の生成
  private generateSubPatternConditions(
    pattern: ConstructorPattern,
    valueVar: string
  ): string[] {
    if (!pattern.patterns || pattern.patterns.length === 0) {
      return []
    }

    const subConditions: string[] = []

    for (let i = 0; i < pattern.patterns.length; i++) {
      const subPattern = pattern.patterns[i]
      if (subPattern.kind === "LiteralPattern") {
        const valueAccess = this.generateValueAccess(pattern, valueVar, i)
        const condition = this.generateLiteralCondition(
          subPattern as LiteralPattern,
          valueAccess
        )
        subConditions.push(condition)
      }
      // IdentifierPatternの場合は常にtrue（バインディングのみ）
    }

    return subConditions
  }

  // 値アクセス文字列の生成
  private generateValueAccess(
    pattern: ConstructorPattern,
    valueVar: string,
    index: number
  ): string {
    if (this.isBuiltinConstructor(pattern.constructorName)) {
      return this.generateBuiltinValueAccess(
        pattern.constructorName,
        valueVar,
        index
      )
    } else {
      return `${valueVar}.data[${index}]`
    }
  }

  // ビルトイン型の値アクセス
  private generateBuiltinValueAccess(
    constructorName: string,
    valueVar: string,
    index: number
  ): string {
    if (
      constructorName === "Just" ||
      constructorName === "Left" ||
      constructorName === "Right"
    ) {
      return `${valueVar}.value`
    } else if (constructorName === "Cons") {
      return index === 0 ? `${valueVar}.head` : `${valueVar}.tail`
    } else {
      return `${valueVar}.data[${index}]`
    }
  }

  // リテラル条件の生成
  private generateLiteralCondition(
    pattern: LiteralPattern,
    valueAccess: string
  ): string {
    if (typeof pattern.value === "string") {
      return `${valueAccess} === ${JSON.stringify(pattern.value)}`
    } else {
      return `${valueAccess} === ${pattern.value}`
    }
  }

  // タプルパターンの条件生成
  private generateTuplePatternCondition(
    pattern: TuplePattern,
    valueVar: string
  ): string {
    const tupleConditions = pattern.patterns.map((subPattern, i) => {
      return this.generatePatternCondition(
        subPattern,
        `${valueVar}.elements[${i}]`
      )
    })
    return tupleConditions.join(" && ")
  }

  // Orパターンの条件生成
  private generateOrPatternCondition(
    pattern: OrPattern,
    valueVar: string
  ): string {
    const orConditions = pattern.patterns.map((subPattern: Pattern) => {
      return this.generatePatternCondition(subPattern, valueVar)
    })
    return `(${orConditions.join(" || ")})`
  }

  // ガードパターンの条件生成
  private generateGuardPatternCondition(
    pattern: GuardPattern,
    valueVar: string
  ): string {
    const patternCondition = this.generatePatternCondition(
      pattern.pattern,
      valueVar
    )
    const guardCondition = this.generateExpression(pattern.guard)
    return `(${patternCondition} && (${guardCondition}))`
  }

  // リスト糖衣構文パターンの条件生成
  private generateListSugarPatternCondition(
    pattern: ListSugarPattern,
    valueVar: string
  ): string {
    // 空リストパターン `[]
    if (pattern.patterns.length === 0 && !pattern.restPattern) {
      return `${valueVar}.tag === 'Empty'`
    }

    // restのみのパターン `[...rest]
    if (pattern.patterns.length === 0 && pattern.restPattern) {
      return "true" // すべてのリストにマッチ
    }

    // パターンを構築
    const conditions: string[] = []
    let currentVar = valueVar

    // 各要素パターンをチェック
    for (let i = 0; i < pattern.patterns.length; i++) {
      conditions.push(`${currentVar}.tag === 'Cons'`)
      currentVar = `${currentVar}.tail`
    }

    // restパターンがない場合、残りはEmptyである必要がある
    if (!pattern.restPattern) {
      conditions.push(`${currentVar}.tag === 'Empty'`)
    }

    return `(${conditions.join(" && ")})`
  }

  // 配列パターンの条件生成
  private generateArrayPatternCondition(
    pattern: ArrayPattern,
    valueVar: string
  ): string {
    // 空配列パターン []
    if (pattern.patterns.length === 0 && !pattern.restPattern) {
      return `${valueVar}.length === 0`
    }

    // restのみのパターン [...rest]
    if (pattern.patterns.length === 0 && pattern.restPattern) {
      return "true" // すべての配列にマッチ
    }

    const conditions: string[] = []

    // 要素数チェック
    if (!pattern.restPattern) {
      // restがない場合、正確な長さを要求
      conditions.push(`${valueVar}.length === ${pattern.patterns.length}`)
    } else {
      // restがある場合、最小長さを要求
      conditions.push(`${valueVar}.length >= ${pattern.patterns.length}`)
    }

    return `(${conditions.join(" && ")})`
  }

  // 識別子パターンのバインディング生成
  private generateIdentifierPatternBindings(
    pattern: IdentifierPattern,
    valueVar: string
  ): string {
    return `const ${pattern.name} = ${valueVar};\n    `
  }

  // コンストラクタパターンのバインディング生成
  private generateConstructorPatternBindings(
    pattern: ConstructorPattern,
    valueVar: string
  ): string {
    if (!pattern.patterns || pattern.patterns.length === 0) {
      return ""
    }

    if (this.isBuiltinConstructor(pattern.constructorName)) {
      return this.generateBuiltinConstructorBindings(pattern, valueVar)
    } else {
      return this.generateUserDefinedConstructorBindings(pattern, valueVar)
    }
  }

  // ビルトインコンストラクタバインディング生成
  private generateBuiltinConstructorBindings(
    pattern: ConstructorPattern,
    valueVar: string
  ): string {
    if (this.isSingleValueConstructor(pattern.constructorName)) {
      return this.generateSingleValueConstructorBindings(pattern, valueVar)
    } else if (pattern.constructorName === "Cons") {
      return this.generateConsConstructorBindings(pattern, valueVar)
    }
    return ""
  }

  // 単一値コンストラクタのバインディング
  private generateSingleValueConstructorBindings(
    pattern: ConstructorPattern,
    valueVar: string
  ): string {
    if (pattern.patterns.length > 0) {
      const subPattern = pattern.patterns[0]
      if (subPattern.kind === "IdentifierPattern") {
        const identifierPattern = subPattern as IdentifierPattern
        return `const ${identifierPattern.name} = ${valueVar}.value;\n    `
      }
    }
    return ""
  }

  // Consコンストラクタのバインディング
  private generateConsConstructorBindings(
    pattern: ConstructorPattern,
    valueVar: string
  ): string {
    let bindings = ""

    if (pattern.patterns.length > 0) {
      const headPattern = pattern.patterns[0]
      if (headPattern.kind === "IdentifierPattern") {
        const identifierPattern = headPattern as IdentifierPattern
        bindings += `const ${identifierPattern.name} = ${valueVar}.head;\n    `
      }
    }

    if (pattern.patterns.length > 1) {
      const tailPattern = pattern.patterns[1]
      if (tailPattern.kind === "IdentifierPattern") {
        const identifierPattern = tailPattern as IdentifierPattern
        bindings += `const ${identifierPattern.name} = ${valueVar}.tail;\n    `
      }
    }

    return bindings
  }

  // ユーザー定義コンストラクタバインディング生成
  private generateUserDefinedConstructorBindings(
    pattern: ConstructorPattern,
    valueVar: string
  ): string {
    let bindings = ""

    for (let i = 0; i < pattern.patterns.length; i++) {
      const subPattern = pattern.patterns[i]
      if (subPattern.kind === "IdentifierPattern") {
        const identifierPattern = subPattern as IdentifierPattern
        bindings += `const ${identifierPattern.name} = ${valueVar}.data[${i}];\n    `
      }
    }

    return bindings
  }

  // 単一値コンストラクタの判定
  private isSingleValueConstructor(constructorName: string): boolean {
    return (
      constructorName === "Just" ||
      constructorName === "Left" ||
      constructorName === "Right"
    )
  }

  // タプルパターンのバインディング生成
  private generateTuplePatternBindings(
    pattern: TuplePattern,
    valueVar: string
  ): string {
    let tupleBindings = ""
    pattern.patterns.forEach((subPattern: Pattern, i: number) => {
      tupleBindings += this.generatePatternBindings(
        subPattern,
        `${valueVar}.elements[${i}]`
      )
    })
    return tupleBindings
  }

  // Orパターンのバインディング生成
  private generateOrPatternBindings(
    pattern: OrPattern,
    valueVar: string
  ): string {
    if (pattern.patterns.length > 0) {
      // 最初のパターンからバインディングを生成
      // 実際のマッチングは条件で制御される
      return pattern.patterns[0]
        ? this.generatePatternBindings(pattern.patterns[0], valueVar)
        : ""
    }
    return ""
  }

  // ガードパターンのバインディング生成
  private generateGuardPatternBindings(
    pattern: GuardPattern,
    valueVar: string
  ): string {
    return this.generatePatternBindings(pattern.pattern, valueVar)
  }

  // リスト糖衣構文パターンのバインディング生成
  private generateListSugarPatternBindings(
    pattern: ListSugarPattern,
    valueVar: string
  ): string {
    let bindings = ""
    let currentVar = valueVar

    // 各要素パターンのバインディング
    for (let i = 0; i < pattern.patterns.length; i++) {
      const elemPattern = pattern.patterns[i]
      const headVar = `${currentVar}.head`
      bindings += elemPattern
        ? this.generatePatternBindings(elemPattern, headVar)
        : ""
      currentVar = `${currentVar}.tail`
    }

    // restパターンのバインディング
    if (pattern.restPattern) {
      bindings += this.generatePatternBindings(pattern.restPattern, currentVar)
    }

    return bindings
  }

  // 配列パターンのバインディング生成
  private generateArrayPatternBindings(
    pattern: ArrayPattern,
    valueVar: string
  ): string {
    let bindings = ""

    // 各要素パターンのバインディング
    for (let i = 0; i < pattern.patterns.length; i++) {
      const elemPattern = pattern.patterns[i]
      const indexVar = `${valueVar}[${i}]`
      bindings += elemPattern
        ? this.generatePatternBindings(elemPattern, indexVar)
        : ""
    }

    // restパターンのバインディング
    if (pattern.restPattern) {
      const sliceVar = `${valueVar}.slice(${pattern.patterns.length})`
      bindings += this.generatePatternBindings(pattern.restPattern, sliceVar)
    }

    return bindings
  }

  // パターンの生成（旧メソッド、下位互換性のため保持）
  generatePattern(pattern: Pattern): string {
    // 簡易実装：リテラルパターンのみサポート
    if (pattern.kind === "LiteralPattern") {
      const literalPattern = pattern as LiteralPattern
      return JSON.stringify(literalPattern.value)
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

    // 型に基づいて適切なランタイム関数を選択
    // 実際の型判定はランタイムで行う
    return `(() => {
      const _value = ${value};
      if (_value && _value.tag === 'Tuple') {
        return { tag: 'Tuple', elements: mapArray(_value.elements, ${func}) };
      } else if (Array.isArray(_value)) {
        return mapArray(_value, ${func});
      } else if (_value && _value.tag === 'Cons' || _value && _value.tag === 'Empty') {
        return mapList(_value, ${func});
      } else if (_value && (_value.tag === 'Left' || _value.tag === 'Right')) {
        return mapEither(_value, ${func});
      } else {
        return mapMaybe(_value, ${func});
      }
    })()`
  }

  // アプリカティブ適用の生成
  generateApplicativeApply(apply: ApplicativeApply): string {
    const funcContainer = this.generateExpression(apply.left)
    const valueContainer = this.generateExpression(apply.right)

    // 型に基づいて適切なランタイム関数を選択
    return `(() => {
      const _funcs = ${funcContainer};
      const _values = ${valueContainer};
      if (Array.isArray(_funcs) && Array.isArray(_values)) {
        return applyArray(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Cons' || _funcs.tag === 'Empty') &&
                _values && (_values.tag === 'Cons' || _values.tag === 'Empty')) {
        return applyList(_funcs, _values);
      } else if (_funcs && (_funcs.tag === 'Left' || _funcs.tag === 'Right') &&
                _values && (_values.tag === 'Left' || _values.tag === 'Right')) {
        return applyEither(_funcs, _values);
      } else {
        return applyMaybe(_funcs, _values);
      }
    })()`
  }

  // モナドバインドの生成
  generateMonadBind(bind: MonadBind): string {
    const monadValue = this.generateExpression(bind.left)
    const bindFunc = this.generateExpression(bind.right)

    // 型に基づいて適切なランタイム関数を選択
    return `(() => {
      const _monad = ${monadValue};
      if (_monad && _monad.tag === 'Tuple') {
        const results = bindArray(_monad.elements, ${bindFunc});
        return { tag: 'Tuple', elements: results };
      } else if (Array.isArray(_monad)) {
        return bindArray(_monad, ${bindFunc});
      } else if (_monad && (_monad.tag === 'Cons' || _monad.tag === 'Empty')) {
        return bindList(_monad, ${bindFunc});
      } else if (_monad && (_monad.tag === 'Left' || _monad.tag === 'Right')) {
        return bindEither(_monad, ${bindFunc});
      } else {
        return bindMaybe(_monad, ${bindFunc});
      }
    })()`
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
          return "any" // Placeholder type for inference
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
    } else if (type instanceof TupleType) {
      if (type.elementTypes.length === 0) {
        return "[]"
      }
      const elements = type.elementTypes
        .map((elementType) => this.generateType(elementType))
        .join(", ")
      return `[${elements}]`
    } else if (type instanceof StructType) {
      return type.name
    } else if (type instanceof UnionType) {
      const types = type.types
        .map((t: Type) => this.generateType(t))
        .join(" | ")
      return `(${types})`
    } else if (type instanceof IntersectionType) {
      const types = type.types
        .map((t: Type) => this.generateType(t))
        .join(" & ")
      return `(${types})`
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

    const fieldStrings = record.fields.map((field) => {
      if (field.kind === "RecordInitField") {
        const initField = field as RecordInitField
        const value = this.generateExpression(initField.value)
        return `${initField.name}: ${value}`
      } else if (field.kind === "RecordShorthandField") {
        const shorthandField = field as RecordShorthandField
        // JavaScript/TypeScript shorthand property notation
        return shorthandField.name
      } else if (field.kind === "RecordSpreadField") {
        const spreadField = field as RecordSpreadField
        const spreadValue = this.generateExpression(
          spreadField.spreadExpression.expression
        )
        return `...${spreadValue}`
      }
      return ""
    })

    return `{ ${fieldStrings.join(", ")} }`
  }

  // Recordアクセスの生成
  generateRecordAccess(access: RecordAccess): string {
    const record = this.generateExpression(access.record)

    // 配列の.lengthアクセスを特別に処理
    if (access.fieldName === "length") {
      return `${record}.length`
    }

    return `${record}.${access.fieldName}`
  }

  // 配列リテラルの生成
  generateArrayLiteral(arrayLiteral: ArrayLiteral): string {
    if (arrayLiteral.elements.length === 0) {
      return "[]"
    }

    const elements = arrayLiteral.elements.map((element) =>
      this.generateExpression(element)
    )

    return `[${elements.join(", ")}]`
  }

  // 配列アクセスの生成
  generateArrayAccess(arrayAccess: ArrayAccess): string {
    const array = this.generateExpression(arrayAccess.array)
    const index = this.generateExpression(arrayAccess.index)

    // 安全な配列アクセス: Maybe型を返す
    const actualArray = `(${array}.tag === 'Tuple' ? ${array}.elements : ${array})`
    return `((${index}) >= 0 && (${index}) < ${actualArray}.length ? { tag: 'Just', value: ${actualArray}[${index}] } : { tag: 'Nothing' })`
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
            const [${comp.generators.map((g) => g.variable).join(", ")}] = tuple;
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
          const [${comp.generators.map((g) => g.variable).join(", ")}] = tuple;
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
            const [${comp.generators.map((g) => g.variable).join(", ")}] = tuple;
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
          const [${comp.generators.map((g) => g.variable).join(", ")}] = tuple;
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

  // タプル式の生成
  generateTupleExpression(tuple: TupleExpression): string {
    const elements = tuple.elements
      .map((element) => this.generateExpression(element))
      .join(", ")
    return `{ tag: 'Tuple', elements: [${elements}] }`
  }

  // 構造体式の生成
  generateStructExpression(structExpr: StructExpression): string {
    // スプレッド構文または省略記法があるかチェック
    const hasSpread = this.hasSpreadFields(structExpr)
    const hasShorthand = this.hasShorthandFields(structExpr)

    if (hasSpread || hasShorthand) {
      return this.generateComplexStructExpression(structExpr)
    }

    return this.generateSimpleStructExpression(structExpr)
  }

  private hasSpreadFields(structExpr: StructExpression): boolean {
    return structExpr.fields.some((field) => field.kind === "RecordSpreadField")
  }

  private hasShorthandFields(structExpr: StructExpression): boolean {
    return structExpr.fields.some(
      (field) => field.kind === "RecordShorthandField"
    )
  }

  private generateComplexStructExpression(
    structExpr: StructExpression
  ): string {
    // スプレッドフィールドとイニシャライザーフィールドを収集
    const { spreadExpressions, initFields } =
      this.collectStructFields(structExpr)

    // フィールド部分文字列を組み立て
    const allFields = this.combineStructFields(spreadExpressions, initFields)

    if (allFields) {
      // 一時オブジェクトを作成し、構造体定義の順序に従ってコンストラクタ引数を構築
      const tempVar = `__tmp${Math.random().toString(36).substring(2, 8)}`
      return `(() => { const ${tempVar} = { ${allFields} }; return Object.assign(Object.create(${structExpr.structName}.prototype), ${tempVar}); })()`
    }

    return `new ${structExpr.structName}({})`
  }

  private collectStructFields(structExpr: StructExpression): {
    spreadExpressions: string[]
    initFields: { name: string; value: string }[]
  } {
    const spreadExpressions: string[] = []
    const initFields: { name: string; value: string }[] = []

    for (const field of structExpr.fields) {
      if (field.kind === "RecordSpreadField") {
        const spreadField = field as RecordSpreadField
        const spreadValue = this.generateExpression(
          spreadField.spreadExpression.expression
        )
        spreadExpressions.push(spreadValue)
      } else if (field.kind === "RecordInitField") {
        const initField = field as RecordInitField
        const value = this.generateExpression(initField.value)
        initFields.push({ name: initField.name, value })
      } else if (field.kind === "RecordShorthandField") {
        const shorthandField = field as RecordShorthandField
        // Shorthand property: use variable name directly
        initFields.push({
          name: shorthandField.name,
          value: shorthandField.name,
        })
      }
    }

    return { spreadExpressions, initFields }
  }

  private combineStructFields(
    spreadExpressions: string[],
    initFields: { name: string; value: string }[]
  ): string {
    const spreadPart = spreadExpressions.map((expr) => `...${expr}`).join(", ")
    const fieldsPart = initFields.map((f) => `${f.name}: ${f.value}`).join(", ")

    if (spreadPart && fieldsPart) {
      return `${spreadPart}, ${fieldsPart}`
    } else if (spreadPart) {
      return spreadPart
    } else if (fieldsPart) {
      return fieldsPart
    }

    return ""
  }

  private generateSimpleStructExpression(structExpr: StructExpression): string {
    // 従来のコンストラクタ形式（スプレッドなし）
    const fieldEntries: string[] = []

    for (const field of structExpr.fields) {
      if (field.kind === "RecordInitField") {
        const initField = field as RecordInitField
        const value = this.generateExpression(initField.value)
        fieldEntries.push(`${initField.name}: ${value}`)
      } else if (field.kind === "RecordShorthandField") {
        const shorthandField = field as RecordShorthandField
        fieldEntries.push(shorthandField.name)
      }
    }

    const fieldsObject =
      fieldEntries.length > 0 ? `{ ${fieldEntries.join(", ")} }` : "{}"

    return `new ${structExpr.structName}(${fieldsObject})`
  }

  // メソッド呼び出しの生成
  generateMethodCall(call: MethodCall): string {
    const receiver = this.generateExpression(call.receiver)
    const args = call.arguments.map((arg) => this.generateExpression(arg))

    // ディスパッチテーブルを使用したメソッド呼び出し
    const allArgs = args.length === 0 ? "" : `, ${args.join(", ")}`
    return `__dispatchMethod(${receiver}, "${call.methodName}"${allArgs})`
  }

  // タプル分解の生成
  generateTupleDestructuring(stmt: TupleDestructuring): string {
    const patternVars = this.extractTuplePatternVars(stmt.pattern)
    const initializer = this.generateExpression(stmt.initializer)
    // タプル型の場合はelementsから取り出す
    return `const [${patternVars.join(", ")}] = ${initializer}.tag === 'Tuple' ? ${initializer}.elements : ${initializer};`
  }

  // タプルパターンから変数名を抽出
  private extractTuplePatternVars(pattern: TuplePattern): string[] {
    const vars: string[] = []

    for (const subPattern of pattern.patterns) {
      if (subPattern.kind === "IdentifierPattern") {
        const identifierPattern = subPattern as IdentifierPattern
        if (identifierPattern.name === "_") {
          // ワイルドカードの場合は一意な変数名を生成
          vars.push(`_${this.wildcardCounter++}`)
        } else {
          vars.push(identifierPattern.name)
        }
      } else if (subPattern.kind === "WildcardPattern") {
        // ワイルドカードパターンの場合は一意な変数名を生成
        vars.push(`_${this.wildcardCounter++}`)
      } else {
        // より複雑なパターンは後で実装
        vars.push("/* complex pattern */")
      }
    }

    return vars
  }

  // スプレッド式の生成
  generateSpreadExpression(spread: SpreadExpression): string {
    // スプレッド式は通常直接使われることはないが、TypeScriptのスプレッド構文と同じ
    return `...${this.generateExpression(spread.expression)}`
  }

  // レコード分割代入の生成
  generateRecordDestructuring(stmt: RecordDestructuring): string {
    const initializer = this.generateExpression(stmt.initializer)
    const fieldPatterns = stmt.pattern.fields.map((field) => {
      if (field.alias) {
        return `${field.fieldName}: ${field.alias}`
      } else {
        return field.fieldName
      }
    })
    return `const { ${fieldPatterns.join(", ")} } = ${initializer};`
  }

  // 構造体分割代入の生成
  generateStructDestructuring(stmt: StructDestructuring): string {
    const initializer = this.generateExpression(stmt.initializer)
    const fieldPatterns = stmt.pattern.fields.map((field) => {
      if (field.alias) {
        return `${field.fieldName}: ${field.alias}`
      } else {
        return field.fieldName
      }
    })
    return `const { ${fieldPatterns.join(", ")} } = ${initializer};`
  }

  // テンプレートリテラルの生成
  generateTemplateExpression(expr: TemplateExpression): string {
    let result = "`"

    for (const part of expr.parts) {
      if (typeof part === "string") {
        // 文字列部分はそのまま追加
        result += part
      } else {
        // 埋め込み式はTypeScriptのテンプレートリテラル記法で囲む
        const exprCode = this.generateExpression(part)
        result += `\${${exprCode}}`
      }
    }

    result += "`"
    return result
  }

  // 型アサーションの生成
  generateTypeAssertion(assertion: TypeAssertion): string {
    const expr = this.generateExpression(assertion.expression)
    const targetType = this.generateType(assertion.targetType)

    // TypeScript風の型アサーション構文で生成
    return `(${expr} as ${targetType})`
  }
}
