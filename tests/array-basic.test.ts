import { describe, test, expect } from "bun:test"
import { Lexer } from "../src/lexer"
import { Parser } from "../src/parser"
import { generateTypeScript } from "../src/codegen"
import type * as AST from "../src/ast"

describe("Array Basic Implementation", () => {
  test("Array literal parsing", () => {
    const source = "let arr = [1, 2, 3]"
    const parser = new Parser(source)
    const ast = parser.parse()

    expect(ast.statements).toHaveLength(1)
    const stmt = ast.statements[0] as AST.VariableDeclaration
    expect(stmt.kind).toBe("VariableDeclaration")
    expect(stmt.initializer.kind).toBe("ArrayLiteral")

    const arrayLiteral = stmt.initializer as AST.ArrayLiteral
    expect(arrayLiteral.elements).toHaveLength(3)
    expect((arrayLiteral.elements[0] as AST.Literal).value).toBe(1)
    expect((arrayLiteral.elements[1] as AST.Literal).value).toBe(2)
    expect((arrayLiteral.elements[2] as AST.Literal).value).toBe(3)
  })

  test("Array access parsing", () => {
    const source = "let first = arr[0]"
    const parser = new Parser(source)
    const ast = parser.parse()

    const stmt = ast.statements[0] as AST.VariableDeclaration
    expect(stmt.initializer.kind).toBe("ArrayAccess")

    const arrayAccess = stmt.initializer as AST.ArrayAccess
    expect((arrayAccess.array as AST.Identifier).name).toBe("arr")
    expect((arrayAccess.index as AST.Literal).value).toBe(0)
  })

  test("Empty array parsing", () => {
    const source = "let empty = []"
    const parser = new Parser(source)
    const ast = parser.parse()

    const stmt = ast.statements[0] as AST.VariableDeclaration
    expect(stmt.initializer.kind).toBe("ArrayLiteral")

    const arrayLiteral = stmt.initializer as AST.ArrayLiteral
    expect(arrayLiteral.elements).toHaveLength(0)
  })

  test("Array code generation", () => {
    const source = `
      let numbers = [1, 2, 3]
      let first = numbers[0]
    `
    const parser = new Parser(source)
    const ast = parser.parse()
    const code = generateTypeScript(ast.statements, { generateComments: false })

    expect(code).toContain("[1, 2, 3]")
    expect(code).toContain("numbers[0]")
    expect(code).toContain("const numbers")
    expect(code).toContain("const first")
  })

  test("Nested array access parsing", () => {
    const source = "let value = matrix[0][1]"
    const parser = new Parser(source)
    const ast = parser.parse()

    const stmt = ast.statements[0] as AST.VariableDeclaration
    expect(stmt.initializer.kind).toBe("ArrayAccess")

    const outerAccess = stmt.initializer as AST.ArrayAccess
    expect(outerAccess.array.kind).toBe("ArrayAccess")

    const innerAccess = outerAccess.array as AST.ArrayAccess
    expect((innerAccess.array as AST.Identifier).name).toBe("matrix")
    expect((innerAccess.index as AST.Literal).value).toBe(0)
    expect((outerAccess.index as AST.Literal).value).toBe(1)
  })
})
