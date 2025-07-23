/**
 * 範囲指定とリスト内包表記のテスト
 */

import { describe, expect, test } from "bun:test"
import type * as AST from "../src/ast"
import { Lexer, TokenType } from "../src/lexer"
import { Parser } from "../src/parser"

describe("Range Literals", () => {
  test("basic range parsing", () => {
    const source = "1..10"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()
    const parser = new Parser(tokens)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const stmt = result.statements![0] as AST.ExpressionStatement
    const expr = stmt.expression as AST.RangeLiteral

    expect(expr.kind).toBe("RangeLiteral")
    expect(expr.inclusive).toBe(false)
    expect((expr.start as AST.Literal).value).toBe(1)
    expect((expr.end as AST.Literal).value).toBe(10)
  })

  test("inclusive range parsing", () => {
    const source = "1..=5"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()
    const parser = new Parser(tokens)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const stmt = result.statements![0] as AST.ExpressionStatement
    const expr = stmt.expression as AST.RangeLiteral

    expect(expr.kind).toBe("RangeLiteral")
    expect(expr.inclusive).toBe(true)
    expect((expr.start as AST.Literal).value).toBe(1)
    expect((expr.end as AST.Literal).value).toBe(5)
  })

  test("range with variables", () => {
    const source = "start..end"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()
    const parser = new Parser(tokens)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const stmt = result.statements![0] as AST.ExpressionStatement
    const expr = stmt.expression as AST.RangeLiteral

    expect(expr.kind).toBe("RangeLiteral")
    expect(expr.inclusive).toBe(false)
    expect((expr.start as AST.Identifier).name).toBe("start")
    expect((expr.end as AST.Identifier).name).toBe("end")
  })
})

describe("List Comprehensions", () => {
  test("basic list comprehension parsing", () => {
    const source = "[x | x <- numbers]"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()
    const parser = new Parser(tokens)
    const result = parser.parse()

    if (result.errors.length > 0) {
      console.log(
        "Parse errors:",
        result.errors.map((e) => e.message)
      )
    }
    console.log("Parsed statements:", result.statements?.length)
    if (result.statements && result.statements.length > 0) {
      console.log("First statement:", result.statements[0])
    }

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const stmt = result.statements![0] as AST.ExpressionStatement
    const expr = stmt.expression as AST.ListComprehension

    expect(expr.kind).toBe("ListComprehension")
    expect((expr.expression as AST.Identifier).name).toBe("x")
    expect(expr.generators).toHaveLength(1)
    expect(expr.generators[0].variable).toBe("x")
    expect((expr.generators[0].iterable as AST.Identifier).name).toBe("numbers")
    expect(expr.filters).toHaveLength(0)
  })

  test("list comprehension with expression", () => {
    const source = "[x * 2 | x <- numbers]"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()
    const parser = new Parser(tokens)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const stmt = result.statements![0] as AST.ExpressionStatement
    const expr = stmt.expression as AST.ListComprehension

    expect(expr.kind).toBe("ListComprehension")

    const mulExpr = expr.expression as AST.BinaryOperation
    expect(mulExpr.kind).toBe("BinaryOperation")
    expect(mulExpr.operator).toBe("*")
    expect((mulExpr.left as AST.Identifier).name).toBe("x")
    expect((mulExpr.right as AST.Literal).value).toBe(2)
  })

  test("list comprehension with filter", () => {
    const source = "[x | x <- numbers, x % 2 == 0]"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()
    const parser = new Parser(tokens)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const stmt = result.statements![0] as AST.ExpressionStatement
    const expr = stmt.expression as AST.ListComprehension

    expect(expr.kind).toBe("ListComprehension")
    expect(expr.generators).toHaveLength(1)
    expect(expr.filters).toHaveLength(1)

    const filter = expr.filters[0] as AST.BinaryOperation
    expect(filter.operator).toBe("==")
  })

  test("list comprehension with range", () => {
    const source = "[x * x | x <- 1..10]"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()
    const parser = new Parser(tokens)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const stmt = result.statements![0] as AST.ExpressionStatement
    const expr = stmt.expression as AST.ListComprehension

    expect(expr.kind).toBe("ListComprehension")
    expect(expr.generators).toHaveLength(1)

    const generator = expr.generators[0]
    expect(generator.variable).toBe("x")
    expect(generator.iterable.kind).toBe("RangeLiteral")

    const range = generator.iterable as AST.RangeLiteral
    expect((range.start as AST.Literal).value).toBe(1)
    expect((range.end as AST.Literal).value).toBe(10)
  })

  test("list comprehension with multiple generators", () => {
    const source = "[x + y | x <- 1..3, y <- 1..2]"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()
    const parser = new Parser(tokens)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const stmt = result.statements![0] as AST.ExpressionStatement
    const expr = stmt.expression as AST.ListComprehension

    expect(expr.kind).toBe("ListComprehension")
    expect(expr.generators).toHaveLength(2)
    expect(expr.generators[0].variable).toBe("x")
    expect(expr.generators[1].variable).toBe("y")
  })
})

describe("Token Recognition", () => {
  test("range operators are tokenized correctly", () => {
    const source = "1..10 1..=5"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    expect(tokens).toHaveLength(7) // 1, .., 10, 1, ..=, 5, EOF
    expect(tokens[1].type).toBe(TokenType.RANGE)
    expect(tokens[1].value).toBe("..")
    expect(tokens[4].type).toBe(TokenType.RANGE_INCLUSIVE)
    expect(tokens[4].value).toBe("..=")
  })

  test("generator operator is tokenized correctly", () => {
    const source = "x <- list"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    expect(tokens).toHaveLength(4) // x, <-, list, EOF
    expect(tokens[1].type).toBe(TokenType.GENERATOR)
    expect(tokens[1].value).toBe("<-")
  })
})
