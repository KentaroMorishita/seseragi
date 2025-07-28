import { test, expect } from "bun:test"
import { infer } from "../src/type-inference"
import { parse } from "../src/parser"
import { lex } from "../src/lexer"

// Task の基本的な型推論テスト
test("Task type inference with resolve", () => {
  const code = "let a = Task $ resolve<Int> 100"
  const tokens = lex(code)
  const ast = parse(tokens)
  if (!ast.statements) throw new Error("Parse failed")
  const result = infer(ast.statements)

  expect(result.errors).toHaveLength(0)
})

test("Task type inference with reject", () => {
  const code = 'let b = Task $ reject<String> "error"'
  const tokens = lex(code)
  const ast = parse(tokens)
  if (!ast.statements) throw new Error("Parse failed")
  const result = infer(ast.statements)

  expect(result.errors).toHaveLength(0)
})

test("Task function type annotation", () => {
  const code = "fn main -> Task<Int> = Task $ resolve 100"
  const tokens = lex(code)
  const ast = parse(tokens)
  if (!ast.statements) throw new Error("Parse failed")
  const result = infer(ast.statements)

  expect(result.errors).toHaveLength(0)
})
