import { describe, test, expect } from "bun:test"
import { Parser } from "../src/parser"
import { TypeInferenceSystem } from "../src/type-inference"
import * as AST from "../src/ast"

describe("Type Assertion Inference", () => {
  test("変数の型アサーション推論確認", () => {
    const source = `
let x = 42 as String
`

    const parser = new Parser(source)
    const parseResult = parser.parse()

    expect(parseResult.errors).toHaveLength(0)
    expect(parseResult.statements).toHaveLength(1)

    const program = new AST.Program(parseResult.statements || [])
    const typeInference = new TypeInferenceSystem()
    const typeResult = typeInference.infer(program)

    expect(typeResult.errors).toHaveLength(0)

    // 変数宣言を取得
    const varDecl = parseResult.statements![0] as AST.VariableDeclaration
    expect(varDecl.kind).toBe("VariableDeclaration")
    expect(varDecl.name).toBe("x")

    // 型アサーション式を取得
    const typeAssertion = varDecl.initializer as AST.TypeAssertion
    expect(typeAssertion.kind).toBe("TypeAssertion")
    expect(typeAssertion.assertionKind).toBe("as")

    // TypeAssertionの型を確認
    const assertionType = typeResult.nodeTypeMap.get(typeAssertion)
    console.log("TypeAssertion type:", assertionType)
    expect(assertionType).toBeDefined()
    expect((assertionType as AST.PrimitiveType).name).toBe("String")

    // 元の式の型も確認
    const originalExprType = typeResult.nodeTypeMap.get(
      typeAssertion.expression
    )
    console.log("Original expression type:", originalExprType)
    expect(originalExprType).toBeDefined()
    expect((originalExprType as AST.PrimitiveType).name).toBe("Int")

    // 変数宣言全体の型も確認（型アサーション結果と同じになるべき）
    const varDeclType = typeResult.nodeTypeMap.get(varDecl.initializer)
    console.log("Variable declaration initializer type:", varDeclType)
    expect(varDeclType).toBeDefined()
    expect((varDeclType as AST.PrimitiveType).name).toBe("String")
  })

  test("型アサーションの型がnodeTypesに記録されているか確認", () => {
    const source = `
let y = "hello" as Int
`

    const parser = new Parser(source)
    const parseResult = parser.parse()

    const program = new AST.Program(parseResult.statements || [])
    const typeInference = new TypeInferenceSystem()
    const typeResult = typeInference.infer(program)

    expect(typeResult.errors).toHaveLength(0)

    // 変数宣言を取得
    const varDecl = parseResult.statements![0] as AST.VariableDeclaration
    const typeAssertion = varDecl.initializer as AST.TypeAssertion

    // TypeAssertionの型を確認
    const assertionType = typeResult.nodeTypeMap.get(typeAssertion)
    expect(assertionType).toBeDefined()
    expect((assertionType as AST.PrimitiveType).name).toBe("Int")

    // 変数の型も確認（型アサーション結果と同じになるべき）
    const varDeclType = typeResult.nodeTypeMap.get(varDecl.initializer)
    expect(varDeclType).toBeDefined()
    console.log("Variable y type:", varDeclType)
  })

  test("ネストした型アサーションの型推論", () => {
    const source = `
let z = (42 as Float) as String
`

    const parser = new Parser(source)
    const parseResult = parser.parse()

    const program = new AST.Program(parseResult.statements || [])
    const typeInference = new TypeInferenceSystem()
    const typeResult = typeInference.infer(program)

    expect(typeResult.errors).toHaveLength(0)

    // 変数宣言を取得
    const varDecl = parseResult.statements![0] as AST.VariableDeclaration
    const outerAssertion = varDecl.initializer as AST.TypeAssertion
    const innerAssertion = outerAssertion.expression as AST.TypeAssertion

    // 外側の型アサーション: String
    const outerType = typeResult.nodeTypeMap.get(outerAssertion)
    expect(outerType).toBeDefined()
    expect((outerType as AST.PrimitiveType).name).toBe("String")

    // 内側の型アサーション: Float
    const innerType = typeResult.nodeTypeMap.get(innerAssertion)
    expect(innerType).toBeDefined()
    expect((innerType as AST.PrimitiveType).name).toBe("Float")

    // 変数の型は最終的にString
    const varDeclType = typeResult.nodeTypeMap.get(varDecl.initializer)
    expect(varDeclType).toBeDefined()
    console.log("Variable z type:", varDeclType)
  })
})
