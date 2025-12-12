// テスト: ??演算子（Nullish Coalescing）

import { describe, expect, test } from "bun:test"
import * as AST from "../src/ast"
import { generateTypeScript } from "../src/codegen"
import { infer } from "../src/inference/engine/infer"
import { lex, TokenType } from "../src/lexer"
import { Parser } from "../src/parser"
import { compileSeseragi } from "./test-utils"

describe("Nullish Coalescing Operator", () => {
  test("should tokenize ?? operator correctly", () => {
    const tokens = lex("a ?? b")

    expect(tokens).toHaveLength(4) // a, ??, b, EOF
    expect(tokens[0].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[0].value).toBe("a")
    expect(tokens[1].type).toBe(TokenType.NULLISH_COALESCING)
    expect(tokens[1].value).toBe("??")
    expect(tokens[2].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[2].value).toBe("b")
  })

  test("should parse ?? operator with correct precedence", () => {
    const parser = new Parser("let result = value ?? 0")
    const parseResult = parser.parse()

    expect(parseResult.errors).toHaveLength(0)
    expect(parseResult.statements).toHaveLength(1)

    const stmt = parseResult.statements![0]
    expect(stmt.kind).toBe("VariableDeclaration")

    const varDecl = stmt as any
    expect(varDecl.initializer.kind).toBe("NullishCoalescingExpression")
  })

  test("should handle Maybe type with ?? operator", () => {
    const source = `
      let maybeValue = Just 42
      let result = maybeValue ?? 0
    `

    const parser = new Parser(source)
    const parseResult = parser.parse()
    expect(parseResult.errors).toHaveLength(0)

    const program = new AST.Program(parseResult.statements!)
    const typeResult = infer(program)
    expect(typeResult.errors).toHaveLength(0)

    // 型推論結果を渡す
    const tsCode = generateTypeScript(parseResult.statements!, {
      typeInferenceResult: typeResult,
    })
    expect(tsCode).toContain("fromMaybe(0, maybeValue)")
  })

  test("should handle Either type with ?? operator", () => {
    const source = `
      let eitherValue = Right 100
      let result = eitherValue ?? -1
    `

    const parser = new Parser(source)
    const parseResult = parser.parse()
    expect(parseResult.errors).toHaveLength(0)

    const program = new AST.Program(parseResult.statements!)
    const typeResult = infer(program)
    expect(typeResult.errors).toHaveLength(0)

    // 型推論結果を渡す
    const tsCode = generateTypeScript(parseResult.statements!, {
      typeInferenceResult: typeResult,
    })
    expect(tsCode).toContain("fromRight(-1, eitherValue)")
  })

  test("should handle chained ?? operators", () => {
    const source = `
      let a: Maybe<Int> = Nothing
      let b: Maybe<Int> = Just 10
      let result = (a ?? b) ?? 0
    `

    const parser = new Parser(source)
    const parseResult = parser.parse()
    expect(parseResult.errors).toHaveLength(0)

    const program = new AST.Program(parseResult.statements!)
    const typeResult = infer(program)
    expect(typeResult.errors).toHaveLength(0)
  })

  test("should generate correct TypeScript for runtime functions", () => {
    const source = `
      let value1: Maybe<Int> = Just 42
      let value2: Maybe<Int> = Nothing
      let result1 = value1 ?? 0
      let result2 = value2 ?? 0
    `

    const parser = new Parser(source)
    const parseResult = parser.parse()

    const program = new AST.Program(parseResult.statements!)
    const typeResult = infer(program)
    expect(typeResult.errors).toHaveLength(0)

    // Force embedded runtime mode + 型推論結果を渡す
    const tsCode = generateTypeScript(parseResult.statements!, {
      runtimeMode: "embedded",
      typeInferenceResult: typeResult,
    })

    // ランタイム関数が含まれているかチェック
    expect(tsCode).toContain("function fromMaybe")
    expect(tsCode).toContain(
      'maybe.tag === "Just" ? maybe.value : defaultValue'
    )

    // 正しい関数呼び出しが生成されているかチェック
    expect(tsCode).toContain("fromMaybe(0, value1)")
    expect(tsCode).toContain("fromMaybe(0, value2)")
  })

  test("should handle function application with ?? operator", () => {
    const source = `
      fn getValue x: Int -> Maybe<Int> = Just x
      let maybeVal = getValue 10
      let result = maybeVal ?? 0
    `

    const tsCode = compileSeseragi(source)
    // 明示的型注釈により正しくfromMaybeが生成される
    expect(tsCode).toContain("fromMaybe(0, maybeVal)")
  })

  test("should handle pipeline with ?? operator", () => {
    const source = `
      fn getValue x: Int -> Maybe<Int> = Just x
      let maybeVal = 10 | getValue
      let result = maybeVal ?? 0
    `

    const tsCode = compileSeseragi(source)
    // 明示的型注釈により正しくfromMaybeが生成される
    expect(tsCode).toContain("fromMaybe(0, maybeVal)")
  })

  test("should handle function returning Maybe with ?? fallback", () => {
    const source = `
      fn safeDivide a: Int -> b: Int -> Maybe<Int> =
        if b == 0 then Nothing else Just (a / b)

      let result1 = safeDivide 10 2 ?? -1
      let result2 = safeDivide 10 0 ?? -1
    `

    const tsCode = compileSeseragi(source)
    // 明示的型注釈により正しくfromMaybeが生成される
    expect(tsCode).toContain("fromMaybe(-1,")
  })
})
