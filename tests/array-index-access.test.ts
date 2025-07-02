/**
 * Tests for Array and Tuple index access functionality
 */
import { expect, test, describe } from "bun:test"
import { Parser } from "../src/parser"
import { generateTypeScript } from "../src/codegen"
import * as AST from "../src/ast"

describe("Array Index Access", () => {
  test("should parse array access expression", () => {
    const source = "let value = arr[1]"
    const parser = new Parser(source)
    const ast = parser.parse()
    
    // プログラム内の最初の変数定義のArrayAccessを確認
    const stmt = ast.statements[0] as AST.VariableDeclaration
    const expr = stmt.initializer as AST.ArrayAccess
    
    expect(expr.kind).toBe("ArrayAccess")
    expect((expr.array as AST.Identifier).name).toBe("arr")
    expect((expr.index as AST.Literal).value).toBe(1)
  })

  test("should generate correct TypeScript for array access", () => {
    const source = "let value = arr[0]"
    const parser = new Parser(source)
    const ast = parser.parse()
    
    const tsCode = generateTypeScript(ast)
    expect(tsCode).toContain("(arr.tag === 'Tuple' ? arr.elements : arr)[0]")
  })
})

describe("Tuple Index Access", () => {
  test("should parse tuple access expression", () => {
    const source = "let coord = point[0]"
    const parser = new Parser(source)
    const ast = parser.parse()
    
    const stmt = ast.statements[0] as AST.VariableDeclaration
    const expr = stmt.initializer as AST.ArrayAccess
    
    expect(expr.kind).toBe("ArrayAccess")
    expect((expr.array as AST.Identifier).name).toBe("point")
    expect((expr.index as AST.Literal).value).toBe(0)
  })

  test("should generate correct TypeScript for tuple access", () => {
    const source = `
      let point = (10, 20)
      let x = point[0]
    `
    const parser = new Parser(source)
    const ast = parser.parse()
    const tsCode = generateTypeScript(ast)
    
    // Tuple型の場合は.elementsから取り出すことを確認
    expect(tsCode).toContain("(point.tag === 'Tuple' ? point.elements : point)[0]")
  })

  test("should handle nested array access", () => {
    const source = "let cell = matrix[0][1]"
    const parser = new Parser(source)
    const ast = parser.parse()
    const tsCode = generateTypeScript(ast)
    
    // ネストした配列アクセスが正しく生成されることを確認
    expect(tsCode).toContain("matrix")
    expect(tsCode).toContain("[0]")
    expect(tsCode).toContain("[1]")
  })
})

describe("Error Cases", () => {
  test("should handle malformed bracket syntax", () => {
    expect(() => {
      const parser = new Parser("arr[")
      parser.parse()
    }).toThrow()
  })

  test("should handle missing index", () => {
    expect(() => {
      const parser = new Parser("arr[]")
      parser.parse()
    }).toThrow()
  })
})