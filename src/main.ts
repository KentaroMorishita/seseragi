#!/usr/bin/env bun

/**
 * Seseragi Programming Language Compiler
 * Main entry point
 */

import { Parser } from "./parser"
import { Lexer } from "./lexer"
import { generateTypeScript } from "./codegen"

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
