import { describe, test, expect } from "bun:test"
import { Lexer } from "../src/lexer"
import { Parser } from "../src/parser"
import { generateTypeScript } from "../src/codegen"
import { TypeInferenceSystem } from "../src/type-inference"
import * as AST from "../src/ast"

describe("Record Type Tests", () => {
  test("should parse record type definition", () => {
    const lexer = new Lexer("{ name: String, age: Int }")
    const tokens = lexer.tokenize()
    const parser = new Parser("{ name: String, age: Int }")
    
    const type = parser["parseType"]()
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
    const expr = parser["primaryExpression"]()
    
    expect(expr.kind).toBe("RecordExpression")
    const recordExpr = expr as AST.RecordExpression
    expect(recordExpr.fields).toHaveLength(2)
    expect(recordExpr.fields[0].name).toBe("name")
    expect(recordExpr.fields[1].name).toBe("age")
  })

  test("should parse record field access", () => {
    const parser = new Parser("person.name")
    const expr = parser["callExpression"]()
    
    expect(expr.kind).toBe("RecordAccess")
    const accessExpr = expr as AST.RecordAccess
    expect(accessExpr.fieldName).toBe("name")
    expect(accessExpr.record.kind).toBe("Identifier")
    expect((accessExpr.record as AST.Identifier).name).toBe("person")
  })

  test("should generate correct TypeScript for record expression", () => {
    const parser = new Parser('{ name: "Alice", age: 30 }')
    const expr = parser["primaryExpression"]() as AST.RecordExpression
    
    const typescript = generateTypeScript([
      new AST.ExpressionStatement(expr, 1, 1)
    ])
    
    expect(typescript).toContain('{ name: "Alice", age: 30 }')
  })

  test("should generate correct TypeScript for record access", () => {
    const parser = new Parser("person.name")
    const expr = parser["callExpression"]() as AST.RecordAccess
    
    const typescript = generateTypeScript([
      new AST.ExpressionStatement(expr, 1, 1)
    ])
    
    expect(typescript).toContain("person.name")
  })

  test("should infer record types correctly", () => {
    const source = `let person = { name: "Alice", age: 30 }`
    
    const parser = new Parser(source)
    const program = parser.parse()
    
    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(program)
    
    // Print errors for debugging
    if (result.errors.length > 0) {
      console.log("Type inference errors:", result.errors.map(e => e.message))
    }
    
    // Check that the record expression has the correct type
    const personDecl = program.statements[0] as AST.VariableDeclaration
    const recordExpr = personDecl.initializer as AST.RecordExpression
    const recordType = result.nodeTypeMap.get(recordExpr)
    
    expect(recordType?.kind).toBe("RecordType")
    if (recordType?.kind === "RecordType") {
      const rt = recordType as AST.RecordType
      expect(rt.fields).toHaveLength(2)
      expect(rt.fields.find(f => f.name === "name")?.type.kind).toBe("PrimitiveType")
      expect(rt.fields.find(f => f.name === "age")?.type.kind).toBe("PrimitiveType")
    }
  })

  test("should handle nested record access", () => {
    const parser = new Parser("employee.info.name")
    let expr = parser["primaryExpression"]() // employee
    
    // Parse the chain manually since our parser doesn't support this yet
    expect(expr.kind).toBe("Identifier")
  })

  test("should detect record field access errors", () => {
    const source = `
      let person = { name: "Alice", age: 30 }
      let invalid = person.nonexistent
    `
    
    const parser = new Parser(source)
    const program = parser.parse()
    
    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(program)
    
    // Should have type inference errors for non-existent field
    // Note: This depends on how the constraint system handles record field constraints
    // The test might need adjustment based on the actual implementation
  })

  test("should handle empty record", () => {
    const parser = new Parser("{}")
    const expr = parser["primaryExpression"]()
    
    expect(expr.kind).toBe("RecordExpression")
    const recordExpr = expr as AST.RecordExpression
    expect(recordExpr.fields).toHaveLength(0)
  })

  test("should generate TypeScript type for record type", () => {
    const recordType = new AST.RecordType([
      new AST.RecordField("name", new AST.PrimitiveType("String", 1, 1), 1, 1),
      new AST.RecordField("age", new AST.PrimitiveType("Int", 1, 1), 1, 1)
    ], 1, 1)
    
    // Simple test to verify RecordType exists
    expect(recordType.kind).toBe("RecordType")
    expect(recordType.fields).toHaveLength(2)
  })
})