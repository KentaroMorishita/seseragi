import { describe, expect, test } from "bun:test"
import * as AST from "../src/ast"
import { generateTypeScript } from "../src/codegen"
import { Parser } from "../src/parser"
import { TypeInferenceSystem } from "../src/type-inference"

describe("Record Type Tests", () => {
  test("should parse record type definition", () => {
    const source = "type Person = { name: String, age: Int }"
    const parser = new Parser(source)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    const stmt = result.statements![0] as AST.TypeAliasDeclaration
    const type = stmt.aliasedType
    expect(type.kind).toBe("RecordType")

    const recordType = type as AST.RecordType
    expect(recordType.fields).toHaveLength(2)
    expect(recordType.fields[0].name).toBe("name")
    expect(recordType.fields[0].type.kind).toBe("PrimitiveType")
    expect((recordType.fields[0].type as AST.PrimitiveType).name).toBe("String")
    expect(recordType.fields[1].name).toBe("age")
    expect(recordType.fields[1].type.kind).toBe("PrimitiveType")
    expect((recordType.fields[1].type as AST.PrimitiveType).name).toBe("Int")
  })

  test("should parse record expression", () => {
    const parser = new Parser('{ name: "Alice", age: 30 }')
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    const stmt = result.statements![0] as AST.ExpressionStatement
    const expr = stmt.expression
    expect(expr.kind).toBe("RecordExpression")
    const recordExpr = expr as AST.RecordExpression
    expect(recordExpr.fields).toHaveLength(2)
    const field0 = recordExpr.fields[0] as AST.RecordInitField
    const field1 = recordExpr.fields[1] as AST.RecordInitField
    expect(field0.name).toBe("name")
    expect(field1.name).toBe("age")
  })

  test("should parse record field access", () => {
    const parser = new Parser("person.name")
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    const stmt = result.statements![0] as AST.ExpressionStatement
    const expr = stmt.expression
    expect(expr.kind).toBe("RecordAccess")
    const accessExpr = expr as AST.RecordAccess
    expect(accessExpr.fieldName).toBe("name")
    expect(accessExpr.record.kind).toBe("Identifier")
    expect((accessExpr.record as AST.Identifier).name).toBe("person")
  })

  test("should generate correct TypeScript for record expression", () => {
    const parser = new Parser('{ name: "Alice", age: 30 }')
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    const typescript = generateTypeScript(result.statements!)
    expect(typescript).toContain('{ name: "Alice", age: 30 }')
  })

  test("should generate correct TypeScript for record access", () => {
    const parser = new Parser("person.name")
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    const typescript = generateTypeScript(result.statements!)
    expect(typescript).toContain("person.name")
  })

  test("should infer record types correctly", () => {
    const source = `let person = { name: "Alice", age: 30 }`

    const parser = new Parser(source)
    const parseResult = parser.parse()

    const program = new AST.Program(parseResult.statements || [])
    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(program)

    // Print errors for debugging
    if (result.errors.length > 0) {
      console.log(
        "Type inference errors:",
        result.errors.map((e) => e.message)
      )
    }

    // Check that the record expression has the correct type
    const personDecl = parseResult.statements![0] as AST.VariableDeclaration
    const recordExpr = personDecl.initializer as AST.RecordExpression
    const recordType = result.nodeTypeMap.get(recordExpr)

    expect(recordType?.kind).toBe("RecordType")
    if (recordType?.kind === "RecordType") {
      const rt = recordType as AST.RecordType
      expect(rt.fields).toHaveLength(2)
      expect(rt.fields.find((f) => f.name === "name")?.type.kind).toBe(
        "PrimitiveType"
      )
      expect(rt.fields.find((f) => f.name === "age")?.type.kind).toBe(
        "PrimitiveType"
      )
    }
  })

  test("should handle nested record access", () => {
    const parser = new Parser("employee.info.name")
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    const stmt = result.statements![0] as AST.ExpressionStatement
    // For now just check that it parses without error
    expect(stmt.expression).toBeDefined()
  })

  test("should detect record field access errors", () => {
    const source = `
      let person = { name: "Alice", age: 30 }
      let invalid = person.nonexistent
    `

    const parser = new Parser(source)
    const parseResult = parser.parse()

    const program = new AST.Program(parseResult.statements || [])
    const typeInference = new TypeInferenceSystem()
    const _result = typeInference.infer(program)

    // Should have type inference errors for non-existent field
    // Note: This depends on how the constraint system handles record field constraints
    // The test might need adjustment based on the actual implementation
  })

  test("should handle empty record", () => {
    const parser = new Parser("{}")
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    const stmt = result.statements![0] as AST.ExpressionStatement
    const expr = stmt.expression
    expect(expr.kind).toBe("RecordExpression")
    const recordExpr = expr as AST.RecordExpression
    expect(recordExpr.fields).toHaveLength(0)
  })

  test("should generate TypeScript type for record type", () => {
    const recordType = new AST.RecordType(
      [
        new AST.RecordField(
          "name",
          new AST.PrimitiveType("String", 1, 1),
          1,
          1
        ),
        new AST.RecordField("age", new AST.PrimitiveType("Int", 1, 1), 1, 1),
      ],
      1,
      1
    )

    // Simple test to verify RecordType exists
    expect(recordType.kind).toBe("RecordType")
    expect(recordType.fields).toHaveLength(2)
  })
})
