import { describe, expect, test } from "bun:test"
import * as AST from "../src/ast"
import { generateTypeScript } from "../src/codegen"
import { Parser } from "../src/parser"
import { infer } from "../src/inference/engine/infer"

describe("Shorthand Property Notation Tests", () => {
  test("should parse shorthand property in record expression", () => {
    const code = `
      let x = 10
      let y = 20
      let point = { x, y }
    `
    const parser = new Parser(code)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)

    const recordStmt = result.statements?.[2] as AST.VariableDeclaration
    expect(recordStmt.kind).toBe("VariableDeclaration")

    const recordExpr = recordStmt.initializer as AST.RecordExpression
    expect(recordExpr.kind).toBe("RecordExpression")
    expect(recordExpr.fields).toHaveLength(2)

    const field1 = recordExpr.fields[0] as AST.RecordShorthandField
    const field2 = recordExpr.fields[1] as AST.RecordShorthandField

    expect(field1.kind).toBe("RecordShorthandField")
    expect(field1.name).toBe("x")
    expect(field2.kind).toBe("RecordShorthandField")
    expect(field2.name).toBe("y")
  })

  test("should parse shorthand property in struct expression", () => {
    const code = `
      struct Point { x: Int, y: Int }
      let x = 10
      let y = 20
      let point = Point { x, y }
    `
    const parser = new Parser(code)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)

    const varStmt = result.statements?.[3] as AST.VariableDeclaration
    expect(varStmt.kind).toBe("VariableDeclaration")

    const structExpr = varStmt.initializer as AST.StructExpression
    expect(structExpr.kind).toBe("StructExpression")
    expect(structExpr.structName).toBe("Point")
    expect(structExpr.fields).toHaveLength(2)

    const field1 = structExpr.fields[0] as AST.RecordShorthandField
    const field2 = structExpr.fields[1] as AST.RecordShorthandField

    expect(field1.kind).toBe("RecordShorthandField")
    expect(field1.name).toBe("x")
    expect(field2.kind).toBe("RecordShorthandField")
    expect(field2.name).toBe("y")
  })

  test("should parse mixed shorthand and explicit properties", () => {
    const code = `
      struct User { name: String, age: Int, active: Bool }
      let name = "Alice"
      let user = User { name, age: 25, active: True }
    `
    const parser = new Parser(code)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)

    const varStmt = result.statements?.[2] as AST.VariableDeclaration
    const structExpr = varStmt.initializer as AST.StructExpression

    expect(structExpr.fields).toHaveLength(3)

    const field1 = structExpr.fields[0] as AST.RecordShorthandField
    const field2 = structExpr.fields[1] as AST.RecordInitField
    const field3 = structExpr.fields[2] as AST.RecordInitField

    expect(field1.kind).toBe("RecordShorthandField")
    expect(field1.name).toBe("name")

    expect(field2.kind).toBe("RecordInitField")
    expect(field2.name).toBe("age")

    expect(field3.kind).toBe("RecordInitField")
    expect(field3.name).toBe("active")
  })

  test("should perform type inference on shorthand properties", () => {
    const code = `
      struct Point { x: Int, y: Int }
      let x = 10
      let y = 20
      let point = Point { x, y }
    `
    const parser = new Parser(code)
    const parseResult = parser.parse()

    expect(parseResult.errors).toHaveLength(0)

    const program = new AST.Program(parseResult.statements!)
    const inferenceResult = infer(program)

    expect(inferenceResult.errors).toHaveLength(0)
  })

  test("should generate correct TypeScript for shorthand properties in records", () => {
    const code = `
      let x = 10
      let y = 20
      let point = { x, y }
    `
    const parser = new Parser(code)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements!)
    const inferenceResult = infer(program)

    const generated = generateTypeScript(parseResult.statements!, {
      typeInferenceResult: inferenceResult,
    })

    expect(generated).toContain("{ x, y }")
  })

  test("should generate correct TypeScript for shorthand properties in structs", () => {
    const code = `
      struct Point { x: Int, y: Int }
      let x = 10
      let y = 20
      let point = Point { x, y }
    `
    const parser = new Parser(code)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements!)
    const inferenceResult = infer(program)

    const generated = generateTypeScript(parseResult.statements!, {
      typeInferenceResult: inferenceResult,
    })

    // Should generate code that uses the shorthand property values
    expect(generated).toContain("x: x")
    expect(generated).toContain("y: y")
  })

  test("should handle shorthand properties with spread syntax", () => {
    const code = `
      struct Point { x: Int, y: Int }
      let existing = Point { x: 5, y: 8 }
      let y = 20
      let point = Point { ...existing, y }
    `
    const parser = new Parser(code)
    const parseResult = parser.parse()

    expect(parseResult.errors).toHaveLength(0)

    const varStmt = parseResult.statements?.[3] as AST.VariableDeclaration
    const structExpr = varStmt.initializer as AST.StructExpression

    expect(structExpr.fields).toHaveLength(2)

    const spreadField = structExpr.fields[0] as AST.RecordSpreadField
    const shorthandField = structExpr.fields[1] as AST.RecordShorthandField

    expect(spreadField.kind).toBe("RecordSpreadField")
    expect(shorthandField.kind).toBe("RecordShorthandField")
    expect(shorthandField.name).toBe("y")
  })

  test("should detect undefined variable in shorthand property", () => {
    const code = `
      let point = { x, y }
    `
    const parser = new Parser(code)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements!)
    const inferenceResult = infer(program)

    // Should have type errors for undefined variables
    expect(inferenceResult.errors.length).toBeGreaterThan(0)
    expect(
      inferenceResult.errors.some((e) =>
        e.message.includes("Undefined variable 'x'")
      )
    ).toBe(true)
    expect(
      inferenceResult.errors.some((e) =>
        e.message.includes("Undefined variable 'y'")
      )
    ).toBe(true)
  })
})
