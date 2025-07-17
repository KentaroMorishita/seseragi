import { test, expect } from "bun:test"
import { compileSeseragi } from "./test-utils"

test("Ternary operator should work with union types", () => {
  const input = `
type AB = String | Int
fn test condition: Bool -> AB = condition ? 10 : "hello"
`

  // コンパイルが成功することを確認
  const result = compileSeseragi(input)

  // 生成されたコードに正しい型と式が含まれていることを確認
  expect(result).toContain("type AB = (string | number)")
  expect(result).toContain('condition ? 10 : "hello"')
})

test("Ternary operator should work with same types", () => {
  const input = `
fn test condition: Bool -> Int = condition ? 10 : 20
`

  // コンパイルが成功することを確認
  const result = compileSeseragi(input)

  // 生成されたコードに正しい型と式が含まれていることを確認
  expect(result).toContain("condition ? 10 : 20")
})

test("Ternary operator with nested expression", () => {
  const input = `
type Result = String | Int
fn complexTest x: Int -> Result = 
  x > 5 ? (x > 10 ? "big" : x) : "small"
`

  // コンパイルが成功することを確認
  const result = compileSeseragi(input)

  // 生成されたコードに正しい型と式が含まれていることを確認
  expect(result).toContain("type Result = (string | number)")
  expect(result).toContain("(x > 5) ?")
  expect(result).toContain("(x > 10) ?")
})
