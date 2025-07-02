import { describe, test, expect } from "bun:test"
import { compileSeseragi } from "../src/main"

describe("Arithmetic Type System", () => {
  test("should handle float arithmetic in struct fields correctly", () => {
    const source = `
struct Point {
  x: Float,
  y: Float
}

fn distance p1: Point -> p2: Point -> Float {
  let dx = p2.x - p1.x
  let dy = p2.y - p1.y
  dx * dx + dy * dy
}

let p1 = Point { x: 3.0, y: 0.0 }
let p2 = Point { x: 0.0, y: 4.0 }
distance p1 p2
`
    const output = compileSeseragi(source)

    // プリミティブ型の演算は直接演算子を使用
    expect(output).toContain("(p2.x - p1.x)")
    expect(output).toContain("(p2.y - p1.y)")
    expect(output).toContain("((dx * dx) + (dy * dy))")

    // __dispatchOperatorを使わないこと
    expect(output).not.toContain("__dispatchOperator(p2.x")
  })

  test("should handle struct operator overloading correctly", () => {
    const source = `
struct Vec2 {
  x: Float,
  y: Float
}

impl Vec2 {
  operator + self: Vec2 -> other: Vec2 -> Vec2 {
    Vec2 { x: self.x + other.x, y: self.y + other.y }
  }
}

let v1 = Vec2 { x: 1.0, y: 2.0 }
let v2 = Vec2 { x: 3.0, y: 4.0 }
v1 + v2
`
    const output = compileSeseragi(source)

    // 構造体同士の演算は__dispatchOperatorを使用
    expect(output).toContain("__dispatchOperator(v1")

    // 構造体内部のFloat演算は直接演算子を使用
    expect(output).toContain("(self.x + other.x)")
    expect(output).toContain("(self.y + other.y)")
  })

  test("should handle basic arithmetic without dispatch", () => {
    const source = `
let x = 1 + 2
let y = 3.5 - 1.5
let z = 2 * 3
let w = 10 / 2
`
    const output = compileSeseragi(source)

    // 基本的な算術演算は直接演算子を使用
    expect(output).toContain("(1 + 2)")
    expect(output).toContain("(3.5 - 1.5)")
    expect(output).toContain("(2 * 3)")
    expect(output).toContain("(10 / 2)")

    // __dispatchOperatorを使わないこと
    expect(output).not.toContain("__dispatchOperator")
  })

  test("should generate dispatch helper when using structs", () => {
    const source = `
struct Point {
  x: Float,
  y: Float
}

let p = Point { x: 1.0, y: 2.0 }
p.x
`
    const output = compileSeseragi(source)

    // 構造体を使用している場合はディスパッチヘルパーを生成
    expect(output).toContain("__dispatchMethod")
    expect(output).toContain("__dispatchOperator")
  })

  test("should handle mixed arithmetic expressions correctly", () => {
    const source = `
let a = 1 + 2 * 3
let b = (4 - 2) / 2
let c = 10 % 3
`
    const output = compileSeseragi(source)

    // 優先順位を保持した演算子使用
    expect(output).toContain("(1 + (2 * 3))")
    expect(output).toContain("((4 - 2) / 2)")
    expect(output).toContain("(10 % 3)")
  })
})
