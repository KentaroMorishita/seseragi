import { expect, test } from "bun:test"
import { generateTypeScript } from "../src/codegen"
import { Parser } from "../src/parser"

test("Maybe - should parse Just constructor", () => {
  const source = "let maybeValue: Maybe<Int> = Just 42"
  const parser = new Parser(source)
  const parseResult = parser.parse()

  expect(parseResult.statements).toHaveLength(1)
  const stmt = parseResult.statements?.[0]
  expect(stmt.kind).toBe("VariableDeclaration")
})

test("Maybe - should parse Nothing constructor", () => {
  const source = "let emptyValue: Maybe<Int> = Nothing"
  const parser = new Parser(source)
  const parseResult = parser.parse()

  expect(parseResult.statements).toHaveLength(1)
  const stmt = parseResult.statements?.[0]
  expect(stmt.kind).toBe("VariableDeclaration")
})

test("Maybe - should generate TypeScript for Just", () => {
  const source = "let maybeValue = Just 42"
  const parser = new Parser(source)
  const parseResult = parser.parse()
  const tsCode = generateTypeScript(parseResult.statements || [])

  expect(tsCode).toContain("Just(42)")
})

test("Maybe - should generate TypeScript for Nothing", () => {
  const source = "let emptyValue = Nothing"
  const parser = new Parser(source)
  const parseResult = parser.parse()
  const tsCode = generateTypeScript(parseResult.statements || [])

  expect(tsCode).toContain("Nothing")
})

test("Maybe - should handle pattern matching", () => {
  const source = `
    match maybeValue {
      Just x -> x
      Nothing -> 0
    }
  `
  const parser = new Parser(source)
  const parseResult = parser.parse()

  expect(parseResult.statements).toHaveLength(1)
  const stmt = parseResult.statements?.[0]
  expect(stmt.kind).toBe("ExpressionStatement")
})
