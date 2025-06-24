import { describe, it, expect } from "bun:test"
import { Parser } from "../src/parser"
import { generateTypeScript } from "../src/codegen"
import type * as AST from "../src/ast"

describe("Function Application Operator ($)", () => {
  it("should tokenize $ operator", () => {
    const source = "f $ x"
    const parser = new Parser(source)
    const program = parser.parse()

    expect(program.statements).toHaveLength(1)
    const stmt = program.statements[0] as AST.ExpressionStatement
    const expr = stmt.expression as AST.FunctionApplicationOperator

    expect(expr.kind).toBe("FunctionApplicationOperator")
    expect((expr.left as AST.Identifier).name).toBe("f")
    expect((expr.right as AST.Identifier).name).toBe("x")
  })

  it("should parse right-associative $ operator", () => {
    const source = "f $ g $ x"
    const parser = new Parser(source)
    const program = parser.parse()

    const stmt = program.statements[0] as AST.ExpressionStatement
    const expr = stmt.expression as AST.FunctionApplicationOperator

    expect(expr.kind).toBe("FunctionApplicationOperator")
    expect((expr.left as AST.Identifier).name).toBe("f")
    
    // Right side should be another FunctionApplicationOperator
    const right = expr.right as AST.FunctionApplicationOperator
    expect(right.kind).toBe("FunctionApplicationOperator")
    expect((right.left as AST.Identifier).name).toBe("g")
    expect((right.right as AST.Identifier).name).toBe("x")
  })

  it("should parse $ with function calls", () => {
    const source = "print $ toString $ add 10 5"
    const parser = new Parser(source)
    const program = parser.parse()

    const stmt = program.statements[0] as AST.ExpressionStatement
    const expr = stmt.expression as AST.FunctionApplicationOperator

    expect(expr.kind).toBe("FunctionApplicationOperator")
    expect((expr.left as AST.Identifier).name).toBe("print")
    
    // Right side should be another function application
    const right = expr.right as AST.FunctionApplicationOperator
    expect(right.kind).toBe("FunctionApplicationOperator")
    expect((right.left as AST.Identifier).name).toBe("toString")
  })

  it("should have correct precedence (lowest)", () => {
    const source = "f $ x + y"
    const parser = new Parser(source)
    const program = parser.parse()

    const stmt = program.statements[0] as AST.ExpressionStatement
    const expr = stmt.expression as AST.FunctionApplicationOperator

    expect(expr.kind).toBe("FunctionApplicationOperator")
    expect((expr.left as AST.Identifier).name).toBe("f")
    
    // Right side should be the binary operation x + y
    const right = expr.right as AST.BinaryOperation
    expect(right.kind).toBe("BinaryOperation")
    expect(right.operator).toBe("+")
    expect((right.left as AST.Identifier).name).toBe("x")
    expect((right.right as AST.Identifier).name).toBe("y")
  })

  it("should generate correct TypeScript code", () => {
    const source = "f $ x"
    const parser = new Parser(source)
    const program = parser.parse()
    const generated = generateTypeScript(program.statements)

    expect(generated).toContain("f(x)")
  })

  it("should generate correct TypeScript for nested applications", () => {
    const source = "f $ g $ x"
    const parser = new Parser(source)
    const program = parser.parse()
    const generated = generateTypeScript(program.statements)

    // Should generate f(g(x)) due to right-associativity
    expect(generated).toContain("f(g(x))")
  })

  it("should work with built-in functions", () => {
    const source = "print $ toString 42"
    const parser = new Parser(source)
    const program = parser.parse()
    const generated = generateTypeScript(program.statements)

    // TypeScript生成では toString -> String に変換される
    expect(generated).toContain("print(String(42))")
  })

  it("should work with complex expressions", () => {
    const source = "print $ toString $ multiply 4 6"
    const parser = new Parser(source)
    const program = parser.parse()
    const generated = generateTypeScript(program.statements)

    expect(generated).toContain("print(toString(")
    expect(generated).toContain("multiply")
  })

  it("should handle $ with parentheses correctly", () => {
    const source = "(f $ g) $ x"
    const parser = new Parser(source)
    const program = parser.parse()

    const stmt = program.statements[0] as AST.ExpressionStatement
    const expr = stmt.expression as AST.FunctionApplicationOperator

    expect(expr.kind).toBe("FunctionApplicationOperator")
    // Left side should be parenthesized function application
    const left = expr.left as AST.FunctionApplicationOperator
    expect(left.kind).toBe("FunctionApplicationOperator")
    expect((left.left as AST.Identifier).name).toBe("f")
    expect((left.right as AST.Identifier).name).toBe("g")
    expect((expr.right as AST.Identifier).name).toBe("x")
  })
})