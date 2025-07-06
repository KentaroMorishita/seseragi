import { describe, test, expect } from "bun:test"
import { Lexer, TokenType } from "../src/lexer"
import { Parser } from "../src/parser"
import * as AST from "../src/ast"
import { TypeInferenceSystem } from "../src/type-inference"
import { generateTypeScript } from "../src/codegen"

describe("Template Literal Tests", () => {
  test("should tokenize simple template literal", () => {
    const source = "`Hello World`"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    expect(tokens[0].type).toBe(TokenType.TEMPLATE_STRING)
    expect(tokens[0].value).toBe("Hello World")
    expect(tokens[1].type).toBe(TokenType.EOF)
  })

  test("should tokenize template literal with expression", () => {
    const source = "`Hello ${name}!`"
    const lexer = new Lexer(source)
    const tokens = lexer.tokenize()

    expect(tokens[0].type).toBe(TokenType.TEMPLATE_STRING)
    expect(tokens[0].value).toBe("Hello ${name}!")
    expect(tokens[1].type).toBe(TokenType.EOF)
  })

  test("should parse simple template literal", () => {
    const source = "`Hello World`"
    const parser = new Parser(source)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const stmt = result.statements![0] as AST.ExpressionStatement
    expect(stmt.kind).toBe("ExpressionStatement")

    const expr = stmt.expression as AST.TemplateExpression
    expect(expr.kind).toBe("TemplateExpression")
    expect(expr.parts).toHaveLength(1)
    expect(expr.parts[0]).toBe("Hello World")
  })

  test("should parse template literal with embedded expression", () => {
    const source = "`Hello ${name}!`"
    const parser = new Parser(source)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const stmt = result.statements![0] as AST.ExpressionStatement
    const expr = stmt.expression as AST.TemplateExpression
    expect(expr.kind).toBe("TemplateExpression")
    expect(expr.parts).toHaveLength(3)
    expect(expr.parts[0]).toBe("Hello ")
    expect((expr.parts[1] as AST.Identifier).name).toBe("name")
    expect(expr.parts[2]).toBe("!")
  })

  test("should infer String type for template literal", () => {
    const source = "`Hello World`"
    const parser = new Parser(source)
    const result = parser.parse()

    const stmt = result.statements![0] as AST.ExpressionStatement
    const templateExpr = stmt.expression as AST.TemplateExpression

    const system = new TypeInferenceSystem()
    const env = new Map()
    const resultType = system.generateConstraintsForExpression(
      templateExpr,
      env
    )

    expect(resultType.kind).toBe("PrimitiveType")
    expect((resultType as AST.PrimitiveType).name).toBe("String")
  })

  test("should generate TypeScript template literal", () => {
    const source = "`Hello World`"
    const parser = new Parser(source)
    const result = parser.parse()

    const generatedCode = generateTypeScript(result.statements!)
    expect(generatedCode).toContain("`Hello World`")
  })

  test("should generate TypeScript template literal with expression", () => {
    const source = "`Hello ${name}!`"
    const parser = new Parser(source)
    const result = parser.parse()

    const generatedCode = generateTypeScript(result.statements!)
    expect(generatedCode).toContain("`Hello ")
    expect(generatedCode).toContain("${name}")
    expect(generatedCode).toContain("!`")
  })

  test("should parse template literal as function argument", () => {
    const source = "show `Hello World`"
    const parser = new Parser(source)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const stmt = result.statements![0] as AST.ExpressionStatement
    expect(stmt.kind).toBe("ExpressionStatement")

    const expr = stmt.expression as AST.FunctionApplication
    expect(expr.kind).toBe("FunctionApplication")
    expect((expr.function as AST.Identifier).name).toBe("show")

    const arg = expr.argument as AST.TemplateExpression
    expect(arg.kind).toBe("TemplateExpression")
    expect(arg.parts[0]).toBe("Hello World")
  })

  test("should parse template literal with expression as function argument", () => {
    const source = "print `Count: ${42}`"
    const parser = new Parser(source)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const stmt = result.statements![0] as AST.ExpressionStatement
    const expr = stmt.expression as AST.FunctionApplication
    expect(expr.kind).toBe("FunctionApplication")
    expect((expr.function as AST.Identifier).name).toBe("print")

    const arg = expr.argument as AST.TemplateExpression
    expect(arg.kind).toBe("TemplateExpression")
    expect(arg.parts).toHaveLength(2)
    expect(arg.parts[0]).toBe("Count: ")
    expect((arg.parts[1] as AST.Literal).value).toBe(42)
  })

  test("should generate correct code for template literal function application", () => {
    const source = "show `Hello World`"
    const parser = new Parser(source)
    const result = parser.parse()

    const generatedCode = generateTypeScript(result.statements!)
    expect(generatedCode).toContain("show(`Hello World`)")
  })
})
