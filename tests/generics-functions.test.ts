import { describe, expect, test } from "bun:test"
import * as AST from "../src/ast"
import { generateTypeScript } from "../src/codegen"
import { Parser } from "../src/parser"
import { TypeInferenceSystem } from "../src/type-inference"

describe("Generic Functions", () => {
  test("should parse simple generic function", () => {
    const code = `fn identity<T> x: T -> T = x`

    const parser = new Parser(code)
    const parseResult = parser.parse()

    expect(parseResult.statements).toHaveLength(1)
    const funcDecl = parseResult.statements![0] as AST.FunctionDeclaration

    expect(funcDecl.kind).toBe("FunctionDeclaration")
    expect(funcDecl.name).toBe("identity")
    expect(funcDecl.typeParameters).toHaveLength(1)
    expect(funcDecl.typeParameters![0].name).toBe("T")
    expect(funcDecl.parameters).toHaveLength(1)
    expect(funcDecl.parameters[0].name).toBe("x")
  })

  test("should parse generic function with multiple type parameters", () => {
    const code = `fn const<A, B> a: A -> b: B -> A = a`

    const parser = new Parser(code)
    const parseResult = parser.parse()

    const funcDecl = parseResult.statements![0] as AST.FunctionDeclaration

    expect(funcDecl.typeParameters).toHaveLength(2)
    expect(funcDecl.typeParameters![0].name).toBe("A")
    expect(funcDecl.typeParameters![1].name).toBe("B")
  })

  test("should infer types for generic function calls", () => {
    const code = `
fn identity<T> x: T -> T = x

let intResult = identity 42
let stringResult = identity "hello"
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements!)

    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(program)

    expect(result.errors).toHaveLength(0)

    // intResult should be Int
    const intResultStmt = parseResult.statements![1] as AST.VariableDeclaration
    const intResultType = result.nodeTypeMap.get(intResultStmt)
    expect(
      typeInference.typeToString(result.substitution.apply(intResultType!))
    ).toBe("Int")

    // stringResult should be String
    const stringResultStmt =
      parseResult.statements![2] as AST.VariableDeclaration
    const stringResultType = result.nodeTypeMap.get(stringResultStmt)
    expect(
      typeInference.typeToString(result.substitution.apply(stringResultType!))
    ).toBe("String")
  })

  test("should generate correct TypeScript for generic functions", () => {
    const code = `fn identity<T> x: T -> T = x`

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const generated = generateTypeScript(parseResult.statements || [])

    // TypeScriptジェネリクス構文が含まれているかチェック
    expect(generated).toContain("<T>")
    expect(generated).toContain("identity")
    expect(generated).toContain("return x;")
  })

  test("should handle generic function with constraints", () => {
    const code = `fn const2<T, U> x: T -> y: U -> T = x`

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const funcDecl = parseResult.statements![0] as AST.FunctionDeclaration

    expect(funcDecl.typeParameters).toHaveLength(2)
    expect(funcDecl.typeParameters![0].name).toBe("T")
    expect(funcDecl.typeParameters![1].name).toBe("U")
  })

  test("should handle nested generic function calls", () => {
    const code = `
fn identity<T> x: T -> T = x
fn const<A, B> a: A -> b: B -> A = a

let result = const (identity 42) "ignored"
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements!)

    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(program)

    expect(result.errors).toHaveLength(0)

    // result should be Int
    const resultStmt = parseResult.statements![2] as AST.VariableDeclaration
    const resultType = result.nodeTypeMap.get(resultStmt)
    expect(
      typeInference.typeToString(result.substitution.apply(resultType!))
    ).toBe("Int")
  })

  test("should handle higher-order generic functions", () => {
    const code = `
fn twice<T> f: (T -> T) -> x: T -> T = f (f x)

let addOne = \\x -> x + 1
let result = twice addOne 5
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements!)

    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(program)

    expect(result.errors).toHaveLength(0)

    // result should be Int
    const resultStmt = parseResult.statements![2] as AST.VariableDeclaration
    const resultType = result.nodeTypeMap.get(resultStmt)
    expect(
      typeInference.typeToString(result.substitution.apply(resultType!))
    ).toBe("Int")
  })

  test("should parse explicit type arguments in function calls", () => {
    const code = `
fn identity<T> x: T -> T = x
let result = identity<String> "hello"
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()

    expect(parseResult.statements).toHaveLength(2)
    const varDecl = parseResult.statements![1] as AST.VariableDeclaration

    // Function call with explicit type arguments
    expect(varDecl.initializer.kind).toBe("FunctionCall")
    const funcCall = varDecl.initializer as AST.FunctionCall
    expect(funcCall.function.kind).toBe("Identifier")

    // Check for type arguments in function call
    expect(funcCall.typeArguments).toBeDefined()
    expect(funcCall.typeArguments).toHaveLength(1)
    expect(funcCall.typeArguments![0].kind).toBe("PrimitiveType")
    expect((funcCall.typeArguments![0] as AST.PrimitiveType).name).toBe(
      "String"
    )
  })

  test("should handle type annotations with generic types", () => {
    const code = `
fn identity<T> x: T -> T = x
let result: String = identity<String> "hello"
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()

    expect(parseResult.statements).toHaveLength(2)
    const varDecl = parseResult.statements![1] as AST.VariableDeclaration

    // Variable should have explicit type annotation
    expect(varDecl.type).toBeDefined()
    expect(varDecl.type!.kind).toBe("PrimitiveType")
  })

  test("should generate TypeScript with explicit type arguments", () => {
    const code = `
fn identity<T> x: T -> T = x
let result = identity<String> "hello"
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const generated = generateTypeScript(parseResult.statements || [])

    // TypeScript出力に型引数が含まれることを確認
    expect(generated).toContain("identity<string>")
    expect(generated).toContain("<T>")
  })
})
