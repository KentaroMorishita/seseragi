import { describe, test, expect } from "bun:test"
import * as AST from "../src/ast"
import { TypeInferenceSystem, TypeInferenceError } from "../src/type-inference"

describe("Plus operator type inference", () => {
  test("String + String works correctly", () => {
    const system = new TypeInferenceSystem()
    
    // "hello" + " world"
    const leftLiteral = new AST.Literal("hello", "string", 1, 1)
    const rightLiteral = new AST.Literal(" world", "string", 1, 11)
    const binOp = new AST.BinaryOperation(leftLiteral, "+", rightLiteral, 1, 8)
    
    const program = new AST.Program([
      new AST.ExpressionStatement(binOp, 1, 1)
    ], 1, 1)
    
    const result = system.infer(program)
    
    expect(result.errors).toHaveLength(0)
    const exprType = result.nodeTypeMap.get(binOp)
    expect(exprType?.kind).toBe("PrimitiveType")
    expect((exprType as AST.PrimitiveType).name).toBe("String")
  })
  
  test("Int + Int works correctly", () => {
    const system = new TypeInferenceSystem()
    
    // 1 + 2
    const leftLiteral = new AST.Literal(1, "integer", 1, 1)
    const rightLiteral = new AST.Literal(2, "integer", 1, 5)
    const binOp = new AST.BinaryOperation(leftLiteral, "+", rightLiteral, 1, 3)
    
    const program = new AST.Program([
      new AST.ExpressionStatement(binOp, 1, 1)
    ], 1, 1)
    
    const result = system.infer(program)
    
    expect(result.errors).toHaveLength(0)
    const exprType = result.nodeTypeMap.get(binOp)
    expect(exprType?.kind).toBe("PrimitiveType")
    expect((exprType as AST.PrimitiveType).name).toBe("Int")
  })
  
  test("String + Int fails with clear error", () => {
    const system = new TypeInferenceSystem()
    
    // "hello" + 42
    const leftLiteral = new AST.Literal("hello", "string", 1, 1)
    const rightLiteral = new AST.Literal(42, "integer", 1, 11)
    const binOp = new AST.BinaryOperation(leftLiteral, "+", rightLiteral, 1, 9)
    
    const program = new AST.Program([
      new AST.ExpressionStatement(binOp, 1, 1)
    ], 1, 1)
    
    const result = system.infer(program)
    
    expect(result.errors).toHaveLength(1)
    expect(result.errors[0].message).toContain("Cannot unify")
    expect(result.errors[0].message).toContain("String")
    expect(result.errors[0].message).toContain("Int")
    expect(result.errors[0].context).toBe("Binary operation + operands must have same type")
  })
  
  test("Int + String fails with clear error", () => {
    const system = new TypeInferenceSystem()
    
    // 42 + "hello"
    const leftLiteral = new AST.Literal(42, "integer", 1, 1)
    const rightLiteral = new AST.Literal("hello", "string", 1, 6)
    const binOp = new AST.BinaryOperation(leftLiteral, "+", rightLiteral, 1, 4)
    
    const program = new AST.Program([
      new AST.ExpressionStatement(binOp, 1, 1)
    ], 1, 1)
    
    const result = system.infer(program)
    
    expect(result.errors).toHaveLength(1)
    expect(result.errors[0].message).toContain("Cannot unify")
    expect(result.errors[0].message).toContain("Int")
    expect(result.errors[0].message).toContain("String")
    expect(result.errors[0].context).toBe("Binary operation + operands must have same type")
  })
})