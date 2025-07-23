import { describe, expect, it } from "bun:test"
import { Lexer, TokenType } from "../src/lexer"

describe("Lexer", () => {
  it("should tokenize basic input", () => {
    const source = "let x = 10"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    expect(tokens).toHaveLength(5) // let, x, =, 10, EOF
    expect(tokens[0].type).toBe(TokenType.LET)
    expect(tokens[1].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[1].value).toBe("x")
    expect(tokens[2].type).toBe(TokenType.ASSIGN)
    expect(tokens[3].type).toBe(TokenType.INTEGER)
    expect(tokens[3].value).toBe("10")
    expect(tokens[4].type).toBe(TokenType.EOF)
  })

  it("should handle keywords", () => {
    const source = "fn type let impl monoid"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    expect(tokens[0].type).toBe(TokenType.FN)
    expect(tokens[1].type).toBe(TokenType.TYPE)
    expect(tokens[2].type).toBe(TokenType.LET)
    expect(tokens[3].type).toBe(TokenType.IMPL)
    expect(tokens[4].type).toBe(TokenType.MONOID)
  })

  it("should handle operators", () => {
    const source = "| ~ >>= >>> -> == != < > <= >= + - * / %"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    const expectedTypes = [
      TokenType.PIPE,
      TokenType.REVERSE_PIPE,
      TokenType.BIND,
      TokenType.FOLD_MONOID,
      TokenType.ARROW,
      TokenType.EQUAL,
      TokenType.NOT_EQUAL,
      TokenType.LESS_THAN,
      TokenType.GREATER_THAN,
      TokenType.LESS_EQUAL,
      TokenType.GREATER_EQUAL,
      TokenType.PLUS,
      TokenType.MINUS,
      TokenType.MULTIPLY,
      TokenType.DIVIDE,
      TokenType.MODULO,
      TokenType.EOF,
    ]

    expectedTypes.forEach((expectedType, index) => {
      expect(tokens[index].type).toBe(expectedType)
    })
  })

  it("should handle function definition", () => {
    const source = "fn add a: Int -> b: Int -> Int = a + b"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    expect(tokens[0].type).toBe(TokenType.FN)
    expect(tokens[1].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[1].value).toBe("add")
    expect(tokens[2].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[2].value).toBe("a")
    expect(tokens[3].type).toBe(TokenType.COLON)
    expect(tokens[4].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[4].value).toBe("Int")
  })

  it("should handle strings", () => {
    const source = '"Hello, World!"'
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    expect(tokens[0].type).toBe(TokenType.STRING)
    expect(tokens[0].value).toBe("Hello, World!")
  })

  it("should handle numbers", () => {
    const source = "42 3.14"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    expect(tokens[0].type).toBe(TokenType.INTEGER)
    expect(tokens[0].value).toBe("42")
    expect(tokens[1].type).toBe(TokenType.FLOAT)
    expect(tokens[1].value).toBe("3.14")
  })

  it("should handle boolean literals", () => {
    const source = "True False"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    expect(tokens[0].type).toBe(TokenType.BOOLEAN)
    expect(tokens[0].value).toBe("True")
    expect(tokens[1].type).toBe(TokenType.BOOLEAN)
    expect(tokens[1].value).toBe("False")
  })

  it("should handle modulo operator in expressions", () => {
    const source = "x % y"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    expect(tokens[0].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[0].value).toBe("x")
    expect(tokens[1].type).toBe(TokenType.MODULO)
    expect(tokens[1].value).toBe("%")
    expect(tokens[2].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[2].value).toBe("y")
    expect(tokens[3].type).toBe(TokenType.EOF)
  })

  it("should handle identifiers with apostrophes (Haskell-style)", () => {
    const source = "x' y'' z''' f' g''"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    expect(tokens[0].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[0].value).toBe("x'")
    expect(tokens[1].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[1].value).toBe("y''")
    expect(tokens[2].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[2].value).toBe("z'''")
    expect(tokens[3].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[3].value).toBe("f'")
    expect(tokens[4].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[4].value).toBe("g''")
    expect(tokens[5].type).toBe(TokenType.EOF)
  })

  it("should handle function definition with apostrophes", () => {
    const source = "fn f' x = x + 1"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    expect(tokens[0].type).toBe(TokenType.FN)
    expect(tokens[1].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[1].value).toBe("f'")
    expect(tokens[2].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[2].value).toBe("x")
    expect(tokens[3].type).toBe(TokenType.ASSIGN)
  })

  it("should handle let binding with apostrophes", () => {
    const source = "let x' = 42"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    expect(tokens[0].type).toBe(TokenType.LET)
    expect(tokens[1].type).toBe(TokenType.IDENTIFIER)
    expect(tokens[1].value).toBe("x'")
    expect(tokens[2].type).toBe(TokenType.ASSIGN)
    expect(tokens[3].type).toBe(TokenType.INTEGER)
    expect(tokens[3].value).toBe("42")
  })
})
