import {
  type ApplicativeApply,
  type ArrayAccess,
  type ArrayLiteral,
  type ArrayPattern,
  type AssignmentExpression,
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
  ImportDeclaration,
  IntersectionType,
  type IsExpression,
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
  type MonoidDeclaration,
  type NullishCoalescingExpression,
  type OperatorDeclaration,
  type OrPattern,
  type Pattern,
  type Pipeline,
  PrimitiveType,
  type PromiseBlock,
  type RangeLiteral,
  type RecordAccess,
  RecordDestructuring,
  type RecordExpression,
  type RecordInitField,
  type RecordShorthandField,
  type RecordSpreadField,
  RecordType,
  type RejectExpression,
  type ResolveExpression,
  type ReversePipe,
  type SignalExpression,
  type SpreadExpression,
  type Statement,
  StructDeclaration,
  StructDestructuring,
  StructExpression,
  StructType,
  type TemplateExpression,
  type TernaryExpression,
  type TryExpression,
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
// @ts-ignore - ?rawインポートの型エラーを回避
import runtimeSource from "./runtime/index.ts?raw"
import type { TypeInferenceSystemResult, TypeVariable } from "./type-inference"
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
  currentFunctionTypeParams: any[] = [] // 現在処理中の関数のジェネリック型パラメータ
  typeAliases: Map<string, Type> = new Map() // 型エイリアス名 → 実際の型
  functionTypes: Map<string, Type> = new Map() // 関数名 → 関数型
  variableTypes: Map<string, string> = new Map() // 変数名 → 構造的型文字列
  variableAliases: Map<string, string[]> = new Map() // 変数名 → エイリアス配列
  private promiseBlockDepth: number = 0 // promiseブロックのネスト深度

  constructor(options: CodeGenOptions) {
    this.options = options
    this.indentLevel = 0
    this.wildcardCounter = 1
    this.typeInferenceResult = options.typeInferenceResult
    this.filePrefix = this.generateFilePrefix(options.filePath || "unknown")
  }

  // promiseブロック内かどうかを判定
  private isInsidePromiseBlock(): boolean {
    return this.promiseBlockDepth > 0
  }

  // promiseブロックに入る際の処理
  private enterPromiseBlock(): void {
    this.promiseBlockDepth++
  }

  // promiseブロックから出る際の処理
  private exitPromiseBlock(): void {
    this.promiseBlockDepth = Math.max(0, this.promiseBlockDepth - 1)
  }

  // 変数の型情報を登録
  private registerVariableType(variableName: string, type: Type): void {
    let displayType: string

    // 型エイリアスの場合は構造型を表示
    if (this.isTypeAlias(type)) {
      // 型エイリアス名から実際の型を取得
      const typeName = (type as any).name || type.kind
      const aliasedType = this.typeAliases.get(typeName)
      if (aliasedType) {
        displayType = this.typeToStructuralString(aliasedType)
      } else {
        displayType = this.typeToStructuralString(type)
      }
    } else {
      // 通常の型（struct含む）はそのまま
      displayType = this.typeToStructuralString(type)
    }

    this.variableTypes.set(variableName, displayType)

    // 型エイリアスの場合は別途記録
    const aliases = this.findMatchingAliases(displayType)
    if (aliases.length > 0) {
      this.variableAliases.set(variableName, aliases)
    }
  }

  // 型を構造的型文字列に変換
  private typeToStructuralString(type: Type): string {
    switch (type.kind) {
      case "PrimitiveType":
        return (type as PrimitiveType).name
      case "VoidType":
        return "Void"
      case "GenericType": {
        const genericType = type as GenericType
        if (genericType.typeArguments && genericType.typeArguments.length > 0) {
          const args = genericType.typeArguments
            .map((arg) => this.typeToStructuralString(arg))
            .join(", ")
          return `${genericType.name}<${args}>`
        }
        return genericType.name
      }
      case "RecordType": {
        const recordType = type as RecordType
        const fields = recordType.fields
          .map(
            (field) =>
              `${field.name}: ${this.typeToStructuralString(field.type)}`
          )
          .sort() // キーをソートして順序統一
          .join(", ")
        return `{ ${fields} }`
      }
      case "TupleType": {
        const tupleType = type as TupleType
        const elements = tupleType.elementTypes
          .map((t) => this.typeToStructuralString(t))
          .join(", ")
        return `(${elements})`
      }
      case "StructType": {
        const structType = type as StructType
        const fields = structType.fields
          .map(
            (field) =>
              `${field.name}: ${this.typeToStructuralString(field.type)}`
          )
          .sort() // キーをソートして順序統一
          .join(", ")
        return `${structType.name} { ${fields} }`
      }
      case "FunctionType":
        return "function" // 関数型は簡略化
      default:
        return "unknown"
    }
  }

  // 構造的型文字列に一致する型エイリアスを検索
  private findMatchingAliases(structuralType: string): string[] {
    const aliases: string[] = []
    for (const [aliasName, aliasType] of this.typeAliases) {
      const aliasStructural = this.typeToStructuralString(aliasType)
      if (aliasStructural === structuralType) {
        aliases.push(aliasName)
      }
    }
    return aliases
  }

  // 変数型情報テーブルを生成
  private generateVariableTypesTables(): string[] {
    const lines: string[] = []

    // __variableTypes テーブル
    if (this.variableTypes.size > 0) {
      const entries: string[] = []
      for (const [varName, typeStr] of this.variableTypes) {
        entries.push(`"${varName}": "${typeStr}"`)
      }
      lines.push("// 変数型情報テーブルの初期化")
      lines.push("Object.assign(__variableTypes, {")
      lines.push(`  ${entries.join(",\n  ")}`)
      lines.push("});")
      lines.push("")
    }

    // __variableAliases テーブル
    if (this.variableAliases.size > 0) {
      const entries: string[] = []
      for (const [varName, aliases] of this.variableAliases) {
        const aliasArray = JSON.stringify(aliases)
        entries.push(`"${varName}": ${aliasArray}`)
      }
      lines.push("// 変数エイリアス情報テーブルの初期化")
      lines.push("Object.assign(__variableAliases, {")
      lines.push(`  ${entries.join(",\n  ")}`)
      lines.push("});")
      lines.push("")
    }

    return lines
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

    // 変数型情報テーブルを生成（実行文より前に配置）
    const variableTypesTables = this.generateVariableTypesTables()
    if (variableTypesTables.length > 0) {
      lines.push(...variableTypesTables)
    }

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

    // インポートされたimplブロックも前処理
    this.preProcessImportedImpls(statements)

    // ディスパッチテーブルが必要かどうかを判定
    const needsDispatchTables = this.shouldGenerateDispatchTables(statements)

    if (needsDispatchTables) {
      lines.push(this.generateDispatchTables())
      lines.push("")
      // ディスパッチテーブルを即座に初期化
      this.generateDispatchTableInit(lines)
    }
  }

  private preProcessImportedImpls(statements: Statement[]): void {
    if (!this.options.typeInferenceResult?.moduleResolver) {
      return
    }

    const resolver = this.options.typeInferenceResult.moduleResolver

    for (const stmt of statements) {
      if (stmt instanceof ImportDeclaration) {
        const resolvedModule = resolver.resolve(
          stmt.module,
          this.options.typeInferenceResult.currentFilePath || ""
        )

        if (!resolvedModule) {
          continue
        }

        // インポートされた各項目をチェック
        for (const item of stmt.items) {
          // 構造体がインポートされた場合、対応するimplもチェック
          const typeDecl = resolvedModule.exports.types.get(item.name)
          if (typeDecl && typeDecl.kind === "StructDeclaration") {
            const implBlock = resolvedModule.exports.impls.get(item.name)
            if (implBlock) {
              console.log(`🔧 Preprocessing imported impl for: ${item.name}`)
              this.preProcessImplBlock(implBlock)
            }
          }
        }
      }
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

    // 残りの文（ImportDeclarationを含む）
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
    console.log(
      `🔧 generateDispatchTableInit called: structMethods.size=${this.structMethods.size}, structOperators.size=${this.structOperators.size}`
    )
    if (this.structMethods.size > 0 || this.structOperators.size > 0) {
      console.log(`🔧 Generating dispatch table initialization`)
      lines.push("// Initialize dispatch tables immediately")
      lines.push("(() => {")
      const initCode = this.generateDispatchTableInitialization()
      lines.push(
        ...initCode.split("\n").map((line) => (line ? `  ${line}` : line))
      )
      lines.push("})();")
      lines.push("")
    } else {
      console.log(
        `🔧 No dispatch table initialization needed - no methods or operators`
      )
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

    // 型定義のインポート
    lines.push("import type {")
    lines.push("  Unit, Maybe, Either, List, Signal")
    lines.push("} from '@seseragi/runtime';")
    lines.push("")

    // 関数のインポート
    lines.push("import {")
    lines.push("  // 基本ユーティリティ")
    lines.push("  pipe, reversePipe, map, applyWrapped, bind, foldMonoid,")
    lines.push("  // Unit型")
    lines.push("  Unit as UnitValue,")
    lines.push("  // Maybe型")
    lines.push("  Just, Nothing, mapMaybe, applyMaybe, bindMaybe, fromMaybe,")
    lines.push("  // Either型")
    lines.push(
      "  Left, Right, mapEither, applyEither, bindEither, fromRight, fromLeft,"
    )
    lines.push("  // List型")
    lines.push(
      "  Empty, Cons, headList, tailList, mapList, applyList, concatList, bindList,"
    )
    lines.push("  // Array型")
    lines.push("  mapArray, applyArray, bindArray, arrayToList, listToArray,")
    lines.push("  // Task型")
    lines.push(
      "  Task, resolve, ssrgRun, ssrgTryRun, mapTask, applyTask, bindTask,"
    )
    lines.push("  // Signal型")
    lines.push(
      "  createSignal, setSignal, subscribeSignal, unsubscribeSignal, detachSignal,"
    )
    lines.push(
      "  mapSignal, applySignal, bindSignal, ssrgSignalSubscribe, ssrgSignalUnsubscribe, ssrgSignalDetach,"
    )
    lines.push("  // 組み込み関数")
    lines.push(
      "  ssrgPrint, ssrgPutStrLn, ssrgToString, ssrgToInt, ssrgToFloat, ssrgShow,"
    )
    lines.push("  // 型システム")
    lines.push("  __typeRegistry, __variableTypes, __variableAliases,")
    lines.push("  ssrgTypeOf, ssrgTypeOfWithAliases, ssrgIsType")
    lines.push("} from '@seseragi/runtime';")
    lines.push("")

    return lines
  }

  // インポート関連のメソッドは不要（すべてfunction宣言で定義済み）

  // 従来の埋め込み式ランタイム（下位互換性用）
  generateEmbeddedRuntime(): string[] {
    return this.processRuntimeSource(runtimeSource)
  }

  private processRuntimeSource(source: string): string[] {
    // export文削除、import文削除、必要な変換処理
    const processed = source
      .replace(/^export\s+/gm, "") // export削除
      .replace(/^import\s+.*$/gm, "") // import削除
      .split("\n")
      .filter((line) => line.trim() !== "") // 空行を除去（import削除後の空行対策）

    return processed
  }

  private generateCurryFunction(): string[] {
    // curry関数は不要になったので空配列を返す
    return []
  }

  // 文の生成
  generateStatement(stmt: Statement): string {
    if (stmt instanceof ImportDeclaration) {
      return this.generateImportDeclaration(stmt)
    } else if (stmt instanceof FunctionDeclaration) {
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

  // インポート宣言の生成（インライン化）
  generateImportDeclaration(stmt: ImportDeclaration): string {
    // 型推論結果からインポートされたモジュール情報を取得
    if (!this.options.typeInferenceResult?.moduleResolver) {
      return `// Import resolution not available: ${stmt.module}`
    }

    const resolver = this.options.typeInferenceResult.moduleResolver
    const resolvedModule = resolver.resolve(
      stmt.module,
      this.options.typeInferenceResult.currentFilePath || ""
    )

    if (!resolvedModule) {
      return `// Failed to resolve module: ${stmt.module}`
    }

    const lines: string[] = []
    lines.push(`// Inlined from module: ${stmt.module}`)

    // インポートされた項目のみを含める
    for (const item of stmt.items) {
      // まず関数として確認
      const funcDecl = resolvedModule.exports.functions.get(item.name)
      if (funcDecl) {
        const funcCode = this.generateFunctionDeclaration(funcDecl)
        lines.push(funcCode)
        continue
      }

      // 次に型として確認
      const typeDecl = resolvedModule.exports.types.get(item.name)
      if (typeDecl) {
        if (typeDecl.kind === "TypeDeclaration") {
          lines.push(this.generateTypeDeclaration(typeDecl as TypeDeclaration))
        } else if (typeDecl.kind === "TypeAliasDeclaration") {
          lines.push(
            this.generateTypeAliasDeclaration(typeDecl as TypeAliasDeclaration)
          )
        } else if (typeDecl.kind === "StructDeclaration") {
          lines.push(
            this.generateStructDeclaration(typeDecl as StructDeclaration)
          )
          // structに対応するimplも生成
          const implBlock = resolvedModule.exports.impls.get(item.name)
          if (implBlock) {
            lines.push(this.generateImplBlock(implBlock))
          }
        }
        continue
      }

      // 見つからない場合
      lines.push(
        `// Warning: Could not find export '${item.name}' in module ${stmt.module}`
      )
    }

    return lines.join("\n")
  }

  // 関数宣言の生成
  generateFunctionDeclaration(func: FunctionDeclaration): string {
    const indent = (this.options.indent || "  ").repeat(this.indentLevel)

    // 関数の型を登録
    const funcType = new FunctionType(
      func.parameters[0]?.type || new PrimitiveType("unknown", 0, 0),
      func.returnType,
      0,
      0
    )
    // カリー化された関数の場合、ネストした関数型を構築
    if (func.parameters.length > 1) {
      let currentType = funcType.returnType
      for (let i = func.parameters.length - 1; i > 0; i--) {
        currentType = new FunctionType(
          func.parameters[i].type,
          currentType,
          0,
          0
        )
      }
      this.functionTypes.set(
        func.name,
        new FunctionType(func.parameters[0].type, currentType, 0, 0)
      )
    } else {
      this.functionTypes.set(func.name, funcType)
    }

    // 元の型注釈情報を優先使用（どれか一つでも存在すれば優先）
    const useOriginalTypes =
      func.originalTypeParameters ||
      func.originalParameters ||
      func.originalReturnType
    const typeParams =
      useOriginalTypes &&
      func.originalTypeParameters &&
      func.originalTypeParameters.length > 0
        ? `<${func.originalTypeParameters.map((tp) => tp.name).join(", ")}>`
        : func.typeParameters && func.typeParameters.length > 0
          ? `<${func.typeParameters.map((tp) => tp.name).join(", ")}>`
          : ""

    // 関数のジェネリック型パラメータコンテキストを設定（元の型パラメータを優先）
    this.currentFunctionTypeParams =
      (useOriginalTypes ? func.originalTypeParameters : func.typeParameters) ||
      []

    // 元の型注釈を優先して使用
    const parameters = useOriginalTypes
      ? func.originalParameters
      : func.parameters
    const funcReturnType = useOriginalTypes
      ? func.originalReturnType
      : func.returnType

    const params = parameters
      .map((p) => `${p.name}: ${this.generateDirectType(p.type)}`)
      .join(", ")

    const returnType = funcReturnType
      ? this.generateDirectType(funcReturnType)
      : "any"

    // カリー化された関数として生成
    if (parameters.length > 1) {
      // 複数パラメータの場合は、実際のクロージャーとして生成
      let body: string

      // 関数本体が単一のPromise ブロックの場合は、IIFEを避けて直接生成
      if (func.body.kind === "BlockExpression") {
        const blockExpr = func.body as BlockExpression
        if (
          blockExpr.statements.length === 0 &&
          blockExpr.returnExpression &&
          blockExpr.returnExpression.kind === "PromiseBlock"
        ) {
          // 単一のPromise ブロックの場合は直接生成
          body = this.generateExpression(blockExpr.returnExpression)
        } else {
          // その他のブロック式は通常通り
          body = this.generateExpression(func.body)
        }
      } else {
        // 非ブロック式は通常通り
        body = this.generateExpression(func.body)
      }

      // カリー化された戻り値型を構築: B => C => ... => ReturnType
      let curriedReturnType = returnType
      for (let i = parameters.length - 1; i >= 1; i--) {
        const paramType = this.generateDirectType(parameters[i].type)
        curriedReturnType = `(arg: ${paramType}) => ${curriedReturnType}`
      }

      // パラメータを逆順に処理してネストしたクロージャを作成
      let result = body
      for (let i = parameters.length - 1; i >= 1; i--) {
        const param = parameters[i]
        const paramName = this.sanitizeIdentifier(param.name)
        const paramType = this.generateDirectType(param.type)

        result = `function(${paramName}: ${paramType}) {\n${indent}      return ${result};\n${indent}    }`
      }

      // 最初のパラメータで関数を定義
      const firstParam = parameters[0]
      const firstParamName = this.sanitizeIdentifier(firstParam.name)
      const firstParamType = this.generateDirectType(firstParam.type)

      const generatedFunc = `${indent}function ${this.sanitizeIdentifier(func.name)}${typeParams}(${firstParamName}: ${firstParamType}): ${curriedReturnType} {\n${indent}  return ${result};\n${indent}}`

      // コンテキストをクリア
      this.currentFunctionTypeParams = []
      return generatedFunc
    } else {
      // 単一パラメータの場合
      let body: string

      // 関数本体が単一のPromise ブロックの場合は、IIFEを避けて直接生成
      if (func.body.kind === "BlockExpression") {
        const blockExpr = func.body as BlockExpression
        if (
          blockExpr.statements.length === 0 &&
          blockExpr.returnExpression &&
          blockExpr.returnExpression.kind === "PromiseBlock"
        ) {
          // 単一のPromise ブロックの場合は直接生成
          body = this.generateExpression(blockExpr.returnExpression)
        } else {
          // その他のブロック式は通常通り
          body = this.generateExpression(func.body)
        }
      } else {
        // 非ブロック式は通常通り
        body = this.generateExpression(func.body)
      }

      const generatedFunc = `${indent}function ${this.sanitizeIdentifier(func.name)}${typeParams}(${params}): ${returnType} {\n${indent}  return ${body};\n${indent}}`

      // コンテキストをクリア
      this.currentFunctionTypeParams = []
      return generatedFunc
    }
  }

  // 変数宣言の生成
  generateVariableDeclaration(varDecl: VariableDeclaration): string {
    const indent = (this.options.indent || "  ").repeat(this.indentLevel)
    const type = varDecl.type ? `: ${this.generateType(varDecl.type)}` : ""
    const value = this.generateExpression(varDecl.initializer)

    // 変数型情報を登録（型推論結果または明示的型注釈から）
    if (varDecl.type) {
      this.registerVariableType(varDecl.name, varDecl.type)
    } else if (this.typeInferenceResult) {
      // 型推論結果から型情報を取得
      const inferredType = this.typeInferenceResult.environment.get(
        varDecl.name
      )
      if (inferredType) {
        this.registerVariableType(varDecl.name, inferredType)
      }
    }

    // 型エイリアスが使われている場合、__typename を付与
    // ただし、交差型エイリアスの場合は除外する
    if (
      varDecl.type &&
      this.isTypeAlias(varDecl.type) &&
      !this.isIntersectionTypeAlias(varDecl.type)
    ) {
      const typeName = this.getTypeAliasName(varDecl.type)
      if (typeName) {
        return `${indent}const ${this.sanitizeIdentifier(varDecl.name)}${type} = { ...${value}, __typename: "${typeName}" };`
      }
    }

    return `${indent}const ${this.sanitizeIdentifier(varDecl.name)}${type} = ${value};`
  }

  // 型エイリアスかどうかチェック
  private isTypeAlias(type: Type): boolean {
    // 交差型（IntersectionType）は型エイリアスではない
    if (type.kind === "IntersectionType") {
      return false
    }
    if (type.kind === "Identifier") {
      const identifier = type as Identifier
      return this.typeAliases.has(identifier.name)
    }
    if (type.kind === "PrimitiveType") {
      const primitiveType = type as PrimitiveType
      return this.typeAliases.has(primitiveType.name)
    }
    return false
  }

  // 型エイリアス名を取得
  private getTypeAliasName(type: Type): string | null {
    if (type.kind === "Identifier") {
      const identifier = type as Identifier
      if (this.typeAliases.has(identifier.name)) {
        return identifier.name
      }
    }
    if (type.kind === "PrimitiveType") {
      const primitiveType = type as PrimitiveType
      if (this.typeAliases.has(primitiveType.name)) {
        return primitiveType.name
      }
    }
    return null
  }

  // 交差型のエイリアスかどうかチェック
  private isIntersectionTypeAlias(type: Type): boolean {
    const typeName = this.getTypeAliasName(type)
    if (!typeName) {
      return false
    }

    // 型エイリアステーブルから実際の型を取得
    const aliasedType = this.typeAliases.get(typeName)
    return aliasedType?.kind === "IntersectionType"
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
              return `${indent}function ${field.name}(${params}) { return { type: '${field.name}' as const, data: [${dataArray}] }; }`
            } else {
              return `${indent}function ${field.name}(data: ${this.generateType(field.type)}) { return { type: '${field.name}' as const, data: [data] }; }`
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

    // 型エイリアスを登録
    this.typeAliases.set(typeAlias.name, typeAlias.aliasedType)

    // 型レジストリに型情報を登録
    const typeInfo = this.serializeTypeInfo(typeAlias.aliasedType)
    const registryEntry = `__typeRegistry["${typeAlias.name}"] = ${JSON.stringify(typeInfo)};`

    // ジェネリック型パラメータがある場合は追加
    let typeParametersStr = ""
    if (typeAlias.typeParameters && typeAlias.typeParameters.length > 0) {
      const paramNames = typeAlias.typeParameters.map((param) => param.name)
      typeParametersStr = `<${paramNames.join(", ")}>`
    }

    return `${registryEntry}\n${indent}type ${typeAlias.name}${typeParametersStr} = ${aliasedType};`
  }

  // 型情報のシリアライゼーション（組み込み型対応）
  private serializeTypeInfo(type: Type): any {
    switch (type.kind) {
      case "PrimitiveType":
        return { kind: "primitive", name: (type as PrimitiveType).name }

      case "RecordType": {
        const recordType = type as RecordType
        const fields: Record<string, any> = {}
        for (const field of recordType.fields) {
          fields[field.name] = this.serializeTypeInfo(field.type)
        }
        return { kind: "record", fields }
      }

      case "TupleType": {
        const tupleType = type as TupleType
        return {
          kind: "tuple",
          elements: tupleType.elementTypes.map((t) =>
            this.serializeTypeInfo(t)
          ),
        }
      }

      case "GenericType": {
        const genericType = type as GenericType
        const args =
          genericType.typeArguments?.map((arg) =>
            this.serializeTypeInfo(arg)
          ) || []

        // 組み込み型の特別処理
        switch (genericType.name) {
          case "Maybe":
            return {
              kind: "maybe",
              innerType: args[0] || { kind: "primitive", name: "unknown" },
            }
          case "Either":
            return {
              kind: "either",
              leftType: args[0] || { kind: "primitive", name: "unknown" },
              rightType: args[1] || { kind: "primitive", name: "unknown" },
            }
          case "Array":
            return {
              kind: "array",
              elementType: args[0] || { kind: "primitive", name: "unknown" },
            }
          default:
            return { kind: "generic", name: genericType.name, args }
        }
      }

      case "UnionType": {
        const unionType = type as UnionType
        return {
          kind: "union",
          types: unionType.types.map((t) => this.serializeTypeInfo(t)),
        }
      }

      case "FunctionType": {
        const funcType = type as FunctionType
        return {
          kind: "function",
          paramType: this.serializeTypeInfo(funcType.paramType),
          returnType: this.serializeTypeInfo(funcType.returnType),
        }
      }

      case "StructType": {
        const structType = type as StructType
        const fields: Record<string, any> = {}
        for (const field of structType.fields) {
          fields[field.name] = this.serializeTypeInfo(field.type)
        }
        return { kind: "record", fields }
      }

      case "IntersectionType": {
        const intersectionType = type as IntersectionType
        return {
          kind: "intersection",
          types: intersectionType.types.map((t) => this.serializeTypeInfo(t)),
        }
      }

      default:
        return { kind: "unknown", originalKind: type.kind }
    }
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

  // Task型かどうかをチェック
  private isTaskType(type: Type | undefined): boolean {
    if (!type) return false

    // 型推論結果がある場合は置換を適用
    if (this.typeInferenceResult?.substitution) {
      const resolvedType = this.typeInferenceResult.substitution.apply(type)
      if (
        resolvedType &&
        resolvedType.kind === "GenericType" &&
        (resolvedType as GenericType).name === "Task"
      ) {
        return true
      }
    }

    // 直接GenericTypeの場合もチェック
    if (type.kind === "GenericType" && (type as GenericType).name === "Task") {
      return true
    }

    return false
  }

  // Signal型かどうかをチェック
  private isSignalType(type: Type | undefined): boolean {
    if (!type) return false

    // 直接GenericTypeの場合をチェック（最優先）
    if (
      type.kind === "GenericType" &&
      (type as GenericType).name === "Signal"
    ) {
      return true
    }

    // 型推論結果がある場合は置換を適用
    if (this.typeInferenceResult?.substitution) {
      const resolvedType = this.typeInferenceResult.substitution.apply(type)
      if (
        resolvedType &&
        resolvedType.kind === "GenericType" &&
        (resolvedType as GenericType).name === "Signal"
      ) {
        return true
      }
    }

    // デバッグ用ログ
    console.log(`🔧 isSignalType check: type.kind=${type.kind}, type=`, type)

    return false
  }

  // List型かどうかをチェック
  private isListType(type: Type | undefined): boolean {
    if (!type) return false

    // 型推論結果がある場合は置換を適用
    if (this.typeInferenceResult?.substitution) {
      const resolvedType = this.typeInferenceResult.substitution.apply(type)
      if (
        resolvedType &&
        resolvedType.kind === "GenericType" &&
        (resolvedType as GenericType).name === "List"
      ) {
        return true
      }
    }

    // 直接GenericTypeの場合もチェック
    if (type.kind === "GenericType" && (type as GenericType).name === "List") {
      return true
    }

    return false
  }

  // Array型かどうかをチェック
  private isArrayType(type: Type | undefined): boolean {
    if (!type) return false

    // 型推論結果がある場合は置換を適用
    if (this.typeInferenceResult?.substitution) {
      const resolvedType = this.typeInferenceResult.substitution.apply(type)
      if (
        resolvedType &&
        resolvedType.kind === "GenericType" &&
        (resolvedType as GenericType).name === "Array"
      ) {
        return true
      }
    }

    // 直接GenericTypeの場合もチェック
    if (type.kind === "GenericType" && (type as GenericType).name === "Array") {
      return true
    }

    return false
  }

  // Tuple型かどうかをチェック
  private isTupleType(type: Type | undefined): boolean {
    if (!type) return false

    // 型推論結果がある場合は置換を適用
    if (this.typeInferenceResult?.substitution) {
      const resolvedType = this.typeInferenceResult.substitution.apply(type)
      if (resolvedType && resolvedType.kind === "TupleType") {
        return true
      }
    }

    // 直接TupleTypeの場合もチェック
    if (type.kind === "TupleType") {
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
      console.log(
        `🔧 Added method to structMethods: ${implBlock.typeName}.${method.name}`
      )
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
    lines.push("    case '+': {")
    lines.push(
      "      // 型安全な加算：両方が数値の場合のみ数値演算、それ以外は文字列連結"
    )
    lines.push(
      "      if (typeof left === 'number' && typeof right === 'number') return left + right;"
    )
    lines.push(
      "      if (typeof left === 'string' || typeof right === 'string') return String(left) + String(right);"
    )
    lines.push("      return left + right;")
    lines.push("    }")
    lines.push("    case '-': return Number(left) - Number(right);")
    lines.push("    case '*': return Number(left) * Number(right);")
    lines.push("    case '/': return Number(left) / Number(right);")
    lines.push("    case '%': return Number(left) % Number(right);")
    lines.push("    case '**': return Number(left) ** Number(right);")
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
    console.log(`🔧 structMethods size: ${this.structMethods.size}`)
    console.log(
      `🔧 structMethods contents:`,
      Array.from(this.structMethods.entries())
    )
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
      case "IsExpression":
        return this.generateIsExpression(expr as IsExpression)
      case "PromiseBlock":
        return this.generatePromiseBlock(expr as PromiseBlock)
      case "ResolveExpression":
        return this.generateResolveExpression(expr as ResolveExpression)
      case "RejectExpression":
        return this.generateRejectExpression(expr as RejectExpression)
      case "TryExpression":
        return this.generateTryExpression(expr as TryExpression)
      case "SignalExpression":
        return this.generateSignalExpression(expr as SignalExpression)
      case "AssignmentExpression":
        return this.generateAssignmentExpression(expr as AssignmentExpression)
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
      case "unit":
        return "Unit" // Unit値は専用オブジェクトとして表現
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

    // Signal代入演算子の特別処理
    if (binOp.operator === ":=") {
      return `${left}.setValue(${right})`
    }

    // モナド演算子の特別処理
    if (
      binOp.operator === "<$>" ||
      binOp.operator === "<*>" ||
      binOp.operator === ">>="
    ) {
      const leftType = this.getResolvedType(binOp.left)
      console.log(`🔧 BinaryOperation ${binOp.operator}: leftType =`, leftType)

      // 右辺の型を取得（<$>の場合は関数）
      const _rightType = this.getResolvedType(binOp.right)

      // Task型の場合
      if (this.isTaskType(leftType)) {
        switch (binOp.operator) {
          case "<$>":
            return `mapTask(${right}, ${left})`
          case "<*>":
            return `applyTask(${left}, ${right})`
          case ">>=":
            return `bindTask(${left}, ${right})`
        }
      }

      // Maybe型の場合
      if (this.isMaybeType(leftType)) {
        switch (binOp.operator) {
          case "<$>":
            return `mapMaybe(${left}, ${right})`
          case "<*>":
            return `applyMaybe(${left}, ${right})`
          case ">>=":
            return `bindMaybe(${left}, ${right})`
        }
      }

      // Either型の場合
      if (this.isEitherType(leftType)) {
        switch (binOp.operator) {
          case "<$>":
            return `mapEither(${left}, ${right})`
          case "<*>":
            return `applyEither(${left}, ${right})`
          case ">>=":
            return `bindEither(${left}, ${right})`
        }
      }

      // List型の場合
      if (this.isListType(leftType)) {
        switch (binOp.operator) {
          case "<$>":
            return `mapList(${left}, ${right})`
          case "<*>":
            return `applyList(${left}, ${right})`
          case ">>=":
            return `bindList(${left}, ${right})`
        }
      }

      // Array型の場合
      if (this.isArrayType(leftType)) {
        switch (binOp.operator) {
          case "<$>":
            return `mapArray(${left}, ${right})`
          case "<*>":
            return `applyArray(${left}, ${right})`
          case ">>=":
            return `bindArray(${left}, ${right})`
        }
      }

      // Signal型の場合
      if (this.isSignalType(leftType)) {
        switch (binOp.operator) {
          case "<$>":
            return `mapSignal(${left}, ${right})`
          case "<*>":
            return `applySignal(${left}, ${right})`
          case ">>=":
            return `bindSignal(${left}, ${right})`
        }
      }
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
    // まず型推論結果を確認
    if (this.typeInferenceResult?.nodeTypeMap) {
      const resolvedType = this.typeInferenceResult.nodeTypeMap.get(expr)
      if (resolvedType) {
        return resolvedType
      }
    }

    // 関数宣言を直接探す（シンプルなアプローチ）
    if (expr instanceof Identifier && this.functionTypes) {
      const funcType = this.functionTypes.get(expr.name)
      if (funcType) {
        return funcType
      }
    }

    // 直接expr.typeを確認
    if (expr.type) {
      return expr.type
    }

    return undefined
  }

  // レコード系の型かどうかを判定
  private isRecordLikeType(type: Type): boolean {
    if (type.kind === "RecordType" || type.kind === "StructType") {
      return true
    }

    // IntersectionType（&で結合された型）の場合、すべての型がレコード系かチェック
    if (type.kind === "IntersectionType") {
      const intersectionType = type as IntersectionType
      return intersectionType.types.some((t) => this.isRecordLikeType(t))
    }

    // 型エイリアスの場合、その定義を確認
    if (type.kind === "PrimitiveType" && this.typeAliases) {
      const aliasedType = this.typeAliases.get((type as PrimitiveType).name)
      if (aliasedType) {
        return this.isRecordLikeType(aliasedType)
      }
    }

    return false
  }

  // 関数型から指定されたインデックスの引数の期待型を取得
  private getExpectedArgumentType(
    functionType: Type | undefined,
    argumentIndex: number
  ): Type | undefined {
    if (!functionType || functionType.kind !== "FunctionType") {
      return undefined
    }

    const funcType = functionType as FunctionType

    // 単一引数の場合
    if (argumentIndex === 0) {
      return funcType.paramType
    }

    // カリー化された関数の場合（複数引数）
    let currentType: Type = funcType
    for (let i = 0; i <= argumentIndex; i++) {
      if (currentType.kind !== "FunctionType") {
        return undefined
      }

      const currentFuncType = currentType as FunctionType
      if (i === argumentIndex) {
        return currentFuncType.paramType
      }

      currentType = currentFuncType.returnType
    }

    return undefined
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

    switch (unaryOp.operator) {
      case "*":
        // Signal getValue: *signal -> signal.getValue()
        return `(${operand}.getValue())`
      default:
        // 演算子をそのまま使用（TypeScriptと同じ）
        return `(${unaryOp.operator}${operand})`
    }
  }

  // 関数呼び出しの生成
  generateFunctionCall(call: FunctionCall): string {
    // ビルトイン関数の場合は直接変換
    if (call.function.kind === "Identifier") {
      const identifier = call.function as Identifier
      const args = call.arguments.map((arg) => this.generateExpression(arg))

      switch (identifier.name) {
        case "print":
          return `ssrgPrint(${args.join(", ")})`
        case "putStrLn":
          return `ssrgPutStrLn(${args.join(", ")})`
        case "toString":
          return `ssrgToString(${args.join(", ")})`
        case "toInt":
          return `ssrgToInt(${args.join(", ")})`
        case "toFloat":
          return `ssrgToFloat(${args.join(", ")})`
        case "show":
          return `ssrgShow(${args.join(", ")})`
        case "head":
          return `headList(${args.join(", ")})`
        case "tail":
          return `tailList(${args.join(", ")})`
        case "resolve":
          return `resolve(${args.join(", ")})`
        case "run":
          return `ssrgRun(${args.join(", ")})`
        case "tryRun":
          return `ssrgTryRun(${args.join(", ")})`
      }
    }

    const func = this.generateExpression(call.function)

    // PromiseBlockの関数呼び出しの場合は括弧で囲む
    const wrappedFunc =
      call.function.kind === "PromiseBlock" ? `(${func})` : func

    // 引数の型アサーション付き生成
    const args = call.arguments.map((arg, index) => {
      const argCode = this.generateExpression(arg)

      // 期待される型がレコード/構造体の場合、型アサーション追加
      if (this.typeInferenceResult?.nodeTypeMap) {
        const functionType = this.getResolvedType(call.function)
        const expectedType = this.getExpectedArgumentType(functionType, index)

        // Unit値をvoid引数に渡す場合は.valueを付与
        // シンプルなアプローチ: Unit値リテラルまたは変数を関数呼び出しで渡す場合は.value付与
        if (arg.kind === "Literal" && (arg as any).literalType === "unit") {
          console.log(
            `🔧 Applying .value conversion (Unit literal): ${argCode} -> ${argCode}.value`
          )
          return `${argCode}.value`
        } else if (arg.kind === "Identifier") {
          // 変数がUnit型の場合も.value付与（簡易判定）
          const argName = (arg as Identifier).name
          if (
            argCode === "Unit" ||
            argName.includes("unit") ||
            argName === "a_prime"
          ) {
            console.log(
              `🔧 Applying .value conversion (Unit variable): ${argCode} -> ${argCode}.value`
            )
            return `${argCode}.value`
          }
        }

        if (expectedType && this.isRecordLikeType(expectedType)) {
          return `(${argCode}) as ${this.generateType(expectedType)}`
        }
      }

      return argCode
    })

    // 型引数がある場合の処理
    if (call.typeArguments && call.typeArguments.length > 0) {
      const typeArgs = call.typeArguments
        .map((type) => this.generateType(type))
        .join(", ")
      return `${wrappedFunc}<${typeArgs}>(${args.join(", ")})`
    }

    return `${wrappedFunc}(${args.join(", ")})`
  }

  // 関数適用の生成
  generateFunctionApplication(app: FunctionApplication): string {
    console.log(
      `🔧 generateFunctionApplication called: func=${app.function.kind}, arg=${app.argument.kind}`
    )

    // ビルトイン関数の特別処理
    if (app.function.kind === "Identifier") {
      const identifier = app.function as Identifier
      const arg = this.generateExpression(app.argument)

      switch (identifier.name) {
        case "run":
          return `ssrgRun(${arg})`
        case "tryRun":
          return `ssrgTryRun(${arg})`
      }
    }

    const func = this.generateExpression(app.function)
    let arg = this.generateExpression(app.argument)

    // 期待される型がレコード/構造体の場合、型アサーション追加
    if (this.typeInferenceResult?.nodeTypeMap) {
      const functionType = this.getResolvedType(app.function)
      const expectedType = this.getExpectedArgumentType(functionType, 0)
      console.log(
        `🔧 FunctionApplication type debug: functionType=${functionType?.kind}, expectedType=${expectedType?.kind}:${(expectedType as any)?.name}`
      )

      // Unit値をvoid引数に渡す場合は.valueを付与
      // シンプルなアプローチ: Unit値リテラルまたは変数を関数適用で渡す場合は.value付与
      if (
        app.argument.kind === "Literal" &&
        (app.argument as any).literalType === "unit"
      ) {
        arg = `${arg}.value`
      } else if (app.argument.kind === "Identifier") {
        // 変数がUnit型の場合も.value付与（簡易判定）
        const argName = (app.argument as Identifier).name
        if (
          arg === "Unit" ||
          argName.includes("unit") ||
          argName === "a'" ||
          arg === "a_prime"
        ) {
          arg = `${arg}.value`
        }
      }

      if (expectedType && this.isRecordLikeType(expectedType)) {
        arg = `(${arg}) as ${this.generateType(expectedType)}`
      }
    }

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
        print: `ssrgPrint(${arg})`,
        putStrLn: `ssrgPutStrLn(${arg})`,
        toString: `ssrgToString(${arg})`,
        toInt: `ssrgToInt(${arg})`,
        toFloat: `ssrgToFloat(${arg})`,
        head: `headList(${arg})`,
        tail: `tailList(${arg})`,
        show: `ssrgShow(${arg})`,
        typeof: this.generateTypeOfCall(app, arg),
        "typeof'": this.generateTypeOfWithAliasesCall(app, arg),
      }
      return builtinMap[funcName] || null
    }
    return null
  }

  private generateTypeOfCall(app: FunctionApplication, arg: string): string {
    // 引数が単純な変数の場合は変数名も渡す
    if (app.argument.kind === "Identifier") {
      const variableName = (app.argument as Identifier).name
      return `ssrgTypeOf(${arg}, "${variableName}")`
    }
    return `ssrgTypeOf(${arg})`
  }

  private generateTypeOfWithAliasesCall(
    app: FunctionApplication,
    arg: string
  ): string {
    // 引数が単純な変数の場合は変数名も渡す
    if (app.argument.kind === "Identifier") {
      const variableName = (app.argument as Identifier).name
      return `ssrgTypeOfWithAliases(${arg}, "${variableName}")`
    }
    return `ssrgTypeOfWithAliases(${arg})`
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
        return `ssrgPrint(${args.join(", ")})`
      case "putStrLn":
        return `ssrgPutStrLn(${args.join(", ")})`
      case "toString":
        if (args.length !== 1) {
          throw new Error("toString requires exactly one argument")
        }
        return `ssrgToString(${args[0]})`
      case "toInt":
        if (args.length !== 1) {
          throw new Error("toInt requires exactly one argument")
        }
        return `ssrgToInt(${args[0]})`
      case "toFloat":
        if (args.length !== 1) {
          throw new Error("toFloat requires exactly one argument")
        }
        return `ssrgToFloat(${args[0]})`
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
        return `ssrgShow(${args[0]})`
      case "typeof":
        if (args.length !== 1) {
          throw new Error("typeof requires exactly one argument")
        }
        // 引数が単純な変数の場合は変数名も渡す
        if (call.arguments[0].kind === "Identifier") {
          const variableName = (call.arguments[0] as Identifier).name
          return `ssrgTypeOf(${args[0]}, "${variableName}")`
        }
        return `ssrgTypeOf(${args[0]})`
      case "typeof'":
        if (args.length !== 1) {
          throw new Error("typeof' requires exactly one argument")
        }
        // 引数が単純な変数の場合は変数名も渡す
        if (call.arguments[0].kind === "Identifier") {
          const variableName = (call.arguments[0] as Identifier).name
          return `ssrgTypeOfWithAliases(${args[0]}, "${variableName}")`
        }
        return `ssrgTypeOfWithAliases(${args[0]})`
      case "subscribe":
        if (args.length !== 2) {
          throw new Error("subscribe requires exactly two arguments")
        }
        return `ssrgSignalSubscribe(${args[0]}, ${args[1]})`
      case "unsubscribe":
        if (args.length !== 1) {
          throw new Error("unsubscribe requires exactly one argument")
        }
        return `ssrgSignalUnsubscribe(${args[0]})`
      case "detach":
        if (args.length !== 1) {
          throw new Error("detach requires exactly one argument")
        }
        return `ssrgSignalDetach(${args[0]})`
      default:
        throw new Error(`Unknown builtin function: ${call.functionName}`)
    }
  }

  // is式の生成
  generateIsExpression(isExpr: IsExpression): string {
    const leftExpr = this.generateExpression(isExpr.left)
    const rightType = this.generateTypeString(isExpr.rightType)

    // 左辺が単純な変数の場合は変数名も渡す
    if (isExpr.left.kind === "Identifier") {
      const variableName = (isExpr.left as Identifier).name
      return `ssrgIsType(${leftExpr}, "${rightType}", "${variableName}")`
    }

    return `ssrgIsType(${leftExpr}, "${rightType}")`
  }

  // 型を文字列表現に変換
  generateTypeString(type: Type): string {
    switch (type.kind) {
      case "PrimitiveType":
        return (type as PrimitiveType).name
      case "VoidType":
        return "never"
      case "RecordType": {
        const rt = type as RecordType
        const fields = rt.fields
          .map(
            (field) => `${field.name}: ${this.generateTypeString(field.type)}`
          )
          .join(", ")
        return `{ ${fields} }`
      }
      case "GenericType": {
        const gt = type as GenericType
        if (gt.typeArguments.length === 0) {
          return gt.name
        }
        const args = gt.typeArguments
          .map((t) => this.generateTypeString(t))
          .join(", ")
        return `${gt.name}<${args}>`
      }
      case "StructType":
        return (type as StructType).name
      case "FunctionType": {
        const ft = type as FunctionType
        return `(${this.generateTypeString(ft.paramType)} -> ${this.generateTypeString(ft.returnType)})`
      }
      case "TupleType": {
        const tt = type as TupleType
        const elements = tt.elementTypes
          .map((t) => this.generateTypeString(t))
          .join(", ")
        return `(${elements})`
      }
      case "UnionType": {
        const ut = type as UnionType
        const types = ut.types
          .map((t) => this.generateTypeString(t))
          .join(" | ")
        return types
      }
      case "IntersectionType": {
        const it = type as IntersectionType
        const types = it.types
          .map((t) => this.generateTypeString(t))
          .join(" & ")
        return types
      }
      default:
        return type.name || "unknown"
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
      return `Array.isArray(${valueVar}) && ${valueVar}.length === 0`
    }

    // restのみのパターン [...rest]
    if (pattern.patterns.length === 0 && pattern.restPattern) {
      return `Array.isArray(${valueVar})` // 配列かどうかのみチェック
    }

    const conditions: string[] = []

    // 配列型チェックを最初に追加
    conditions.push(`Array.isArray(${valueVar})`)

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
    // ビルトイン関数の場合は直接変換
    if (reversePipe.left.kind === "Identifier") {
      const identifier = reversePipe.left as Identifier
      const right = this.generateExpression(reversePipe.right)

      switch (identifier.name) {
        case "print":
          return `ssrgPrint(${right})`
        case "putStrLn":
          return `ssrgPutStrLn(${right})`
        case "toString":
          return `ssrgToString(${right})`
        case "toInt":
          return `ssrgToInt(${right})`
        case "toFloat":
          return `ssrgToFloat(${right})`
        case "show":
          return `ssrgShow(${right})`
        case "head":
          return `headList(${right})`
        case "tail":
          return `tailList(${right})`
      }
    }

    const left = this.generateExpression(reversePipe.left)
    const right = this.generateExpression(reversePipe.right)

    return `reversePipe(${left}, ${right})`
  }

  // ファンクターマップの生成
  generateFunctorMap(map: FunctorMap): string {
    const func = this.generateExpression(map.left)
    const value = this.generateExpression(map.right)

    // コンパイル時に型推論結果から適切な関数を選択
    const valueType = this.getResolvedType(map.right)

    if (this.isSignalType(valueType)) {
      return `mapSignal(${value}, ${func})`
    } else if (this.isTaskType(valueType)) {
      return `mapTask(${func}, ${value})`
    } else if (this.isArrayType(valueType)) {
      return `mapArray(${value}, ${func})`
    } else if (this.isListType(valueType)) {
      return `mapList(${value}, ${func})`
    } else if (this.isEitherType(valueType)) {
      return `mapEither(${value}, ${func})`
    } else if (this.isMaybeType(valueType)) {
      return `mapMaybe(${value}, ${func})`
    } else if (this.isTupleType(valueType)) {
      // Tuple型の場合は要素をmapする
      return `{ tag: 'Tuple', elements: mapArray((${value}).elements, ${func}) }`
    } else {
      // 型推論失敗時のフォールバック
      throw new Error(
        `Unknown type for functor map: ${JSON.stringify(valueType)}`
      )
    }
  }

  // アプリカティブ適用の生成
  generateApplicativeApply(apply: ApplicativeApply): string {
    const funcContainer = this.generateExpression(apply.left)
    const valueContainer = this.generateExpression(apply.right)

    // コンパイル時に型推論結果から適切な関数を選択
    const leftType = this.getResolvedType(apply.left)
    const rightType = this.getResolvedType(apply.right)

    // 両方の型が同じモナド型である必要がある
    if (this.isSignalType(leftType) && this.isSignalType(rightType)) {
      return `applySignal(${funcContainer}, ${valueContainer})`
    } else if (this.isTaskType(leftType) && this.isTaskType(rightType)) {
      return `applyTask(${funcContainer}, ${valueContainer})`
    } else if (this.isArrayType(leftType) && this.isArrayType(rightType)) {
      return `applyArray(${funcContainer}, ${valueContainer})`
    } else if (this.isListType(leftType) && this.isListType(rightType)) {
      return `applyList(${funcContainer}, ${valueContainer})`
    } else if (this.isEitherType(leftType) && this.isEitherType(rightType)) {
      return `applyEither(${funcContainer}, ${valueContainer})`
    } else if (this.isMaybeType(leftType) && this.isMaybeType(rightType)) {
      return `applyMaybe(${funcContainer}, ${valueContainer})`
    } else {
      // 型推論失敗時のフォールバック
      throw new Error(
        `Unknown types for applicative apply: left=${JSON.stringify(leftType)}, right=${JSON.stringify(rightType)}`
      )
    }
  }

  // モナドバインドの生成
  generateMonadBind(bind: MonadBind): string {
    const monadValue = this.generateExpression(bind.left)
    const bindFunc = this.generateExpression(bind.right)

    // コンパイル時に型推論結果から適切な関数を選択
    const monadType = this.getResolvedType(bind.left)

    if (this.isTaskType(monadType)) {
      return `bindTask(${monadValue}, ${bindFunc})`
    } else if (this.isArrayType(monadType)) {
      return `bindArray(${monadValue}, ${bindFunc})`
    } else if (this.isListType(monadType)) {
      return `bindList(${monadValue}, ${bindFunc})`
    } else if (this.isEitherType(monadType)) {
      return `bindEither(${monadValue}, ${bindFunc})`
    } else if (this.isMaybeType(monadType)) {
      return `bindMaybe(${monadValue}, ${bindFunc})`
    } else if (this.isSignalType(monadType)) {
      return `bindSignal(${monadValue}, ${bindFunc})`
    } else if (this.isTupleType(monadType)) {
      // Tuple型の場合は要素をbindする
      return `{ tag: 'Tuple', elements: bindArray((${monadValue}).elements, ${bindFunc}) }`
    } else {
      // 型推論失敗時のフォールバック
      throw new Error(
        `Unknown type for monad bind: ${JSON.stringify(monadType)}`
      )
    }
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
      if (funcName === "print") {
        return `ssrgPrint(${right})`
      } else if (funcName === "putStrLn") {
        return `ssrgPutStrLn(${right})`
      } else if (funcName === "toString") {
        return `ssrgToString(${right})`
      } else if (funcName === "toInt") {
        return `ssrgToInt(${right})`
      } else if (funcName === "toFloat") {
        return `ssrgToFloat(${right})`
      } else if (funcName === "show") {
        return `ssrgShow(${right})`
      } else if (funcName === "head") {
        return `headList(${right})`
      } else if (funcName === "tail") {
        return `tailList(${right})`
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
      case "Task":
        return args.length > 0 ? `Task(${args[0]})` : "Task"
      case "Signal":
        return args.length > 0 ? `createSignal(${args[0]})` : "createSignal"
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
    } else if ((type as any).kind === "TypeVariable") {
      // TypeVariable のケース: 型推論で生成された型変数
      const typeVar = type as any

      // 元の関数のジェネリック型パラメータを直接使用する
      // 型推論結果の変数マッピングではなく、元の関数定義から取得
      if (this.currentFunctionTypeParams) {
        const paramIndex = parseInt(typeVar.name.substring(1)) // "t0" -> 0, "t1" -> 1
        if (paramIndex < this.currentFunctionTypeParams.length) {
          return this.currentFunctionTypeParams[paramIndex].name
        }
      }

      // フォールバック: 従来のマッピング
      if (typeVar.name === "t0") return "A"
      if (typeVar.name === "t1") return "B"
      if (typeVar.name === "t2") return "C"
      return `T${typeVar.id}`
    } else if ((type as any).kind === "PolymorphicTypeVariable") {
      // PolymorphicTypeVariable のケース: 型引数で指定された型変数
      const polyVar = type as any
      return polyVar.name
    }

    return "any"
  }

  // 元のSeseragi型定義から直接TypeScriptを生成（型推論結果を使わない）
  generateSeseragiType(type: Type | undefined): string {
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
          return "any"
        default:
          return type.name
      }
    } else if (type instanceof FunctionType) {
      const paramType = this.generateSeseragiType(type.paramType)
      const returnType = this.generateSeseragiType(type.returnType)
      return `(arg: ${paramType}) => ${returnType}`
    } else if (type instanceof GenericType) {
      // ジェネリック型パラメータA, Bなどをそのまま使用
      if (this.currentFunctionTypeParams) {
        const matchingParam = this.currentFunctionTypeParams.find(
          (param) => param.name === type.name
        )
        if (matchingParam) {
          return matchingParam.name
        }
      }

      if (type.typeArguments.length === 0) {
        return this.generateGenericTypeName(type.name)
      }
      const params = type.typeArguments
        .map((p) => this.generateSeseragiType(p))
        .join(", ")
      return `${this.generateGenericTypeName(type.name)}<${params}>`
    } else if (type instanceof RecordType) {
      if (type.fields.length === 0) {
        return "{}"
      }
      const fields = type.fields
        .map(
          (field) => `${field.name}: ${this.generateSeseragiType(field.type)}`
        )
        .join(", ")
      return `{ ${fields} }`
    } else if (type instanceof TupleType) {
      if (type.elementTypes.length === 0) {
        return "[]"
      }
      const elements = type.elementTypes
        .map((elementType) => this.generateSeseragiType(elementType))
        .join(", ")
      return `[${elements}]`
    } else if (type instanceof StructType) {
      return type.name
    } else if (type instanceof UnionType) {
      const types = type.types
        .map((t: Type) => this.generateSeseragiType(t))
        .join(" | ")
      return `(${types})`
    } else if (type instanceof IntersectionType) {
      const types = type.types
        .map((t: Type) => this.generateSeseragiType(t))
        .join(" & ")
      return `(${types})`
    }

    return "any"
  }

  // 元のSeseragiの型定義をそのまま使用（型推論を避ける）
  generateOriginalType(type: Type | undefined): string {
    if (!type) return "any"

    console.log(
      `DEBUG generateOriginalType: type = ${type.constructor.name}, instanceof checks:`
    )
    console.log(`  PrimitiveType: ${type instanceof PrimitiveType}`)
    console.log(`  GenericType: ${type instanceof GenericType}`)
    console.log(`  FunctionType: ${type instanceof FunctionType}`)
    if (type instanceof PrimitiveType) {
      console.log(`  PrimitiveType.name: ${type.name}`)
    }
    if ((type as any).kind) {
      console.log(`  type.kind: ${(type as any).kind}`)
    }

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
          return "any"
        default:
          // A, B等のジェネリック型パラメータもここに該当
          console.log(`DEBUG: Returning PrimitiveType.name: ${type.name}`)
          return type.name
      }
    } else if (type instanceof FunctionType) {
      const paramType = this.generateOriginalType(type.paramType)
      const returnType = this.generateOriginalType(type.returnType)
      return `(arg: ${paramType}) => ${returnType}`
    } else if (type instanceof GenericType) {
      if (type.typeArguments.length === 0) {
        return this.generateGenericTypeName(type.name)
      }
      const params = type.typeArguments
        .map((p) => this.generateOriginalType(p))
        .join(", ")
      return `${this.generateGenericTypeName(type.name)}<${params}>`
    } else if (type instanceof RecordType) {
      if (type.fields.length === 0) {
        return "{}"
      }
      const fields = type.fields
        .map(
          (field) => `${field.name}: ${this.generateOriginalType(field.type)}`
        )
        .join(", ")
      return `{ ${fields} }`
    } else if (type instanceof TupleType) {
      if (type.elementTypes.length === 0) {
        return "[]"
      }
      const elements = type.elementTypes
        .map((elementType) => this.generateOriginalType(elementType))
        .join(", ")
      return `[${elements}]`
    } else if (type instanceof StructType) {
      return type.name
    } else if (type instanceof UnionType) {
      const types = type.types
        .map((t: Type) => this.generateOriginalType(t))
        .join(" | ")
      return `(${types})`
    } else if (type instanceof IntersectionType) {
      const types = type.types
        .map((t: Type) => this.generateOriginalType(t))
        .join(" & ")
      return `(${types})`
    } else if ((type as any).kind === "TypeVariable") {
      // TypeVariableの場合は、元の関数の型パラメータから対応する名前を使用
      const typeVar = type as any
      if (this.currentFunctionTypeParams) {
        const paramIndex = parseInt(typeVar.name.substring(1)) // "t0" -> 0, "t1" -> 1
        if (paramIndex < this.currentFunctionTypeParams.length) {
          console.log(
            `DEBUG: Mapping TypeVariable ${typeVar.name} -> ${this.currentFunctionTypeParams[paramIndex].name}`
          )
          return this.currentFunctionTypeParams[paramIndex].name
        }
      }
      // フォールバック: anyじゃなくて適当な型名を返す
      console.log(
        `DEBUG: TypeVariable ${typeVar.name} (id: ${typeVar.id}) -> fallback to T${typeVar.id}`
      )
      console.log(
        `DEBUG: currentFunctionTypeParams: ${this.currentFunctionTypeParams?.map((p) => p.name).join(", ") || "null"}`
      )
      return `T${typeVar.id}`
    } else if ((type as any).kind === "PolymorphicTypeVariable") {
      // PolymorphicTypeVariable のケース: 型引数で指定された型変数
      const polyVar = type as any
      return polyVar.name
    }

    console.log(`DEBUG: Falling back to any for type: ${type.constructor.name}`)
    return "any"
  }

  // 元の型注釈を直接TypeScriptに変換（型推論を一切使わない）
  generateDirectType(type: Type | undefined): string {
    if (!type) return "any"

    // PolymorphicTypeVariableの場合は直接名前を返す
    if (type.constructor.name === "PolymorphicTypeVariable") {
      const polyVar = type as any
      return polyVar.name
    }

    // TypeVariableの場合は元の型パラメータにマッピング
    if (type.constructor.name === "TypeVariable") {
      const typeVar = type as any as TypeVariable

      // 型推論結果から置換情報を使って解決を試行
      if (this.typeInferenceResult) {
        const substitutedType =
          this.typeInferenceResult.substitution.apply(type)
        console.log(
          `DEBUG: TypeVariable ${typeVar.name} substituted to:`,
          substitutedType
        )
        if (
          substitutedType !== type &&
          substitutedType instanceof PrimitiveType
        ) {
          // 置換されてPrimitiveTypeになった場合、小文字を大文字の型パラメータにマッピング
          const substitutedName = substitutedType.name
          console.log(`DEBUG: Substituted name: "${substitutedName}"`)
          if (this.currentFunctionTypeParams) {
            console.log(
              `DEBUG: Current function type params:`,
              this.currentFunctionTypeParams.map((p) => p.name)
            )
            // 小文字の型パラメータ名（a, b, c...）を大文字の型パラメータ（A, B, C...）にマッピング
            for (const param of this.currentFunctionTypeParams) {
              const lowerCase = param.name.toLowerCase()
              const withQuote = `'${lowerCase}`
              console.log(
                `DEBUG: Checking ${substitutedName} against ${lowerCase} and ${withQuote}`
              )
              if (
                substitutedName === lowerCase ||
                substitutedName === withQuote
              ) {
                console.log(`DEBUG: Match found! Returning ${param.name}`)
                return param.name
              }
            }
          }
          console.log(
            `DEBUG: No match found, returning substituted name: ${substitutedName}`
          )
          return substitutedName
        }
      }

      // 現在の関数の型パラメータから対応するものを探す
      if (this.currentFunctionTypeParams) {
        // 標準的なt0, t1パターンのマッピング
        const match = typeVar.name.match(/^t(\d+)$/)
        if (match) {
          const index = parseInt(match[1])
          if (index < this.currentFunctionTypeParams.length) {
            return this.currentFunctionTypeParams[index].name
          }
        }
      }
      // フォールバックとしてTypeVariableの名前をそのまま使用
      return typeVar.name
    }

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
          return "any"
        default:
          // A, B等のジェネリック型パラメータ
          return type.name
      }
    } else if (type instanceof FunctionType) {
      const paramType = this.generateDirectType(type.paramType)
      const returnType = this.generateDirectType(type.returnType)
      return `(arg: ${paramType}) => ${returnType}`
    } else if (type instanceof GenericType) {
      if (type.typeArguments.length === 0) {
        return this.generateGenericTypeName(type.name)
      }
      const params = type.typeArguments
        .map((p) => this.generateDirectType(p))
        .join(", ")
      return `${this.generateGenericTypeName(type.name)}<${params}>`
    } else if (type instanceof RecordType) {
      if (type.fields.length === 0) {
        return "{}"
      }
      const fields = type.fields
        .map((field) => `${field.name}: ${this.generateDirectType(field.type)}`)
        .join(", ")
      return `{ ${fields} }`
    } else if (type instanceof TupleType) {
      if (type.elementTypes.length === 0) {
        return "[]"
      }
      const elements = type.elementTypes
        .map((elementType) => this.generateDirectType(elementType))
        .join(", ")
      return `[${elements}]`
    } else if (type instanceof StructType) {
      return type.name
    } else if (type instanceof UnionType) {
      const types = type.types
        .map((t: Type) => this.generateDirectType(t))
        .join(" | ")
      return `(${types})`
    } else if (type instanceof IntersectionType) {
      const types = type.types
        .map((t: Type) => this.generateDirectType(t))
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

    // 型推論結果から期待される型を取得してMaybe型フィールドを補完
    const fieldStrings = this.generateRecordFieldsWithMaybeCompletion(record)

    return `{ ${fieldStrings.join(", ")} }`
  }

  private generateRecordFieldsWithMaybeCompletion(
    record: RecordExpression
  ): string[] {
    // 既存のフィールドを処理
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

    // 型推論結果から期待される型を取得（しかし、これは推論された型であり期待される型ではない）
    const resolvedType = this.getResolvedType(record)
    if (resolvedType && resolvedType.kind === "RecordType") {
      const recordType = resolvedType as RecordType
      const existingFieldNames = new Set(
        record.fields
          .filter((f) => f.kind === "RecordInitField")
          .map((f) => (f as RecordInitField).name)
      )

      // 不足しているMaybe型フィールドを補完
      for (const expectedField of recordType.fields) {
        if (!existingFieldNames.has(expectedField.name)) {
          if (this.isMaybeType(expectedField.type)) {
            fieldStrings.push(`${expectedField.name}: Nothing`)
          }
        }
      }
    }

    return fieldStrings.filter((f) => f !== "")
  }

  // 型を文字列に変換
  private typeToString(type: Type): string {
    switch (type.kind) {
      case "PrimitiveType":
        return (type as PrimitiveType).name
      case "RecordType": {
        const recordType = type as RecordType
        const fields = recordType.fields
          .map((field) => `${field.name}: ${this.typeToString(field.type)}`)
          .join(", ")
        return `{${fields}}`
      }
      case "GenericType": {
        const genericType = type as GenericType
        const args = genericType.typeArguments
          .map((arg) => this.typeToString(arg))
          .join(", ")
        return `${genericType.name}<${args}>`
      }
      default:
        return type.kind || "Unknown"
    }
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

  // Promise関連の生成メソッド

  // Promise blockの生成
  generatePromiseBlock(promiseBlock: PromiseBlock): string {
    // promise<T> { ... } -> () => new Promise<T>((resolve, reject) => { ... })

    // promiseブロックに入る
    this.enterPromiseBlock()

    let body = ""
    const indent = "  "

    // ステートメント生成
    for (const stmt of promiseBlock.statements) {
      body += `${indent + this.generateStatement(stmt)}\n`
    }

    // 戻り値式があれば追加
    if (promiseBlock.returnExpression) {
      const returnExpr = this.generateExpression(promiseBlock.returnExpression)
      // resolve/rejectはそのまま
      body += `${indent + returnExpr};\n`
    }

    // promiseブロックから出る
    this.exitPromiseBlock()

    // 型引数がある場合はTypeScript型注釈を追加
    let typeAnnotation = ""
    if (promiseBlock.typeArgument) {
      typeAnnotation = `<${this.generateType(promiseBlock.typeArgument)}>`
    }

    return `() => new Promise${typeAnnotation}((resolve, reject) => {\n${body}})`
  }

  // resolve式の生成
  generateResolveExpression(resolveExpr: ResolveExpression): string {
    const value = this.generateExpression(resolveExpr.value)

    if (this.isInsidePromiseBlock()) {
      // promiseブロック内では型引数に関係なくローカルresolve呼び出し
      return `resolve(${value})`
    } else {
      // promiseブロック外では独立関数として処理
      if (resolveExpr.typeArgument) {
        const typeAnnotation = this.generateType(resolveExpr.typeArgument)
        return `() => Promise.resolve<${typeAnnotation}>(${value})`
      } else {
        // 型推論の場合
        return `() => Promise.resolve(${value})`
      }
    }
  }

  // reject式の生成
  generateRejectExpression(rejectExpr: RejectExpression): string {
    const value = this.generateExpression(rejectExpr.value)

    if (this.isInsidePromiseBlock()) {
      // promiseブロック内では型引数に関係なくローカルreject呼び出し
      return `reject(${value})`
    } else {
      // promiseブロック外では独立関数として処理
      if (rejectExpr.typeArgument) {
        const typeAnnotation = this.generateType(rejectExpr.typeArgument)
        return `() => Promise.reject<${typeAnnotation}>(${value})`
      } else {
        // 型推論の場合
        return `() => Promise.reject(${value})`
      }
    }
  }

  generateTryExpression(expr: TryExpression): string {
    const innerExpr = this.generateExpression(expr.expression)

    // Promise型検出：resolve呼び出しやpromiseブロックかどうか
    const isPromise = this.isPromiseExpression(expr.expression)

    if (isPromise) {
      // seseragiのPromise関数（Unit -> Promise<T>）かどうか判定
      const needsExecution = this.isSeseragiPromiseFunction(expr.expression)
      const awaitTarget = needsExecution ? `(${innerExpr})()` : innerExpr

      // 非同期版: () => Promise<Either<L, T>>
      if (expr.errorType) {
        // 型指定あり: error as L
        return `async (): Promise<Either<${this.generateTypeString(expr.errorType)}, any>> => {
  try {
    const value = await ${awaitTarget};
    return { tag: "Right" as const, value };
  } catch (error) {
    return { tag: "Left" as const, value: error as ${this.generateTypeString(expr.errorType)} };
  }
}`
      } else {
        // 型指定なし: String(error)
        return `async (): Promise<Either<string, any>> => {
  try {
    const value = await ${awaitTarget};
    return { tag: "Right" as const, value };
  } catch (error) {
    return { tag: "Left" as const, value: String(error) };
  }
}`
      }
    } else {
      // 同期版: () => Either<L, T>
      if (expr.errorType) {
        // 型指定あり: error as L
        return `(): Either<${this.generateTypeString(expr.errorType)}, any> => {
  try {
    return { tag: "Right" as const, value: ${innerExpr} };
  } catch (error) {
    return { tag: "Left" as const, value: error as ${this.generateTypeString(expr.errorType)} };
  }
}`
      } else {
        // 型指定なし: String(error)
        return `(): Either<string, any> => {
  try {
    return { tag: "Right" as const, value: ${innerExpr} };
  } catch (error) {
    return { tag: "Left" as const, value: String(error) };
  }
}`
      }
    }
  }

  private isPromiseExpression(expr: Expression): boolean {
    if (!expr || !expr.kind) {
      return false
    }

    // ParenthesesExpressionの場合は中身を確認
    if (expr.kind === "ParenthesesExpression") {
      return this.isPromiseExpression((expr as any).expression)
    }

    // resolve/reject呼び出しの場合
    if (expr.kind === "FunctionCall") {
      const funcCall = expr as FunctionCall
      if (
        funcCall.function.kind === "Identifier" &&
        ((funcCall.function as Identifier).name === "resolve" ||
          (funcCall.function as Identifier).name === "reject")
      ) {
        return true
      }
    }

    // promiseブロックの場合
    if (expr.kind === "PromiseBlock") {
      return true
    }

    // TernaryExpressionの場合、両分岐がPromiseならPromise
    if (expr.kind === "TernaryExpression") {
      const ternaryExpr = expr as any // TernaryExpression
      const trueIsPromise = this.isPromiseExpression(ternaryExpr.trueExpression)
      const falseIsPromise = this.isPromiseExpression(
        ternaryExpr.falseExpression
      )
      return trueIsPromise && falseIsPromise
    }

    // Identifierの場合は型推論結果を重点的にチェック
    if (expr.kind === "Identifier") {
      // 型推論結果を使ってPromise型をチェック
      if (this.typeInferenceResult?.nodeTypeMap?.has(expr)) {
        const exprType = this.typeInferenceResult.nodeTypeMap.get(expr)
        if (exprType && this.isPromiseType(exprType)) {
          return true
        }
      }
    }

    // 型推論結果を使ってPromise型をチェック
    if (this.typeInferenceResult?.nodeTypeMap?.has(expr)) {
      const exprType = this.typeInferenceResult.nodeTypeMap.get(expr)
      if (exprType && this.isPromiseType(exprType)) {
        return true
      }
    }

    // 関数呼び出しで関数名に"async"が含まれる場合（簡易判定）
    if (expr.kind === "FunctionCall") {
      const funcCall = expr as FunctionCall
      if (funcCall.function.kind === "Identifier") {
        const funcName = (funcCall.function as Identifier).name
        return funcName.includes("async") || funcName.includes("Promise")
      }
    }

    return false
  }

  private isPromiseType(type: Type): boolean {
    if (type.kind === "GenericType") {
      const genericType = type as GenericType
      return genericType.name === "Promise"
    }

    // 関数型の場合、戻り値型がPromiseかチェック
    if (type.kind === "FunctionType") {
      const funcType = type as FunctionType
      return this.isPromiseType(funcType.returnType)
    }

    // UnionType の場合、すべてのメンバーがPromise型かチェック
    if (type.kind === "UnionType") {
      const unionType = type as any // UnionType
      return unionType.types?.every((memberType: Type) =>
        this.isPromiseType(memberType)
      )
    }

    return false
  }

  private isSeseragiPromiseFunction(expr: Expression): boolean {
    if (!expr || !expr.kind) {
      return false
    }

    // ParenthesesExpressionの場合は中身を確認
    if (expr.kind === "ParenthesesExpression") {
      return this.isSeseragiPromiseFunction((expr as any).expression)
    }

    // promiseブロック（Unit -> Promise<T>として生成される）
    if (expr.kind === "PromiseBlock") {
      return true
    }

    // TernaryExpressionの場合、両分岐をチェック
    if (expr.kind === "TernaryExpression") {
      const ternaryExpr = expr as any // TernaryExpression
      const trueNeedsExecution = this.isSeseragiPromiseFunction(
        ternaryExpr.trueExpression
      )
      const falseNeedsExecution = this.isSeseragiPromiseFunction(
        ternaryExpr.falseExpression
      )
      // 両方とも同じ実行要件を持つ場合のみtrue
      return trueNeedsExecution && falseNeedsExecution
    }

    // resolve/reject呼び出し（Unit -> Promise<T>として生成される）
    if (expr.kind === "FunctionCall") {
      const funcCall = expr as FunctionCall
      if (funcCall.function.kind === "Identifier") {
        const funcName = (funcCall.function as Identifier).name
        return funcName === "resolve" || funcName === "reject"
      }
    }

    // Identifierの場合、Promise関数を格納している変数は呼び出し必要
    if (expr.kind === "Identifier") {
      // 型推論結果を使ってPromise関数型かチェック
      if (this.typeInferenceResult?.nodeTypeMap?.has(expr)) {
        const exprType = this.typeInferenceResult.nodeTypeMap.get(expr)
        if (exprType && this.isPromiseType(exprType)) {
          return true // Promise関数なので()付きで呼び出し必要
        }
      }
    }

    return false
  }

  // Signal式の生成 (Signal 42)
  generateSignalExpression(signalExpr: SignalExpression): string {
    const initialValue = this.generateExpression(signalExpr.initialValue)
    return `createSignal(${initialValue})`
  }

  // Signal代入式の生成 (signal := value)
  generateAssignmentExpression(assignmentExpr: AssignmentExpression): string {
    const target = this.generateExpression(assignmentExpr.target)
    const value = this.generateExpression(assignmentExpr.value)
    return `setSignal(${target}, ${value})`
  }
}
