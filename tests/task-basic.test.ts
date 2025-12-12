import { expect, test } from "bun:test"
import * as AST from "../src/ast"
import { infer } from "../src/inference/engine/infer"
import { lex } from "../src/lexer"
import { parse } from "../src/parser"

// Task の基本的な型推論テスト
test("Task type inference with resolve", () => {
  const code = "let a = Task $ resolve<Int> 100"
  const tokens = lex(code)
  const ast = parse(tokens)
  if (!ast.statements) throw new Error("Parse failed")
  const program = new AST.Program(ast.statements)
  const result = infer(program)

  expect(result.errors).toHaveLength(0)
})

test("Task type inference with reject", () => {
  const code = 'let b = Task $ reject<String> "error"'
  const tokens = lex(code)
  const ast = parse(tokens)
  if (!ast.statements) throw new Error("Parse failed")
  const program = new AST.Program(ast.statements)
  const result = infer(program)

  expect(result.errors).toHaveLength(0)
})

test("Task function type annotation", () => {
  const code = "fn main -> Task<Int> = Task $ resolve 100"
  const tokens = lex(code)
  const ast = parse(tokens)
  if (!ast.statements) throw new Error("Parse failed")
  const program = new AST.Program(ast.statements)
  const result = infer(program)

  expect(result.errors).toHaveLength(0)
})
