import { test, expect } from "bun:test"
import { Parser } from "../src/parser"
import { generateTypeScript } from "../src/codegen"

test("Either - should parse Left constructor", () => {
  const source = 'let errorValue :Either<String, Int> = Left "Error message"'
  const parser = new Parser(source)
  const ast = parser.parse()

  expect(ast.statements).toHaveLength(1)
  const stmt = ast.statements[0]
  expect(stmt.kind).toBe("VariableDeclaration")
})

test("Either - should parse Right constructor", () => {
  const source = "let successValue :Either<String, Int> = Right 42"
  const parser = new Parser(source)
  const ast = parser.parse()

  expect(ast.statements).toHaveLength(1)
  const stmt = ast.statements[0]
  expect(stmt.kind).toBe("VariableDeclaration")
})

test("Either - should generate TypeScript for Left", () => {
  const source = 'let errorValue = Left "Error message"'
  const parser = new Parser(source)
  const ast = parser.parse()
  const tsCode = generateTypeScript(ast.statements)

  expect(tsCode).toContain('Left("Error message")')
})

test("Either - should generate TypeScript for Right", () => {
  const source = "let successValue = Right 42"
  const parser = new Parser(source)
  const ast = parser.parse()
  const tsCode = generateTypeScript(ast.statements)

  expect(tsCode).toContain("Right(42)")
})

test("Either - should handle pattern matching", () => {
  const source = `
    match eitherValue {
      Left error -> print error
      Right value -> print value
    }
  `
  const parser = new Parser(source)
  const ast = parser.parse()

  expect(ast.statements).toHaveLength(1)
  const stmt = ast.statements[0]
  expect(stmt.kind).toBe("ExpressionStatement")
})
