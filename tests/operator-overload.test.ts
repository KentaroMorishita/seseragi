import { describe, expect, test } from "bun:test"
import * as AST from "../src/ast"
import { Parser } from "../src/parser"
import { TypeInferenceSystem } from "../src/type-inference"

describe("Operator Overload Type Inference Tests", () => {
  test("should infer correct return type for struct operator overload", () => {
    const code = `
struct User {
  name: String,
  age: Int,
  amount: Int
}

impl User {
  operator + self -> other -> Int {
    self.amount + other.amount
  }
}

let u1 = User { name: "foo", age: 20, amount: 10 }
let u2 = User { name: "bar", age: 20, amount: 20 }

let result = u1 + u2
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements!)

    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(program)

    // 型推論エラーが発生しないことを確認
    expect(result.errors).toHaveLength(0)
  })

  test("should handle polymorphic function with operator overload", () => {
    const code = `
struct Point {
  x: Int,
  y: Int
}

impl Point {
  operator + self -> other -> Point {
    let x = self.x + other.x
    let y = self.y + other.y
    Point { x, y }
  }
}

let func = \\x -> \\y -> x + y

let p1 = Point { x: 1, y: 2 }
let p2 = Point { x: 3, y: 4 }

let pointResult = func p1 p2
let intResult = func 1 2
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements!)

    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(program)

    // 型推論エラーが発生しないことを確認
    expect(result.errors).toHaveLength(0)
  })

  test("should handle different operators with different return types", () => {
    const code = `
struct Vector {
  x: Int,
  y: Int
}

impl Vector {
  operator + self -> other -> Vector {
    Vector { x: self.x + other.x, y: self.y + other.y }
  }
  
  operator * self -> other -> Int {
    self.x * other.x + self.y * other.y
  }
}

let v1 = Vector { x: 1, y: 2 }
let v2 = Vector { x: 3, y: 4 }

let vectorSum = v1 + v2
let dotProduct = v1 * v2
    `

    const parser = new Parser(code)
    const parseResult = parser.parse()
    const program = new AST.Program(parseResult.statements!)

    const typeInference = new TypeInferenceSystem()
    const result = typeInference.infer(program)

    // 型推論エラーが発生しないことを確認
    expect(result.errors).toHaveLength(0)
  })
})
