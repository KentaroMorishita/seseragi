import * as fs from "node:fs"
import * as path from "node:path"
import { Parser } from "../parser.js"
import { generateTypeScript } from "../codegen.js"
import { TypeInferenceSystem } from "../type-inference.js"

export interface CompileOptions {
  input: string
  output?: string
  watch?: boolean
  generateComments?: boolean
  useArrowFunctions?: boolean
  runtimeMode?: "embedded" | "import" | "minimal"
  skipTypeCheck?: boolean
}

export async function compileCommand(options: CompileOptions): Promise<void> {
  try {
    if (options.watch) {
      await watchAndCompile(options)
    } else {
      await compile(options)
    }
  } catch (error) {
    console.error(
      "Compilation failed:",
      error instanceof Error ? error.message : error
    )
    process.exit(1)
  }
}

async function compile(options: CompileOptions): Promise<void> {
  // ファイルの存在チェック
  if (!fs.existsSync(options.input)) {
    throw new Error(`Input file not found: ${options.input}`)
  }

  // ソースコードを読み込み
  const sourceCode = fs.readFileSync(options.input, "utf-8")

  // パースしてASTを生成
  console.log(`Parsing ${options.input}...`)
  const parser = new Parser(sourceCode)
  const parseResult = parser.parse()

  // パースエラーをチェック
  if (parseResult.errors.length > 0) {
    console.error("\n❌ Parsing failed:\n")
    for (const error of parseResult.errors) {
      console.error(error.message)
      console.error("") // Empty line for readability
    }
    throw new Error(`Parsing failed with ${parseResult.errors.length} error(s)`)
  }

  const ast = { statements: parseResult.statements }

  // 型チェック
  let inferenceResult: any = null
  if (!options.skipTypeCheck) {
    console.log("Type checking...")

    // 新しい型推論システムを使用
    const typeInference = new TypeInferenceSystem()
    inferenceResult = typeInference.infer(ast)

    if (inferenceResult.errors.length > 0) {
      console.error("\n❌ Type checking failed:\n")
      for (const error of inferenceResult.errors) {
        console.error(error.toString())
        console.error("") // Empty line for readability
      }
      throw new Error(
        `Type checking failed with ${inferenceResult.errors.length} error(s)`
      )
    }
    console.log("✓ Type checking passed")
  }

  // TypeScriptコードを生成
  console.log("Generating TypeScript code...")
  const typeScriptCode = generateTypeScript(ast.statements, {
    generateComments: options.generateComments,
    useArrowFunctions: options.useArrowFunctions,
    runtimeMode: options.runtimeMode || "embedded",
    typeInferenceResult: inferenceResult,
  })

  // TypeScriptコードを出力
  const outputFile = options.output || getDefaultOutputFileName(options.input)
  fs.writeFileSync(outputFile, typeScriptCode, "utf-8")
  console.log(`✓ Compiled to ${outputFile}`)
}

async function watchAndCompile(options: CompileOptions): Promise<void> {
  console.log(`Watching ${options.input} for changes...`)

  // watchモード時は必ず出力ファイルが必要
  if (!options.output) {
    options.output = getDefaultOutputFileName(options.input)
  }

  // 初回コンパイル
  await compile({ ...options, watch: false })

  // ファイル監視
  fs.watchFile(options.input, { interval: 1000 }, async (curr, prev) => {
    if (curr.mtime !== prev.mtime) {
      console.log(`\nFile changed: ${options.input}`)
      try {
        await compile({ ...options, watch: false })
      } catch (error) {
        console.error(
          "Compilation failed:",
          error instanceof Error ? error.message : error
        )
      }
    }
  })

  // プロセスを継続
  return new Promise(() => {
    process.on("SIGINT", () => {
      console.log("\nStopping watch mode...")
      process.exit(0)
    })
  })
}

function getDefaultOutputFileName(inputFile: string): string {
  const parsed = path.parse(inputFile)
  return path.join(parsed.dir, `${parsed.name}.ts`)
}
