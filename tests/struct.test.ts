import { describe, test, expect } from "bun:test"
import { Lexer } from "../src/lexer"
import { Parser } from "../src/parser"
import { generateTypeScript } from "../src/codegen"
import { TypeInferenceSystem } from "../src/type-inference"
import * as AST from "../src/ast"

describe("Struct Tests", () => {
  test("should parse struct declaration", () => {
    const source = `
      struct Person {
        name: String,
        age: Int
      }
    `
    const parser = new Parser(source)
    const result = parser.parse()
    
    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)
    
    const structDecl = result.statements![0] as AST.StructDeclaration
    expect(structDecl.kind).toBe("StructDeclaration")
    expect(structDecl.name).toBe("Person")
    expect(structDecl.fields).toHaveLength(2)
    
    expect(structDecl.fields[0].name).toBe("name")
    expect(structDecl.fields[0].type.kind).toBe("PrimitiveType")
    expect((structDecl.fields[0].type as AST.PrimitiveType).name).toBe("String")
    
    expect(structDecl.fields[1].name).toBe("age")
    expect(structDecl.fields[1].type.kind).toBe("PrimitiveType")
    expect((structDecl.fields[1].type as AST.PrimitiveType).name).toBe("Int")
  })

  test("should parse struct instantiation", () => {
    const source = `Person { name: "Alice", age: 30 }`
    const parser = new Parser(source)
    const expr = parser["primaryExpression"]()
    
    expect(expr.kind).toBe("StructExpression")
    const structExpr = expr as AST.StructExpression
    expect(structExpr.structName).toBe("Person")
    expect(structExpr.fields).toHaveLength(2)
    
    expect(structExpr.fields[0].name).toBe("name")
    expect(structExpr.fields[0].value.kind).toBe("Literal")
    
    expect(structExpr.fields[1].name).toBe("age")
    expect(structExpr.fields[1].value.kind).toBe("Literal")
  })

  test("should generate TypeScript interface for struct", () => {
    const source = `
      struct Person {
        name: String,
        age: Int
      }
    `
    const parser = new Parser(source)
    const result = parser.parse()
    const typescript = generateTypeScript(result.statements!)
    
    expect(typescript).toContain("interface Person")
    expect(typescript).toContain("name: string;")
    expect(typescript).toContain("age: number;")
  })

  test("should generate TypeScript for struct instantiation", () => {
    const source = `let person = Person { name: "Alice", age: 30 }`
    const parser = new Parser(source)
    const result = parser.parse()
    const typescript = generateTypeScript(result.statements!)
    
    expect(typescript).toContain('({ name: "Alice", age: 30 } as Person)')
  })

  test("should parse struct with multiple field types", () => {
    const source = `
      struct User {
        id: Int,
        name: String,
        email: String,
        isActive: Bool,
        score: Float
      }
    `
    const parser = new Parser(source)
    const result = parser.parse()
    
    expect(result.errors).toHaveLength(0)
    const structDecl = result.statements![0] as AST.StructDeclaration
    expect(structDecl.fields).toHaveLength(5)
    
    const typescript = generateTypeScript(result.statements!)
    expect(typescript).toContain("interface User")
    expect(typescript).toContain("id: number;")
    expect(typescript).toContain("name: string;")
    expect(typescript).toContain("email: string;")
    expect(typescript).toContain("isActive: boolean;")
    expect(typescript).toContain("score: number;")
  })

  test("should parse struct with generic types", () => {
    const source = `
      struct Node {
        value: Int,
        next: Maybe<Node>
      }
    `
    const parser = new Parser(source)
    const result = parser.parse()
    
    expect(result.errors).toHaveLength(0)
    const structDecl = result.statements![0] as AST.StructDeclaration
    expect(structDecl.fields).toHaveLength(2)
    
    expect(structDecl.fields[1].name).toBe("next")
    expect(structDecl.fields[1].type.kind).toBe("GenericType")
    const nextType = structDecl.fields[1].type as AST.GenericType
    expect(nextType.name).toBe("Maybe")
    expect(nextType.typeArguments).toHaveLength(1)
  })

  test("should parse empty struct", () => {
    const source = `struct Empty {}`
    const parser = new Parser(source)
    const result = parser.parse()
    
    expect(result.errors).toHaveLength(0)
    const structDecl = result.statements![0] as AST.StructDeclaration
    expect(structDecl.fields).toHaveLength(0)
    
    const typescript = generateTypeScript(result.statements!)
    expect(typescript).toContain("interface Empty")
    expect(typescript).toContain("{")
    expect(typescript).toContain("}")
  })

  test("should handle struct field access", () => {
    const source = `
      struct Person {
        name: String,
        age: Int
      }
      
      let person = Person { name: "Bob", age: 25 }
      let name = person.name
    `
    const parser = new Parser(source)
    const result = parser.parse()
    
    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(3)
    
    const typescript = generateTypeScript(result.statements!)
    expect(typescript).toContain("interface Person")
    expect(typescript).toContain('({ name: "Bob", age: 25 } as Person)')
    expect(typescript).toContain("person.name")
  })

  test("should parse struct with record field", () => {
    const source = `
      struct Employee {
        info: { name: String, department: String },
        salary: Int
      }
    `
    const parser = new Parser(source)
    const result = parser.parse()
    
    expect(result.errors).toHaveLength(0)
    const structDecl = result.statements![0] as AST.StructDeclaration
    expect(structDecl.fields).toHaveLength(2)
    
    expect(structDecl.fields[0].type.kind).toBe("RecordType")
    const infoType = structDecl.fields[0].type as AST.RecordType
    expect(infoType.fields).toHaveLength(2)
  })

  test("should differentiate struct from type alias", () => {
    const source1 = `type PersonType = { name: String, age: Int }`
    const source2 = `struct PersonStruct { name: String, age: Int }`
    
    const parser1 = new Parser(source1)
    const result1 = parser1.parse()
    const stmt1 = result1.statements![0]
    expect(stmt1.kind).toBe("TypeAliasDeclaration")
    
    const parser2 = new Parser(source2)
    const result2 = parser2.parse()
    const stmt2 = result2.statements![0]
    expect(stmt2.kind).toBe("StructDeclaration")
    
    const ts1 = generateTypeScript(result1.statements!)
    const ts2 = generateTypeScript(result2.statements!)
    
    expect(ts1).toContain("type PersonType =")
    expect(ts2).toContain("interface PersonStruct")
  })
})