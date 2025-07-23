import { describe, expect, test } from "bun:test"
import * as AST from "../src/ast"
import { Parser } from "../src/parser"
import { TypeChecker } from "../src/typechecker"

function checkTypes(code: string) {
  const parser = new Parser(code)
  const parseResult = parser.parse()
  const typeChecker = new TypeChecker()
  const program = new AST.Program(parseResult.statements || [])
  return typeChecker.check(program)
}

describe("TypeChecker - Basic Tests", () => {
  test("should type check simple literals", () => {
    const errors = checkTypes(`
      let x = 42
      let y = "hello"
      let z = True
      let w = 3.14
    `)
    expect(errors).toHaveLength(0)
  })

  test("should detect type mismatch", () => {
    const errors = checkTypes(`
      let x: String = 42
    `)
    expect(errors).toHaveLength(1)
    expect(errors[0].message).toBe("Variable 'x' type mismatch")
  })

  test("should type check simple functions", () => {
    const errors = checkTypes(`
      fn add x: Int -> y: Int -> Int = x + y
    `)
    expect(errors).toHaveLength(0)
  })

  test("should detect function return type mismatch", () => {
    const errors = checkTypes(`
      fn bad x: Int -> String = x + 1
    `)
    expect(errors).toHaveLength(1)
    expect(errors[0].message).toBe("Function 'bad' return type mismatch")
  })

  test("should type check binary operations", () => {
    const errors = checkTypes(`
      let x = 1 + 2
      let y = 3.14 * 2.0
    `)
    expect(errors).toHaveLength(0)
  })

  test("should detect binary operation type error", () => {
    const errors = checkTypes(`
      let x = "hello" + 42
    `)
    expect(errors).toHaveLength(1)
    expect(errors[0].message).toContain("Invalid operands")
  })

  test("should type check Maybe constructors", () => {
    const errors = checkTypes(`
      let x = Just 42
      let y = Nothing
    `)
    expect(errors).toHaveLength(0)
  })

  test("should type check Either constructors", () => {
    const errors = checkTypes(`
      let x = Left "error"
      let y = Right 42
    `)
    expect(errors).toHaveLength(0)
  })

  test("should type check builtin functions", () => {
    const errors = checkTypes(`
      print "Hello"
      putStrLn "World"
    `)
    expect(errors).toHaveLength(0)
  })

  test("should accept any type for print", () => {
    const errors = checkTypes(`
      print 42
      print "hello"
      print True
    `)
    expect(errors).toHaveLength(0)
  })
})
