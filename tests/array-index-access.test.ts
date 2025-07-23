/**
 * Tests for Array and Tuple index access functionality
 */
import { describe, expect, test } from "bun:test"
import type * as AST from "../src/ast"
import { generateTypeScript } from "../src/codegen"
import { Parser } from "../src/parser"

describe("Array Index Access", () => {
  test("should parse array access expression", () => {
    const source = "let value = arr[1]"
    const parser = new Parser(source)
    const parseResult = parser.parse()

    // プログラム内の最初の変数定義のArrayAccessを確認
    const stmt = parseResult.statements?.[0] as AST.VariableDeclaration
    const expr = stmt.initializer as AST.ArrayAccess

    expect(expr.kind).toBe("ArrayAccess")
    expect((expr.array as AST.Identifier).name).toBe("arr")
    expect((expr.index as AST.Literal).value).toBe(1)
  })

  test("should generate correct TypeScript for array access", () => {
    const source = "let value = arr[0]"
    const parser = new Parser(source)
    const parseResult = parser.parse()

    const tsCode = generateTypeScript(parseResult.statements || [])
    // 安全な配列アクセスはMaybe型を返す
    expect(tsCode).toContain(
      "((0) >= 0 && (0) < (arr.tag === 'Tuple' ? arr.elements : arr).length ? { tag: 'Just', value: (arr.tag === 'Tuple' ? arr.elements : arr)[0] } : { tag: 'Nothing' })"
    )
  })
})

describe("Tuple Index Access", () => {
  test("should parse tuple access expression", () => {
    const source = "let coord = point[0]"
    const parser = new Parser(source)
    const parseResult = parser.parse()

    const stmt = parseResult.statements?.[0] as AST.VariableDeclaration
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
    const parseResult = parser.parse()
    const tsCode = generateTypeScript(parseResult.statements || [])

    // 安全なタプルアクセスもMaybe型を返す
    expect(tsCode).toContain(
      "((0) >= 0 && (0) < (point.tag === 'Tuple' ? point.elements : point).length ? { tag: 'Just', value: (point.tag === 'Tuple' ? point.elements : point)[0] } : { tag: 'Nothing' })"
    )
  })

  test("should handle nested array access", () => {
    const source = "let cell = matrix[0][1]"
    const parser = new Parser(source)
    const parseResult = parser.parse()
    const tsCode = generateTypeScript(parseResult.statements || [])

    // ネストした配列アクセスもMaybe型を返すようになる
    expect(tsCode).toContain("matrix")
    expect(tsCode).toContain("Just")
    expect(tsCode).toContain("Nothing")
  })
})

describe("Array Length Property", () => {
  test("should parse array length access", () => {
    const source = "let len = arr.length"
    const parser = new Parser(source)
    const parseResult = parser.parse()

    const stmt = parseResult.statements?.[0] as AST.VariableDeclaration
    const expr = stmt.initializer as AST.RecordAccess

    expect(expr.kind).toBe("RecordAccess")
    expect((expr.record as AST.Identifier).name).toBe("arr")
    expect(expr.fieldName).toBe("length")
  })

  test("should generate correct TypeScript for length access", () => {
    const source = "let len = arr.length"
    const parser = new Parser(source)
    const parseResult = parser.parse()

    const tsCode = generateTypeScript(parseResult.statements || [])
    expect(tsCode).toContain("arr.length")
  })
})

describe("Safe Array Access", () => {
  test("should return Maybe type for array access", () => {
    const source = `
      let arr = [1, 2, 3]
      let safe = arr[0]
    `
    const parser = new Parser(source)
    const parseResult = parser.parse()
    const tsCode = generateTypeScript(parseResult.statements || [])

    // Maybe型のランタイムが含まれることを確認
    expect(tsCode).toContain("Just")
    expect(tsCode).toContain("Nothing")
  })
})

describe("Error Cases", () => {
  test("should handle malformed bracket syntax", () => {
    const parser = new Parser("arr[")
    const result = parser.parse()
    expect(result.errors.length).toBeGreaterThan(0)
  })

  test("should handle missing index", () => {
    const parser = new Parser("arr[]")
    const result = parser.parse()
    expect(result.errors.length).toBeGreaterThan(0)
  })
})
