import { test, expect } from "bun:test"
import { Parser } from "./src/parser"
import { generateTypeScript } from "./src/codegen"

test("ADT with arguments should generate constructor functions", () => {
  const source = `
type Shape = Circle Float | Rectangle Float Float

let circle = Circle 5.0
let rect = Rectangle 10.0 20.0
`

  const parser = new Parser(source)
  const ast = parser.parse()
  
  // パースエラーがないことを確認
  expect(ast.errors).toHaveLength(0)
  
  // 3つのステートメントがあることを確認（type定義1つ、let変数2つ）
  expect(ast.statements).toHaveLength(3)
  
  // TypeScript生成
  const tsCode = generateTypeScript(ast.statements)
  console.log("Generated TypeScript:")
  console.log(tsCode)
  
  // 型定義とコンストラクタ関数が生成されることを確認
  expect(tsCode).toContain("type Shape =")
  expect(tsCode).toContain("const Circle =")
  expect(tsCode).toContain("const Rectangle =")
})