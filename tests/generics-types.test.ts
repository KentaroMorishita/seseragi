import { describe, expect, test } from "bun:test"
import { Parser } from "../src/parser"
import { TypeInferenceSystem } from "../src/type-inference"
import { generateTypeScript } from "../src/codegen"
import * as AST from "../src/ast"

describe("Generic Type Declarations", () => {
  test("should parse generic type alias with simple types", () => {
    const code = `type Box<T> = T`

    const parser = new Parser(code)
    const parseResult = parser.parse()

    expect(parseResult.statements).toHaveLength(1)
    const typeAlias = parseResult.statements![0] as AST.TypeAliasDeclaration

    expect(typeAlias.kind).toBe("TypeAliasDeclaration")
    expect(typeAlias.name).toBe("Box")
    expect(typeAlias.typeParameters).toHaveLength(1)
    expect(typeAlias.typeParameters![0].name).toBe("T")
  })

  test("should parse generic type with multiple parameters", () => {
    const code = `type Pair<A, B> = (A, B)`

    const parser = new Parser(code)
    const parseResult = parser.parse()

    const typeAlias = parseResult.statements![0] as AST.TypeAliasDeclaration

    expect(typeAlias.typeParameters).toHaveLength(2)
    expect(typeAlias.typeParameters![0].name).toBe("A")
    expect(typeAlias.typeParameters![1].name).toBe("B")
  })

  test("should parse generic record type", () => {
    const code = `type Container<T> = { value: T, metadata: String }`

    const parser = new Parser(code)
    const parseResult = parser.parse()

    const typeAlias = parseResult.statements![0] as AST.TypeAliasDeclaration

    expect(typeAlias.typeParameters).toHaveLength(1)
    expect(typeAlias.typeParameters![0].name).toBe("T")
  })

  test("should generate correct TypeScript for generic types", () => {
    const code = `type Box<T> = T`

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const generated = generateTypeScript(parseResult.statements || [])

    // TypeScriptジェネリクス構文が含まれているかチェック
    expect(generated).toContain("<T>")
    expect(generated).toContain("Box")
  })

  test("should handle function with generic type return", () => {
    const code = `
type Box<T> = T
fn makeBox<T> value: T -> Box<T> = value
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()

    expect(parseResult.statements).toHaveLength(2)

    const typeAlias = parseResult.statements![0] as AST.TypeAliasDeclaration
    const funcDecl = parseResult.statements![1] as AST.FunctionDeclaration

    expect(typeAlias.typeParameters).toHaveLength(1)
    expect(funcDecl.typeParameters).toHaveLength(1)
  })

  test("should parse nested generic types", () => {
    const code = `type Array<T> = List<T>`

    const parser = new Parser(code)
    const parseResult = parser.parse()

    const typeAlias = parseResult.statements![0] as AST.TypeAliasDeclaration

    expect(typeAlias.typeParameters).toHaveLength(1)
    expect(typeAlias.typeParameters![0].name).toBe("T")
  })

  test("should combine generic functions and types", () => {
    const code = `
type Box<T> = T
fn makeBox<T> value: T -> T = value
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()

    expect(parseResult.statements).toHaveLength(2)
    
    const typeAlias = parseResult.statements![0] as AST.TypeAliasDeclaration
    const funcDecl = parseResult.statements![1] as AST.FunctionDeclaration

    // 基本的なパースが成功していることを確認
    expect(typeAlias.kind).toBe("TypeAliasDeclaration")
    expect(typeAlias.typeParameters).toHaveLength(1)
    expect(funcDecl.kind).toBe("FunctionDeclaration")
    expect(funcDecl.typeParameters).toHaveLength(1)
  })
})
