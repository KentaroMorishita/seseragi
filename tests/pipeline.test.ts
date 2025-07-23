import { describe, expect, it } from "bun:test"
import { generateTypeScript } from "../src/codegen"
import { Lexer, TokenType } from "../src/lexer"
import { Parser } from "../src/parser"

describe("Pipeline Operator", () => {
  it("should parse pipeline operator expressions", () => {
    const source = "x | double"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    expect(tokens[0].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[0].value).toBe("x")
    expect(tokens[1].type).toBe(TokenType.PIPE)
    expect(tokens[1].value).toBe("|")
    expect(tokens[2].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[2].value).toBe("double")
  })

  it("should generate TypeScript code for pipeline operations", () => {
    const source = "let result = x | double"
    const parser = new Parser(source)
    const program = parser.parse()
    const tsCode = generateTypeScript(program.statements)

    expect(tsCode).toContain("pipe(")
    expect(tsCode).toContain("x")
    expect(tsCode).toContain("double")
  })

  it("should handle chained pipeline operations", () => {
    const source = "x | double | add"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    // x | double | add should tokenize correctly
    expect(tokens[0].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[1].type).toBe(TokenType.PIPE)
    expect(tokens[2].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[3].type).toBe(TokenType.PIPE)
    expect(tokens[4].type).toBe(TokenType.IDENTIFIER)
  })
})
