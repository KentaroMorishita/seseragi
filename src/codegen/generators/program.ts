/**
 * プログラム全体のコード生成
 *
 * SeseragiプログラムをTypeScriptコードに変換するメインエントリーポイント
 */

import type { ImplBlock, ImportDeclaration, Statement } from "../../ast"
// @ts-expect-error Bun's ?raw import for embedding source code
import runtimeSource from "../../runtime/index.ts?raw"
import {
  type CodeGenContext,
  type CodeGenOptions,
  createContext,
  defaultOptions,
} from "../context"
import { operatorToMethodName } from "../helpers"
import { generateStatement } from "./statements/dispatcher"

/**
 * プログラム全体をTypeScriptに変換
 */
export function generateProgram(
  statements: Statement[],
  options: CodeGenOptions = {}
): string {
  // statementsのガード
  if (!statements || !Array.isArray(statements)) {
    return ""
  }

  const opts = { ...defaultOptions, ...options }
  const ctx = createContext(opts)

  const lines: string[] = []

  // ヘッダー生成
  addProgramHeader(ctx, lines)

  // 構造体前処理とディスパッチテーブル生成
  processStructuresAndDispatch(ctx, statements, lines)

  // 文を分類して生成（変数型情報はここで登録される）
  const statementLines: string[] = []
  generateStatementsByType(ctx, statements, statementLines)

  // 変数型情報テーブルを生成（文の生成後、登録された型情報から）
  const variableTypesTables = generateVariableTypesTables(ctx)
  if (variableTypesTables.length > 0) {
    lines.push(...variableTypesTables)
  }

  // 生成済みの文を追加
  lines.push(...statementLines)

  return lines.join("\n")
}

/**
 * プログラムヘッダーを追加
 */
function addProgramHeader(ctx: CodeGenContext, lines: string[]): void {
  if (ctx.options.generateComments) {
    lines.push("// Generated TypeScript code from Seseragi")
    lines.push("")
  }

  // runtimeModeに応じてランタイムを生成
  if (ctx.options.runtimeMode === "embedded") {
    lines.push(...generateEmbeddedRuntime())
  } else {
    lines.push(...generateRuntimeImports())
  }
  lines.push("")
}

/**
 * ランタイムインポートを生成
 */
function generateRuntimeImports(): string[] {
  const lines: string[] = []

  // 型定義のインポート
  lines.push("import type {")
  lines.push("  Maybe, Either, List, Signal")
  lines.push("} from '@seseragi/runtime';")
  lines.push("")

  // 関数のインポート
  lines.push("import {")
  lines.push("  // 基本ユーティリティ")
  lines.push("  pipe, reversePipe, map, applyWrapped, bind, foldMonoid,")
  lines.push("  // Unit型")
  lines.push("  Unit,")
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

/**
 * 構造体前処理とディスパッチテーブル生成
 */
function processStructuresAndDispatch(
  ctx: CodeGenContext,
  statements: Statement[],
  lines: string[]
): void {
  // まず構造体を処理してディスパッチテーブルを準備
  for (const stmt of statements) {
    if (stmt.kind === "ImplBlock") {
      preProcessImplBlock(ctx, stmt as ImplBlock)
    }
  }

  // インポートされたimplブロックも前処理
  preProcessImportedImpls(ctx, statements)

  // ディスパッチテーブルが必要かどうかを判定
  const needsDispatchTables = shouldGenerateDispatchTables(statements)

  if (needsDispatchTables) {
    lines.push(generateDispatchTables())
    lines.push("")
  }
}

/**
 * implブロックの前処理
 */
function preProcessImplBlock(ctx: CodeGenContext, implBlock: ImplBlock): void {
  // 構造体のメソッドと演算子を登録
  if (!ctx.structMethods.has(implBlock.typeName)) {
    ctx.structMethods.set(implBlock.typeName, new Set())
  }
  if (!ctx.structOperators.has(implBlock.typeName)) {
    ctx.structOperators.set(implBlock.typeName, new Set())
  }

  const methodSet = ctx.structMethods.get(implBlock.typeName)!
  const operatorSet = ctx.structOperators.get(implBlock.typeName)!

  // メソッドを登録
  for (const method of implBlock.methods) {
    methodSet.add(method.name)
  }

  // 演算子を登録
  for (const operator of implBlock.operators) {
    operatorSet.add(operator.operator)
  }
}

/**
 * インポートされたimplブロックの前処理
 */
function preProcessImportedImpls(
  ctx: CodeGenContext,
  statements: Statement[]
): void {
  if (!ctx.options.typeInferenceResult?.moduleResolver) {
    return
  }

  const resolver = ctx.options.typeInferenceResult.moduleResolver

  for (const stmt of statements) {
    if (stmt.kind === "ImportDeclaration") {
      const importDecl = stmt as ImportDeclaration
      const resolvedModule = resolver.resolve(
        importDecl.module,
        ctx.options.typeInferenceResult.currentFilePath || ""
      )

      if (!resolvedModule) {
        continue
      }

      // インポートされた各項目をチェック
      for (const item of importDecl.items) {
        // 構造体がインポートされた場合、対応するimplもチェック
        const typeDecl = resolvedModule.exports.types.get(item.name)
        if (typeDecl && typeDecl.kind === "StructDeclaration") {
          const implBlock = resolvedModule.exports.impls.get(item.name)
          if (implBlock) {
            preProcessImplBlock(ctx, implBlock)
          }
        }
      }
    }
  }
}

/**
 * ディスパッチテーブルが必要かどうかを判定
 *
 * 二項演算子が生成コードで __dispatchOperator を使用するため、
 * 基本的に常に生成する（オーバーヘッドは小さい）
 */
function shouldGenerateDispatchTables(_statements: Statement[]): boolean {
  // 二項演算子は常に __dispatchOperator を使用する可能性があるため、
  // 常にディスパッチテーブルを生成する
  return true
}

/**
 * ディスパッチテーブルを生成
 */
function generateDispatchTables(): string {
  return `
// Struct method and operator dispatch tables
let __structMethods: Record<string, Record<string, Function>> = {};
let __structOperators: Record<string, Record<string, Function>> = {};

// Method dispatch helper
function __dispatchMethod(obj: any, methodName: string, ...args: any[]): any {
  // 構造体のフィールドアクセスの場合は直接返す
  if (args.length === 0 && obj.hasOwnProperty(methodName)) {
    return obj[methodName];
  }
  const structName = obj.constructor.name;
  const structMethods = __structMethods[structName];
  if (structMethods && structMethods[methodName]) {
    return structMethods[methodName](obj, ...args);
  }
  throw new Error(\`Method '\${methodName}' not found for struct '\${structName}'\`);
}

// Operator dispatch helper
function __dispatchOperator(left: any, operator: string, right: any): any {
  const structName = left.constructor.name;
  const structOperators = __structOperators[structName];
  if (structOperators && structOperators[operator]) {
    return structOperators[operator](left, right);
  }
  // Fall back to native JavaScript operator
  switch (operator) {
    case '+': {
      // 型安全な加算：両方が数値の場合のみ数値演算、それ以外は文字列連結
      if (typeof left === 'number' && typeof right === 'number') return left + right;
      if (typeof left === 'string' || typeof right === 'string') return String(left) + String(right);
      return left + right;
    }
    case '-': return Number(left) - Number(right);
    case '*': return Number(left) * Number(right);
    case '/': return Number(left) / Number(right);
    case '%': return Number(left) % Number(right);
    case '**': return Number(left) ** Number(right);
    case '==': return left == right;
    case '!=': return left != right;
    case '<': return left < right;
    case '>': return left > right;
    case '<=': return left <= right;
    case '>=': return left >= right;
    case '&&': return left && right;
    case '||': return left || right;
    default: throw new Error(\`Unknown operator: \${operator}\`);
  }
}
`.trim()
}

/**
 * 変数型情報テーブルを生成
 */
function generateVariableTypesTables(ctx: CodeGenContext): string[] {
  const lines: string[] = []

  // __variableTypes テーブル
  if (ctx.variableTypes.size > 0) {
    const entries: string[] = []
    for (const [varName, typeStr] of ctx.variableTypes) {
      entries.push(`"${varName}": "${typeStr}"`)
    }
    lines.push("// 変数型情報テーブルの初期化")
    lines.push("Object.assign(__variableTypes, {")
    lines.push(`  ${entries.join(",\n  ")}`)
    lines.push("});")
    lines.push("")
  }

  // __variableAliases テーブル
  if (ctx.variableAliases.size > 0) {
    const entries: string[] = []
    for (const [varName, aliases] of ctx.variableAliases) {
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

/**
 * 文を分類して生成
 */
function generateStatementsByType(
  ctx: CodeGenContext,
  statements: Statement[],
  lines: string[]
): void {
  const { structStatements, implStatements, otherStatements } =
    categorizeStatements(statements)

  // 構造体定義
  generateStatementsOfType(ctx, structStatements, lines)

  // 実装ブロック
  generateStatementsOfType(ctx, implStatements, lines)

  // ディスパッチテーブル初期化（implブロック後、他の文の前）
  const dispatchInit = generateDispatchTableInitialization(ctx)
  if (dispatchInit) {
    lines.push(dispatchInit)
    lines.push("")
  }

  // 残りの文（ImportDeclarationを含む）
  generateStatementsOfType(ctx, otherStatements, lines)
}

/**
 * 文を分類
 */
function categorizeStatements(statements: Statement[]): {
  structStatements: Statement[]
  implStatements: Statement[]
  otherStatements: Statement[]
} {
  const structStatements: Statement[] = []
  const implStatements: Statement[] = []
  const otherStatements: Statement[] = []

  for (const stmt of statements) {
    if (stmt.kind === "StructDeclaration") {
      structStatements.push(stmt)
    } else if (stmt.kind === "ImplBlock") {
      implStatements.push(stmt)
    } else {
      otherStatements.push(stmt)
    }
  }

  return { structStatements, implStatements, otherStatements }
}

/**
 * 同じ種類の文を生成
 */
function generateStatementsOfType(
  ctx: CodeGenContext,
  statements: Statement[],
  lines: string[]
): void {
  for (const stmt of statements) {
    const code = generateStatement(ctx, stmt)
    if (code.trim()) {
      lines.push(code)
      lines.push("")
    }
  }
}

/**
 * 埋め込みランタイムを生成（一時ファイル実行用）
 */
function generateEmbeddedRuntime(): string[] {
  // runtimeSourceからexport/import文を削除してインライン化
  const processed = (runtimeSource as string)
    .replace(/^export\s+/gm, "") // export削除
    .replace(/^import\s+.*$/gm, "") // import削除
    .split("\n")
    .filter((line: string) => line.trim() !== "") // 空行を除去

  return processed
}

/**
 * ディスパッチテーブル初期化コードを生成
 */
function generateDispatchTableInitialization(ctx: CodeGenContext): string {
  const lines: string[] = []

  // メソッドディスパッチテーブル初期化
  if (ctx.structMethods.size > 0) {
    lines.push("// Initialize method dispatch table")
    lines.push("__structMethods = {")
    for (const [structName, methods] of Array.from(ctx.structMethods)) {
      const methodEntries = Array.from(methods)
        .map((methodName) => {
          const funcName = `__ssrg_${structName}_${ctx.filePrefix}_${methodName}`
          return `    "${methodName}": ${funcName}`
        })
        .join(",\n")
      lines.push(`  "${structName}": {\n${methodEntries}\n  },`)
    }
    lines.push("};")
    lines.push("")
  }

  // 演算子ディスパッチテーブル初期化
  if (ctx.structOperators.size > 0) {
    lines.push("// Initialize operator dispatch table")
    lines.push("__structOperators = {")
    for (const [structName, operators] of Array.from(ctx.structOperators)) {
      const operatorEntries = Array.from(operators)
        .map((op) => {
          const opMethodName = operatorToMethodName(op)
          const funcName = `__ssrg_${structName}_${ctx.filePrefix}_op_${opMethodName}`
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
