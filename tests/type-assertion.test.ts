import { describe, test, expect } from "bun:test"
import { Parser } from "../src/parser"
import { generateTypeScript } from "../src/codegen"
import { TypeInferenceSystem } from "../src/type-inference"
import * as AST from "../src/ast"

describe("Type Assertion", () => {
  test("基本的な型アサーション", () => {
    const source = `
let x = 42 as String
`

    const parser = new Parser(source)
    const parseResult = parser.parse()

    expect(parseResult.errors).toHaveLength(0)
    expect(parseResult.statements).toHaveLength(1)

    const generated = generateTypeScript(parseResult.statements || [])
    expect(generated).toContain("(42 as string)")
  })

  test("複雑な式での型アサーション", () => {
    const source = `
let result = (1 + 2) as String
`

    const parser = new Parser(source)
    const parseResult = parser.parse()

    expect(parseResult.errors).toHaveLength(0)

    const generated = generateTypeScript(parseResult.statements || [])
    expect(generated).toContain('(__dispatchOperator(1, "+", 2) as string)')
  })

  test("メソッドチェーンでの型アサーション", () => {
    const source = `
let result = obj.method() as String
`

    const parser = new Parser(source)
    const parseResult = parser.parse()

    expect(parseResult.errors).toHaveLength(0)

    const generated = generateTypeScript(parseResult.statements || [])
    expect(generated).toContain("(obj.method() as string)")
  })

  test("型推論での型アサーション処理", () => {
    const source = `
let x = 42 as String
`

    const parser = new Parser(source)
    const parseResult = parser.parse()

    const program = new AST.Program(parseResult.statements || [])
    const typeInference = new TypeInferenceSystem()
    const typeResult = typeInference.infer(program)

    // 型アサーションにより型エラーが抑制されることを確認
    expect(typeResult.errors).toHaveLength(0)
  })

  test("関数の戻り値での型アサーション", () => {
    const source = `
let value = 123 as String
`

    const parser = new Parser(source)
    const parseResult = parser.parse()

    expect(parseResult.errors).toHaveLength(0)

    const generated = generateTypeScript(parseResult.statements || [])
    expect(generated).toContain("(123 as string)")
  })

  test("ネストした型アサーション", () => {
    const source = `
let x = (42 as Float) as String
`

    const parser = new Parser(source)
    const parseResult = parser.parse()

    expect(parseResult.errors).toHaveLength(0)

    const generated = generateTypeScript(parseResult.statements || [])
    expect(generated).toContain("((42 as number) as string)")
  })

  test("配列要素での型アサーション", () => {
    const source = `
let arr = [1 as String, 2 as String]
`

    const parser = new Parser(source)
    const parseResult = parser.parse()

    expect(parseResult.errors).toHaveLength(0)

    const generated = generateTypeScript(parseResult.statements || [])
    expect(generated).toContain("(1 as string)")
    expect(generated).toContain("(2 as string)")
  })
})
