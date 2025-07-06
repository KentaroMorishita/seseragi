import { test, expect } from "bun:test"
import { Parser } from "./src/parser"
import { generateTypeScript } from "./src/codegen"

test("Complete ADT functionality test", () => {
  const source = `
type Shape = Circle Float | Rectangle Float Float | Triangle Float Float Float

type Animal = Cat | Dog String

let circle = Circle 5.0
let rect = Rectangle 10.0 20.0
let triangle = Triangle 3.0 4.0 5.0
let cat = Cat
let dog = Dog "Buddy"

match circle {
  Circle radius -> radius
  Rectangle w h -> w * h
  Triangle a b c -> a + b + c
}
`

  const parser = new Parser(source)
  const ast = parser.parse()
  
  console.log("パースエラー:", ast.errors)
  expect(ast.errors).toHaveLength(0)
  
  const tsCode = generateTypeScript(ast.statements)
  console.log("Generated TypeScript:")
  console.log(tsCode)
  
  // ADT型定義とコンストラクタが生成される
  expect(tsCode).toContain("type Shape =")
  expect(tsCode).toContain("type Animal =")
  expect(tsCode).toContain("const Circle =")
  expect(tsCode).toContain("const Rectangle =")
  expect(tsCode).toContain("const Triangle =")
  expect(tsCode).toContain("const Cat =")
  expect(tsCode).toContain("const Dog =")
})

test("ADT/struct name conflict detection", () => {
  const source = `
type Circle = Circle Float
struct Circle { radius: Float }
`

  const parser = new Parser(source)
  const ast = parser.parse()
  
  // エラーが発生することを確認
  expect(ast.errors.length).toBeGreaterThan(0)
  expect(ast.errors[0].message).toContain("conflicts")
})