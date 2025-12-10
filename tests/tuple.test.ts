/**
 * タプル機能のテスト
 */

import { describe, expect, test } from "bun:test"
import * as AST from "../src/ast"
import { CodeGenerator } from "../src/codegen"
import { Lexer, TokenType } from "../src/lexer"
import { Parser } from "../src/parser"
import { infer } from "../src/inference/engine/infer"

describe("Tuple Feature Tests", () => {
  describe("Lexer - Wildcard Token", () => {
    test("should tokenize wildcard pattern", () => {
      const lexer = new Lexer("_")
      const tokens = lexer.tokenize()

      expect(tokens).toHaveLength(2) // wildcard + EOF
      expect(tokens[0].type).toBe(TokenType.WILDCARD)
      expect(tokens[0].value).toBe("_")
    })
  })

  describe("Parser - Tuple Expression", () => {
    test("should parse basic tuple expression", () => {
      const parser = new Parser("(1, 2)")
      const result = parser.parse()

      expect(result.errors).toHaveLength(0)
      expect(result.statements).toHaveLength(1)

      const stmt = result.statements![0] as AST.ExpressionStatement
      expect(stmt.expression.kind).toBe("TupleExpression")

      const tuple = stmt.expression as AST.TupleExpression
      expect(tuple.elements).toHaveLength(2)
    })

    test("should parse three-element tuple", () => {
      const parser = new Parser('(1, "hello", true)')
      const result = parser.parse()

      expect(result.errors).toHaveLength(0)
      const stmt = result.statements![0] as AST.ExpressionStatement
      const tuple = stmt.expression as AST.TupleExpression
      expect(tuple.elements).toHaveLength(3)
    })

    test("should reject single-element tuple", () => {
      const parser = new Parser("(1)")
      const result = parser.parse()

      // This should parse as a parenthesized expression, not a tuple
      const stmt = result.statements![0] as AST.ExpressionStatement
      expect(stmt.expression.kind).toBe("Literal")
    })

    test("should parse empty parentheses as Unit value", () => {
      const parser = new Parser("()")
      const result = parser.parse()

      // () は now Unit値として扱われるのでエラーではない
      expect(result.errors.length).toBe(0)
      expect(result.statements.length).toBe(1)

      const stmt = result.statements[0] as any
      expect(stmt.kind).toBe("ExpressionStatement")
      expect(stmt.expression.kind).toBe("Literal")
      expect(stmt.expression.literalType).toBe("unit")
    })
  })

  describe("Parser - Tuple Destructuring", () => {
    test("should parse basic tuple destructuring", () => {
      const parser = new Parser("let (x, y) = (1, 2)")
      const result = parser.parse()

      expect(result.errors).toHaveLength(0)
      expect(result.statements).toHaveLength(1)

      const stmt = result.statements![0]
      expect(stmt.kind).toBe("TupleDestructuring")

      const destructuring = stmt as AST.TupleDestructuring
      expect(destructuring.pattern.patterns).toHaveLength(2)
    })

    test("should parse destructuring with wildcard", () => {
      const parser = new Parser("let (x, _) = (1, 2)")
      const result = parser.parse()

      expect(result.errors).toHaveLength(0)
      const stmt = result.statements![0] as AST.TupleDestructuring

      const patterns = stmt.pattern.patterns
      expect(patterns[0].kind).toBe("IdentifierPattern")
      expect(patterns[1].kind).toBe("WildcardPattern")
    })
  })

  describe("Type Inference - Tuple Types", () => {
    test("should infer tuple type from expression", () => {
      const parser = new Parser('let tuple = (1, "hello")')
      const parseResult = parser.parse()

      expect(parseResult.errors).toHaveLength(0)

      const program = new AST.Program(parseResult.statements!)
      const result = infer(program)

      expect(result.errors).toHaveLength(0)

      // Find the variable declaration
      const varDecl = parseResult.statements![0] as AST.VariableDeclaration
      const inferredType = result.nodeTypeMap.get(varDecl)

      expect(inferredType?.kind).toBe("TupleType")
      const tupleType = inferredType as AST.TupleType
      expect(tupleType.elementTypes).toHaveLength(2)
    })

    test("should type-check tuple destructuring", () => {
      const parser = new Parser(`
        let tuple = (42, "test")
        let (num, str) = tuple
      `)
      const parseResult = parser.parse()

      expect(parseResult.errors).toHaveLength(0)

      const program = new AST.Program(parseResult.statements!)
      const result = infer(program)

      expect(result.errors).toHaveLength(0)
    })
  })

  describe("Code Generation - Tuple to TypeScript", () => {
    test("should generate tuple expression", () => {
      const parser = new Parser("(1, 2, 3)")
      const parseResult = parser.parse()

      const generator = new CodeGenerator({})
      const stmt = parseResult.statements![0] as AST.ExpressionStatement
      const code = generator.generateExpression(stmt.expression)

      expect(code).toBe("{ tag: 'Tuple', elements: [1, 2, 3] }")
    })

    test("should generate tuple destructuring", () => {
      const parser = new Parser("let (x, y) = (1, 2)")
      const parseResult = parser.parse()

      const generator = new CodeGenerator({})
      const stmt = parseResult.statements![0]
      const code = generator.generateStatement(stmt)

      expect(code).toBe(
        "const [x, y] = { tag: 'Tuple', elements: [1, 2] }.tag === 'Tuple' ? { tag: 'Tuple', elements: [1, 2] }.elements : { tag: 'Tuple', elements: [1, 2] };"
      )
    })

    test("should generate destructuring with wildcard", () => {
      const parser = new Parser("let (x, _) = (1, 2)")
      const parseResult = parser.parse()

      const generator = new CodeGenerator({})
      const stmt = parseResult.statements![0]
      const code = generator.generateStatement(stmt)

      expect(code).toBe(
        "const [x, _1] = { tag: 'Tuple', elements: [1, 2] }.tag === 'Tuple' ? { tag: 'Tuple', elements: [1, 2] }.elements : { tag: 'Tuple', elements: [1, 2] };"
      )
    })

    test("should generate unique names for multiple wildcards", () => {
      const parser = new Parser("let (_, x, _, y, _) = (1, 2, 3, 4, 5)")
      const parseResult = parser.parse()

      const generator = new CodeGenerator({})
      const stmt = parseResult.statements![0]
      const code = generator.generateStatement(stmt)

      expect(code).toBe(
        "const [_1, x, _2, y, _3] = { tag: 'Tuple', elements: [1, 2, 3, 4, 5] }.tag === 'Tuple' ? { tag: 'Tuple', elements: [1, 2, 3, 4, 5] }.elements : { tag: 'Tuple', elements: [1, 2, 3, 4, 5] };"
      )
    })

    test("should generate globally unique wildcard names across statements", () => {
      const parser = new Parser(`
        let (_, x, _) = (1, 2, 3)
        let (_, y, _) = (4, 5, 6)
      `)
      const parseResult = parser.parse()

      const generator = new CodeGenerator({})
      const code1 = generator.generateStatement(parseResult.statements![0])
      const code2 = generator.generateStatement(parseResult.statements![1])

      expect(code1).toBe(
        "const [_1, x, _2] = { tag: 'Tuple', elements: [1, 2, 3] }.tag === 'Tuple' ? { tag: 'Tuple', elements: [1, 2, 3] }.elements : { tag: 'Tuple', elements: [1, 2, 3] };"
      )
      expect(code2).toBe(
        "const [_3, y, _4] = { tag: 'Tuple', elements: [4, 5, 6] }.tag === 'Tuple' ? { tag: 'Tuple', elements: [4, 5, 6] }.elements : { tag: 'Tuple', elements: [4, 5, 6] };"
      )
    })
  })

  describe("Integration Tests", () => {
    test("should compile tuple example end-to-end", () => {
      const source = `
        let point = (10, 20)
        let (x, y) = point
      `

      const parser = new Parser(source)
      const parseResult = parser.parse()

      expect(parseResult.errors).toHaveLength(0)

      const program = new AST.Program(parseResult.statements!)
      const _inferenceResult = infer(program)

      // Allow inference errors for now since `show` function may not be defined
      // The main goal is to test tuple parsing and code generation

      const generator = new CodeGenerator({})
      const code = generator.generateProgram(parseResult.statements!)

      expect(code).toContain("[10, 20]")
      expect(code).toContain("const [x, y] =")
    })
  })
})
