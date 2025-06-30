#!/usr/bin/env bun

/**
 * Seseragi Programming Language Compiler
 * Main entry point
 */

import { Parser } from "./parser"
import { Lexer } from "./lexer"
import { generateTypeScript } from "./codegen"
import { TypeInferenceSystem } from "./type-inference"
import { TypeChecker } from "./typechecker"
import * as AST from "./ast"

// テスト用のexport関数
export function compileSeseragi(source: string): string {
  const parser = new Parser(source)
  const parseResult = parser.parse()
  
  if (parseResult.errors && parseResult.errors.length > 0) {
    throw new Error(parseResult.errors.map(e => e.message).join("\n"))
  }
  
  // 型推論
  const typeInference = new TypeInferenceSystem()
  const program: AST.Program = {
    kind: "Program",
    statements: parseResult.statements!,
    line: 0,
    column: 0
  }
  const inferenceResult = typeInference.infer(program)
  
  if (inferenceResult.errors.length > 0) {
    throw new Error(inferenceResult.errors.map(e => e.message).join("\n"))
  }
  
  // 型チェック
  const typeChecker = new TypeChecker(inferenceResult.typeEnvironment)
  const errors = typeChecker.check(program)
  
  if (errors.length > 0) {
    throw new Error(errors.map(e => e.message).join("\n"))
  }
  
  // コード生成
  return generateTypeScript(parseResult.statements!)
}

console.log("Seseragi Compiler v1.0.0")

// テスト用のSeseragiコード
const testCases = [
  "fn add a :Int -> b :Int -> Int = a + b",
  "let x :Int = 42",
  "fn double x :Int -> Int = x * 2",
  "fn isEven n :Int -> Bool = n % 2 == 0",
]

for (const source of testCases) {
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

    // TypeScriptコード生成
    const tsCode = generateTypeScript(program.statements)
    console.log("生成されたTypeScript:")
    console.log(tsCode)
  } catch (error) {
    console.error("エラー:", error)
  }
}
