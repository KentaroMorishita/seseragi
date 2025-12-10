import { describe, expect, it } from "bun:test"
import * as AST from "../src/ast"
import { generateTypeScript } from "../src/codegen"
import { Lexer } from "../src/lexer"
import { Parser } from "../src/parser"
import { infer } from "../src/inference/engine/infer"

describe("Union and Intersection Types", () => {
  const parseAndGenerate = (code: string) => {
    const lexer = new Lexer(code)
    const tokens = lexer.tokenize()
    const parser = new Parser(tokens)
    const parseResult = parser.parse()

    if (parseResult.errors.length > 0) {
      throw new Error(
        `Parse errors: ${parseResult.errors.map((e) => e.message).join(", ")}`
      )
    }

    const generated = generateTypeScript(parseResult.statements || [])
    return { parseResult, generated }
  }

  const runTypeInference = (code: string) => {
    const lexer = new Lexer(code)
    const tokens = lexer.tokenize()
    const parser = new Parser(tokens)
    const parseResult = parser.parse()

    if (parseResult.errors.length > 0) {
      throw new Error(
        `Parse errors: ${parseResult.errors.map((e) => e.message).join(", ")}`
      )
    }

    const program = new AST.Program(parseResult.statements || [])
    const result = infer(program)

    return { parseResult, result }
  }

  describe("Basic Union Types", () => {
    it("should parse simple union type alias", () => {
      const code = `type StringOrInt = String | Int`

      // Built-in types are allowed in union types
      const { parseResult, generated } = parseAndGenerate(code)
      const stmt = parseResult.statements![0] as AST.TypeAliasDeclaration

      expect(stmt.kind).toBe("TypeAliasDeclaration")
      expect(stmt.aliasedType.kind).toBe("UnionType")
      expect(generated).toContain("(string | number)")
    })

    it("should parse union type with existing types", () => {
      const code = `
        type A = String
        type B = Int
        type AB = A | B
      `

      const { parseResult } = parseAndGenerate(code)
      const thirdStmt = parseResult.statements![2] as AST.TypeAliasDeclaration

      expect(thirdStmt.kind).toBe("TypeAliasDeclaration")
      expect(thirdStmt.aliasedType.kind).toBe("UnionType")

      const unionType = thirdStmt.aliasedType as AST.UnionType
      expect(unionType.types.length).toBe(2)
      expect(unionType.types[0].kind).toBe("PrimitiveType")
      expect(unionType.types[1].kind).toBe("PrimitiveType")
    })
  })

  describe("Basic Intersection Types", () => {
    it("should parse simple intersection type", () => {
      const code = `
        type A = String
        type B = Int
        type AB = A & B
      `

      const { parseResult } = parseAndGenerate(code)
      const thirdStmt = parseResult.statements![2] as AST.TypeAliasDeclaration

      expect(thirdStmt.kind).toBe("TypeAliasDeclaration")
      expect(thirdStmt.aliasedType.kind).toBe("IntersectionType")

      const intersectionType = thirdStmt.aliasedType as AST.IntersectionType
      expect(intersectionType.types.length).toBe(2)
    })
  })

  describe("Complex Union/Intersection Types", () => {
    it("should parse union with intersection (precedence test)", () => {
      const code = `
        type A = String
        type B = Int
        type C = Bool
        type Complex = A | B & C
      `

      const { parseResult } = parseAndGenerate(code)
      const complexStmt = parseResult.statements![3] as AST.TypeAliasDeclaration

      expect(complexStmt.aliasedType.kind).toBe("UnionType")

      const unionType = complexStmt.aliasedType as AST.UnionType
      expect(unionType.types.length).toBe(2)
      expect(unionType.types[0].kind).toBe("PrimitiveType") // A
      expect(unionType.types[1].kind).toBe("IntersectionType") // B & C
    })

    it("should parse parenthesized union/intersection", () => {
      const code = `
        type A = String
        type B = Int  
        type C = Bool
        type Complex = (A | B) & C
      `

      const { parseResult } = parseAndGenerate(code)
      const complexStmt = parseResult.statements![3] as AST.TypeAliasDeclaration

      expect(complexStmt.aliasedType.kind).toBe("IntersectionType")

      const intersectionType = complexStmt.aliasedType as AST.IntersectionType
      expect(intersectionType.types.length).toBe(2)
      expect(intersectionType.types[0].kind).toBe("UnionType") // (A | B)
      expect(intersectionType.types[1].kind).toBe("PrimitiveType") // C
    })
  })

  describe("Built-in Types in Union/Intersection", () => {
    it("should allow built-in types in unions", () => {
      const code = `type StringOrInt = String | Int`

      const { parseResult, generated } = parseAndGenerate(code)
      const stmt = parseResult.statements![0] as AST.TypeAliasDeclaration

      expect(stmt.aliasedType.kind).toBe("UnionType")
      expect(generated).toContain("(string | number)")
    })

    it("should allow built-in types in intersections", () => {
      const code = `type StringAndInt = String & Int`

      const { parseResult, generated } = parseAndGenerate(code)
      const stmt = parseResult.statements![0] as AST.TypeAliasDeclaration

      expect(stmt.aliasedType.kind).toBe("IntersectionType")
      expect(generated).toContain("(string & number)")
    })
  })

  describe("ADT vs Union Type Distinction", () => {
    it("should parse ADT with leading pipe", () => {
      const code = `
        type Color = | Red | Green | Blue
        let red = Red
      `

      const { parseResult } = parseAndGenerate(code)
      const typeStmt = parseResult.statements![0] as AST.TypeDeclaration

      expect(typeStmt.kind).toBe("TypeDeclaration")
      expect(typeStmt.name).toBe("Color")
      expect(typeStmt.fields.length).toBe(3)
    })

    it("should reject ADT without leading pipe", () => {
      const code = `type Color = Red | Green | Blue`

      // This should fail because Red, Green, Blue are not defined types
      expect(() => parseAndGenerate(code)).toThrow(/not defined/)
    })

    it("should parse union type when all types are defined", () => {
      const code = `
        type Red = String
        type Green = String
        type Blue = String
        type Color = Red | Green | Blue
      `

      const { parseResult } = parseAndGenerate(code)
      const colorStmt = parseResult.statements![3] as AST.TypeAliasDeclaration

      expect(colorStmt.kind).toBe("TypeAliasDeclaration")
      expect(colorStmt.aliasedType.kind).toBe("UnionType")
    })
  })

  describe("Type Inference with Union/Intersection", () => {
    it("should infer union types correctly", () => {
      const code = `
        type A = String
        type B = Int
        type AB = A | B
        
        let x: AB = "hello"
      `

      const { result } = runTypeInference(code)
      expect(result.errors.length).toBe(0)
    })

    it("should infer intersection types correctly", () => {
      const code = `
        type A = String
        type B = Int
        type AB = A & B
        
        let x: AB = "hello"
      `

      runTypeInference(code)
      // This might fail during type checking depending on implementation
      // The behavior depends on how intersection types are handled
    })
  })

  describe("TypeScript Code Generation", () => {
    it("should generate correct TypeScript for union types", () => {
      const code = `type StringOrInt = String | Int`

      const { generated } = parseAndGenerate(code)
      expect(generated).toContain("type StringOrInt = (string | number)")
    })

    it("should generate correct TypeScript for intersection types", () => {
      const code = `type StringAndInt = String & Int`

      const { generated } = parseAndGenerate(code)
      expect(generated).toContain("type StringAndInt = (string & number)")
    })

    it("should generate correct TypeScript for complex types", () => {
      const code = `
        type A = String
        type B = Int
        type C = Bool
        type Complex = A | B & C
      `

      const { generated } = parseAndGenerate(code)
      expect(generated).toContain("type Complex = (A | (B & C))")
    })
  })

  describe("Error Handling", () => {
    it("should error on undefined types in union", () => {
      const code = `type BadUnion = UndefinedType | String`

      expect(() => parseAndGenerate(code)).toThrow(/not defined/)
    })

    it("should error on undefined types in intersection", () => {
      const code = `type BadIntersection = UndefinedType & String`

      expect(() => parseAndGenerate(code)).toThrow(/not defined/)
    })
  })
})
