#!/usr/bin/env bun

/**
 * Seseragi Programming Language Compiler
 * Main entry point
 */

import { Parser } from "./parser"
import { Lexer } from "./lexer"
import { generateTypeScript } from "./codegen"
import { infer } from "./type-inference"
import { TypeChecker } from "./typechecker"
import * as AST from "./ast"

// テスト用のexport関数
export function compileSeseragi(source: string): string {
  const parser = new Parser(source)
  const parseResult = parser.parse()

  if (parseResult.errors && parseResult.errors.length > 0) {
    throw new Error(parseResult.errors.map((e) => e.message).join("\n"))
  }

  // 型推論
  const inferenceResult = infer(parseResult.statements!)

  if (inferenceResult.errors.length > 0) {
    throw new Error(inferenceResult.errors.map((e) => e.message).join("\n"))
  }

  // 型チェック
  const program = new AST.Program(parseResult.statements!, 1, 1)
  const typeChecker = new TypeChecker(inferenceResult.typeEnvironment)
  const errors = typeChecker.check(program)

  if (errors.length > 0) {
    throw new Error(errors.map((e) => e.message).join("\n"))
  }

  // コード生成
  return generateTypeScript(parseResult.statements!, {
    typeInferenceResult: inferenceResult,
  })
}

console.log("Seseragi Compiler v1.0.0")

// コマンドライン引数からファイルを読み込み
const args = process.argv.slice(2)
let sourceCodeList: string[] = []

if (args.length > 0 && args[0].endsWith(".ssrg")) {
  try {
    const fs = require("fs")
    const fileContent = fs.readFileSync(args[0], "utf8")
    sourceCodeList = [fileContent]
    console.log(`ファイル ${args[0]} を読み込みました`)
  } catch (error) {
    console.error(`ファイル読み込みエラー: ${error}`)
    process.exit(1)
  }
} else {
  // デフォルトのテストケース
  sourceCodeList = [
    "fn add a :Int -> b :Int -> Int = a + b",
    "let x :Int = 42",
    "fn double x :Int -> Int = x * 2",
    "fn isEven n :Int -> Bool = n % 2 == 0",
  ]
}

for (const source of sourceCodeList) {
  console.log("\n" + "=".repeat(50))
  console.log("Seseragiコード:", source)
  console.log("-".repeat(30))

  try {
    // 字句解析
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()
    console.log(
      "トークン:",
      tokens.map((t) => `${t.type}:${t.value}`).join(" ")
    )

    // 構文解析
    const parser = new Parser(source)
    const program = parser.parse()
    console.log("AST:", program.statements.length, "個の文")

    // 型推論
    console.log("型推論を実行中...")
    const inferenceResult = infer(program.statements!)

    if (inferenceResult.errors.length > 0) {
      console.error("型推論エラー:")
      inferenceResult.errors.forEach((error) => console.error(error.message))
      continue
    }

    // TypeScriptコード生成
    const tsCode = generateTypeScript(program.statements, {
      typeInferenceResult: inferenceResult,
    })
    console.log("生成されたTypeScript:")
    console.log(tsCode)
  } catch (error) {
    console.error("エラー:", error)
  }
}
