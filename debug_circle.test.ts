import { test, expect } from "bun:test"
import { Parser } from "./src/parser"
import { TypeInferenceSystem } from "./src/type-inference"

test("Debug Circle stack overflow", () => {
  const source = `
type Shape = Circle Float | Rectangle Float Float

let circle = Circle 5.0
`

  const parser = new Parser(source)
  const ast = parser.parse()
  
  console.log("Parse errors:", ast.errors)
  expect(ast.errors).toHaveLength(0)
  
  const typeInference = new TypeInferenceSystem()
  try {
    const result = typeInference.infer(ast)
    console.log("Type inference completed successfully")
    console.log("Errors:", result.errors)
  } catch (error) {
    console.log("Type inference error:", error.message)
    if (error.stack) {
      console.log("Stack trace:", error.stack.substring(0, 1000))
    }
    throw error
  }
})

test("Debug recursive ADT", () => {
  const source = `
type Tree = Leaf Int | Node Tree Tree

let tree = Node (Leaf 1) (Leaf 2)
`

  const parser = new Parser(source)
  const ast = parser.parse()
  
  console.log("Parse errors:", ast.errors)
  expect(ast.errors).toHaveLength(0)
  
  const typeInference = new TypeInferenceSystem()
  try {
    const result = typeInference.infer(ast)
    console.log("Type inference completed successfully")
    console.log("Errors:", result.errors)
  } catch (error) {
    console.log("Type inference error:", error.message)
    if (error.stack) {
      console.log("Stack trace:", error.stack.substring(0, 2000))
    }
    throw error
  }
})

test("Debug self-referential ADT", () => {
  const source = `
type Shape = Circle Float

let f = \\x -> Circle x
let circle = f 5.0
`

  const parser = new Parser(source)
  const ast = parser.parse()
  
  console.log("Parse errors:", ast.errors)
  expect(ast.errors).toHaveLength(0)
  
  const typeInference = new TypeInferenceSystem()
  try {
    const result = typeInference.infer(ast)
    console.log("Type inference completed successfully")
    console.log("Errors:", result.errors)
  } catch (error) {
    console.log("Type inference error:", error.message)
    if (error.stack) {
      console.log("Stack trace:", error.stack.substring(0, 2000))
    }
    throw error
  }
})