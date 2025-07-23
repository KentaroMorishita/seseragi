import { describe, expect, test } from "bun:test"
import type * as AST from "../src/ast"
import { generateTypeScript } from "../src/codegen"
import { Parser } from "../src/parser"

describe("Array Basic Implementation", () => {
  test("Array literal parsing", () => {
    const source = "let arr = [1, 2, 3]"
    const parser = new Parser(source)
    const parseResult = parser.parse()

    expect(parseResult.statements).toHaveLength(1)
    const stmt = parseResult.statements?.[0] as AST.VariableDeclaration
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
    const parseResult = parser.parse()

    const stmt = parseResult.statements?.[0] as AST.VariableDeclaration
    expect(stmt.initializer.kind).toBe("ArrayAccess")

    const arrayAccess = stmt.initializer as AST.ArrayAccess
    expect((arrayAccess.array as AST.Identifier).name).toBe("arr")
    expect((arrayAccess.index as AST.Literal).value).toBe(0)
  })

  test("Empty array parsing", () => {
    const source = "let empty = []"
    const parser = new Parser(source)
    const parseResult = parser.parse()

    const stmt = parseResult.statements?.[0] as AST.VariableDeclaration
    expect(stmt.initializer.kind).toBe("ArrayLiteral")

    const arrayLiteral = stmt.initializer as AST.ArrayLiteral
    expect(arrayLiteral.elements).toHaveLength(0)
  })

  test("Array code generation with safe access", () => {
    const source = `
      let numbers = [1, 2, 3]
      let first = numbers[0]
    `
    const parser = new Parser(source)
    const parseResult = parser.parse()
    const code = generateTypeScript(parseResult.statements || [], {
      generateComments: false,
    })

    expect(code).toContain("[1, 2, 3]")
    // 安全な配列アクセスはMaybe型を返す
    expect(code).toContain("Just")
    expect(code).toContain("Nothing")
    expect(code).toContain("const numbers")
    expect(code).toContain("const first")
  })

  test("Nested array access parsing", () => {
    const source = "let value = matrix[0][1]"
    const parser = new Parser(source)
    const parseResult = parser.parse()

    const stmt = parseResult.statements?.[0] as AST.VariableDeclaration
    expect(stmt.initializer.kind).toBe("ArrayAccess")

    const outerAccess = stmt.initializer as AST.ArrayAccess
    expect(outerAccess.array.kind).toBe("ArrayAccess")

    const innerAccess = outerAccess.array as AST.ArrayAccess
    expect((innerAccess.array as AST.Identifier).name).toBe("matrix")
    expect((innerAccess.index as AST.Literal).value).toBe(0)
    expect((outerAccess.index as AST.Literal).value).toBe(1)
  })

  test("Array length property parsing", () => {
    const source = "let len = arr.length"
    const parser = new Parser(source)
    const parseResult = parser.parse()

    const stmt = parseResult.statements?.[0] as AST.VariableDeclaration
    expect(stmt.initializer.kind).toBe("RecordAccess")

    const lengthAccess = stmt.initializer as AST.RecordAccess
    expect((lengthAccess.record as AST.Identifier).name).toBe("arr")
    expect(lengthAccess.fieldName).toBe("length")
  })

  test("Array length code generation", () => {
    const source = "let len = arr.length"
    const parser = new Parser(source)
    const parseResult = parser.parse()
    const code = generateTypeScript(parseResult.statements || [], {
      generateComments: false,
    })

    expect(code).toContain("arr.length")
  })
})
