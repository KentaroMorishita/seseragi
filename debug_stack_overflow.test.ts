import { test, expect } from "bun:test"
import { Parser } from "./src/parser"
import { TypeInferenceSystem } from "./src/type-inference"

test("Debug stack overflow - minimal case", () => {
  const source = `
type Tree = Leaf | Node Tree

let node = Node Leaf
`

  const parser = new Parser(source)
  const ast = parser.parse()
  
  console.log("Parse errors:", ast.errors)
  expect(ast.errors).toHaveLength(0)
  
  const typeInference = new TypeInferenceSystem()
  try {
    console.log("Starting type inference...")
    const result = typeInference.infer(ast)
    console.log("Type inference completed")
    console.log("Errors:", result.errors.map(e => e.message))
  } catch (error) {
    console.log("Type inference error:", error.message)
    console.log("Error type:", error.constructor.name)
    if (error.stack) {
      // Look for the specific method calls causing the stack overflow
      const stackLines = error.stack.split('\n').slice(0, 20)
      console.log("Stack trace (first 20 lines):")
      stackLines.forEach((line, i) => console.log(`${i}: ${line}`))
    }
    
    // Don't rethrow, just log for analysis
    // throw error
  }
})