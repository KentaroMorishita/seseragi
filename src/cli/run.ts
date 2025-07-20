import { spawn } from "node:child_process"
import * as fs from "node:fs"
import * as os from "node:os"
import * as path from "node:path"
import { generateTypeScript } from "../codegen.js"
import { Parser } from "../parser.js"
import { infer, TypeInferenceSystem } from "../type-inference.js"
import * as AST from "../ast.js"

export interface RunOptions {
  input: string
  tempDir?: string
  keepTemp?: boolean
  watch?: boolean
}

export async function runCommand(options: RunOptions): Promise<void> {
  if (options.watch) {
    await watchAndRun(options)
  } else {
    await runOnce(options)
  }
}

async function runOnce(options: RunOptions): Promise<void> {
  const tempFile = await compileToTemp(options)

  try {
    await executeTypeScript(tempFile)
  } finally {
    // 一時ファイルのクリーンアップ
    if (!options.keepTemp && fs.existsSync(tempFile)) {
      fs.unlinkSync(tempFile)
    }
  }
}

async function watchAndRun(options: RunOptions): Promise<void> {
  console.log(`Watching ${options.input} for changes...`)

  // 初回実行
  await runOnce(options)

  // ファイル監視
  fs.watchFile(options.input, { interval: 1000 }, async (curr, prev) => {
    if (curr.mtime !== prev.mtime) {
      console.log(`\nFile changed: ${options.input}`)
      try {
        await runOnce(options)
      } catch (error) {
        console.error(
          "Execution failed:",
          error instanceof Error ? error.message : error
        )
      }
    }
  })

  // プロセスを継続
  return new Promise<void>((resolve) => {
    process.on("SIGINT", () => {
      console.log("\nStopping watch mode...")
      fs.unwatchFile(options.input)
      resolve()
    })
  })
}

async function compileToTemp(options: RunOptions): Promise<string> {
  // ファイルの存在チェック
  if (!fs.existsSync(options.input)) {
    throw new Error(`Input file not found: ${options.input}`)
  }

  // 一時ディレクトリの決定
  const tempDir = options.tempDir || os.tmpdir()
  if (!fs.existsSync(tempDir)) {
    fs.mkdirSync(tempDir, { recursive: true })
  }

  // 一時ファイル名の生成
  const inputName = path.parse(options.input).name
  const timestamp = Date.now()
  const tempFileName = `seseragi_${inputName}_${timestamp}.ts`
  const tempFilePath = path.join(tempDir, tempFileName)

  // ソースコードを読み込み
  const sourceCode = fs.readFileSync(options.input, "utf-8")

  // パースしてASTを生成
  console.log(`Parsing ${options.input}...`)
  const parser = new Parser(sourceCode)
  const ast = parser.parse()

  if (ast.errors && ast.errors.length > 0) {
    throw new Error(ast.errors.map((e) => e.message).join("\n"))
  }

  // 型推論
  console.log("Running type inference...")
  const inference = new TypeInferenceSystem()
  const program = new AST.Program(ast.statements!)
  const inferenceResult = inference.infer(program)

  if (inferenceResult.errors.length > 0) {
    throw new Error(inferenceResult.errors.map((e) => e.message).join("\n"))
  }

  // 型チェック（main.tsと合わせるため一旦無効化）
  // console.log("Type checking...")
  // const program = new AST.Program(ast.statements!, 1, 1)
  // const typeChecker = new TypeChecker(inferenceResult.typeEnvironment)
  // const errors = typeChecker.check(program)

  // if (errors.length > 0) {
  //   throw new Error(errors.map((e) => e.message).join("\n"))
  // }

  // TypeScriptコードを生成
  console.log("Generating TypeScript code...")
  const typeScriptCode = generateTypeScript(ast.statements || [], {
    runtimeMode: "embedded", // 一時ファイル実行のため埋め込みモード必須
    typeInferenceResult: inferenceResult, // 型推論結果を渡す
  })

  // 一時ファイルに書き込み
  fs.writeFileSync(tempFilePath, typeScriptCode, "utf-8")

  if (options.keepTemp) {
    console.log(`✓ Compiled to temporary file: ${tempFilePath}`)
  }

  return tempFilePath
}

async function executeTypeScript(tempFile: string): Promise<void> {
  console.log("Running...")
  console.log("") // 空行でコンパイル出力と実行結果を分離

  return new Promise((resolve, reject) => {
    const bunProcess = spawn("bun", [tempFile], {
      stdio: "inherit", // 標準入出力を親プロセスと共有
    })

    bunProcess.on("close", (code) => {
      if (code === 0) {
        resolve()
      } else {
        reject(new Error(`Execution failed with exit code ${code}`))
      }
    })

    bunProcess.on("error", (error) => {
      reject(new Error(`Failed to start bun: ${error.message}`))
    })
  })
}

// プロセス終了時のクリーンアップ
const tempFiles: string[] = []

export function addTempFile(filePath: string): void {
  tempFiles.push(filePath)
}

export function cleanupTempFiles(): void {
  tempFiles.forEach((file) => {
    try {
      if (fs.existsSync(file)) {
        fs.unlinkSync(file)
      }
    } catch (_error) {
      // クリーンアップエラーは無視
    }
  })
  tempFiles.length = 0
}

// プロセス終了時のクリーンアップを登録
process.on("exit", cleanupTempFiles)
process.on("SIGINT", () => {
  cleanupTempFiles()
  process.exit(0)
})
process.on("SIGTERM", () => {
  cleanupTempFiles()
  process.exit(0)
})
