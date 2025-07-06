import { test, expect } from "bun:test"
import { Parser } from "./src/parser"
import { generateTypeScript } from "./src/codegen"

test("Simple ADT test without match", () => {
  const source = `
type Shape = Circle Float | Rectangle Float Float

let circle = Circle 5.0
let rect = Rectangle 10.0 20.0
`

  const parser = new Parser(source)
  const ast = parser.parse()
  
  console.log("パースエラー:", ast.errors)
  expect(ast.errors).toHaveLength(0)
  
  const tsCode = generateTypeScript(ast.statements)
  console.log("Generated TypeScript:")
  console.log(tsCode)
  
  expect(tsCode).toContain("type Shape =")
  expect(tsCode).toContain("const Circle =")
  expect(tsCode).toContain("const Rectangle =")
})

test("Name conflict detection", () => {
  const source1 = `
type Test = Test
struct Test { x: Int }
`
  
  const source2 = `
struct Test { x: Int }
type Test = Test
`

  const parser1 = new Parser(source1)
  const ast1 = parser1.parse()
  
  const parser2 = new Parser(source2)
  const ast2 = parser2.parse()
  
  console.log("Conflict test 1 errors:", ast1.errors)
  console.log("Conflict test 2 errors:", ast2.errors)
  
  // どちらかでエラーが発生することを確認
  const hasError1 = ast1.errors.length > 0
  const hasError2 = ast2.errors.length > 0
  
  expect(hasError1 || hasError2).toBe(true)
})