import * as AST from "../src/ast"
import { generateTypeScript } from "../src/codegen"
import { Parser } from "../src/parser"
import { TypeInferenceSystem } from "../src/type-inference"

export function compileSeseragi(source: string): string {
  const parser = new Parser(source)
  const parseResult = parser.parse()
  if (!parseResult.statements) return ""
  const program = new AST.Program(parseResult.statements)
  const typeInference = new TypeInferenceSystem()
  const typeResult = typeInference.infer(program)
  return generateTypeScript(parseResult.statements, {
    typeInferenceResult: typeResult,
  })
}

export async function compileAndExecute(source: string): Promise<string> {
  // 一時的なSeseragiファイルを作成
  const tmpFile = `/tmp/seseragi-test-${Date.now()}.ssrg`
  await Bun.write(tmpFile, source)

  // bun src/cli.ts run で実行
  const proc = Bun.spawn(["bun", "src/cli.ts", "run", tmpFile], {
    stdout: "pipe",
    stderr: "pipe",
    cwd: "/Users/morishita/Desktop/seseragi",
  })

  const output = await new Response(proc.stdout).text()
  const exitCode = await proc.exited

  // 一時ファイルを削除
  try {
    await Bun.write(tmpFile, "")
  } catch (_e) {
    // ignore cleanup errors
  }

  if (exitCode !== 0) {
    const error = await new Response(proc.stderr).text()
    throw new Error(`Execution failed: ${error}`)
  }

  // デバッグログを除去して実際の出力のみを返す
  return extractActualOutput(output)
}

function extractActualOutput(rawOutput: string): string {
  const lines = rawOutput.split("\n")
  const outputLines: string[] = []
  let inRunningSection = false

  for (const line of lines) {
    if (line.trim() === "Running...") {
      inRunningSection = true
      continue
    }

    if (inRunningSection) {
      // 空行以外を実際の出力として扱う
      if (line.trim() !== "") {
        outputLines.push(line)
      }
    }
  }

  return outputLines.join("\n")
}
