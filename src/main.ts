#!/usr/bin/env bun

/**
 * Seseragi Programming Language Compiler
 * Main entry point
 */

import { Parser } from "./parser"
import { Lexer } from "./lexer"

console.log("Seseragi Compiler v1.0.0")

// Simple test
try {
  const source = "fn add a :Int -> b :Int -> Int = a + b"
  console.log("Parsing:", source)

  // First, let's see the tokens
  const lexer = new Lexer(source)
  const tokens = lexer.tokenize()
  console.log(
    "Tokens:",
    tokens.map((t) => `${t.type}:${t.value}`)
  )

  const parser = new Parser(source)
  const program = parser.parse()

  console.log("Success! Parsed", program.statements.length, "statements")
  console.log(JSON.stringify(program, null, 2))
} catch (error) {
  console.error("Parse error:", error)
}
