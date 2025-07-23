import { describe, expect, test } from "bun:test"
import * as AST from "../src/ast"
import { Parser } from "../src/parser"
import { TypeInferenceSystem } from "../src/type-inference"

describe("Generic Type Alias Usage", () => {
  test("should resolve simple generic type alias in variable declaration", () => {
    const code = `
type Box<T> = T
let x: Box<Int> = 42
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements!)

    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(program)

    // 修正後は成功するはず
    console.log("Errors:", result.errors)
    if (result.errors.length > 0) {
      console.log("First error:", result.errors[0].message)
    }
    expect(result.errors).toHaveLength(0)
  })

  test("should resolve generic type alias with multiple parameters", () => {
    const code = `
type Pair<A, B> = (A, B)
let p: Pair<Int, String> = (42, "hello")
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements!)

    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(program)

    // 修正後は成功するはず
    console.log("Pair errors:", result.errors)
    expect(result.errors).toHaveLength(0)
  })

  test("should resolve generic record type alias", () => {
    const code = `
type Container<T> = { value: T, metadata: String }
let c: Container<Int> = { value: 42, metadata: "number" }
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements!)

    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(program)

    // 修正後は成功するはず
    console.log("Pair errors:", result.errors)
    expect(result.errors).toHaveLength(0)
  })
})
