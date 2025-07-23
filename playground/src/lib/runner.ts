import * as AST from "@seseragi/core/ast"
import { generateTypeScript } from "@seseragi/core/codegen"
import { Parser } from "@seseragi/core/parser"
import { TypeInferenceSystem } from "@seseragi/core/type-inference"
import * as ts from "typescript"

export async function compileAndRun(seseragiCode: string): Promise<string> {
  try {
    // 1. Seseragiコードを TypeScript にコンパイル（run.tsと同じ処理）
    const parser = new Parser(seseragiCode)
    const parseResult = parser.parse()

    if (parseResult.errors && parseResult.errors.length > 0) {
      throw new Error(parseResult.errors.map((e) => e.message).join("\n"))
    }

    // 型推論
    const inference = new TypeInferenceSystem()
    const program = new AST.Program(parseResult.statements!)
    const inferenceResult = inference.infer(program)

    if (inferenceResult.errors.length > 0) {
      throw new Error(inferenceResult.errors.map((e) => e.message).join("\n"))
    }

    // TypeScriptコードを生成（ランタイム込み）
    const combinedTsCode = generateTypeScript(parseResult.statements!, {
      typeInferenceResult: inferenceResult,
      runtimeMode: "embedded",
    })

    // 2. TypeScript → JavaScript変換
    const transpileOptions: ts.TranspileOptions = {
      compilerOptions: {
        target: ts.ScriptTarget.ES2020,
        module: ts.ModuleKind.None,
        strict: false,
      },
    }
    const jsResult = ts.transpile(
      combinedTsCode,
      transpileOptions.compilerOptions
    )

    // 3. export/CommonJS関連をクリーンアップ
    const jsCode = jsResult
      .replace(/export\s+/g, "")
      .replace(/exports\./g, "")
      .replace(/"use strict";/g, "")
      .replace(/Object\.defineProperty\([^)]*\);?/g, "")
      .replace(/const\s+/g, "var ") // constをvarに変更して巻き上げを有効にする

    console.log("Generated JavaScript:", jsCode)

    // 出力をキャプチャするための設定
    let output = ""
    const originalConsoleLog = console.log

    // console.logをオーバーライド
    console.log = (...args: any[]) => {
      output += `${args.map((arg) => String(arg)).join(" ")}\n`
    }

    try {
      // 4. JavaScriptを実行
      const fn = new Function(jsCode)
      fn()

      return output.trim() || "Program executed successfully (no output)"
    } finally {
      // console.logを復元
      console.log = originalConsoleLog
    }
  } catch (error) {
    if (error instanceof Error) {
      throw new Error(`Compilation/Execution Error: ${error.message}`)
    }
    throw error
  }
}
