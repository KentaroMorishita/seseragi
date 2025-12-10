import { describe, expect, test } from "bun:test"
import * as AST from "../src/ast"
import { Parser } from "../src/parser"
import { infer } from "../src/inference/engine/infer"

describe("Polymorphism Tests", () => {
  test("should handle polymorphic lambda with multiple different type applications", () => {
    const code = `
let func = \\x -> \\y -> x + y

let hoge = func 1 2
let fuga = func "foo" "bar"

show hoge
show fuga
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements!)

    const result = infer(program)

    // 型推論エラーが発生しないことを確認
    expect(result.errors).toHaveLength(0)
  })

  test("should generalize lambda function types correctly", () => {
    const code = `
let identity = \\x -> x

let numId = identity 42
let strId = identity "hello"
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements!)

    const result = infer(program)

    expect(result.errors).toHaveLength(0)
  })

  test("should handle complex polymorphic scenarios", () => {
    const code = `
let compose = \\f -> \\g -> \\x -> f (g x)

let addOne = \\x -> x + 1
let toString = \\x -> show x

let combined = compose toString addOne
let result = combined 5
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements!)

    const result = infer(program)

    // 型推論エラーが発生しないことを確認
    expect(result.errors).toHaveLength(0)
  })

  test("should prevent premature type concretization", () => {
    const code = `
let func = \\x -> \\y -> x + y

// 最初にIntで使用
let intResult = func 1 2

// その後Stringで使用してもエラーにならないはず  
let stringResult = func "a" "b"
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements!)

    const result = infer(program)

    // この修正により、型推論エラーが発生しないはず
    expect(result.errors).toHaveLength(0)
  })
})
