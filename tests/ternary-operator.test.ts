/**
 * 三項演算子のテスト
 */

import { describe, expect, test } from "bun:test"
import * as AST from "../src/ast"
import { generateTypeScript } from "../src/codegen"
import { infer } from "../src/inference/engine/infer"
import { Parser } from "../src/parser"

describe("Ternary Operator", () => {
  test("should parse simple ternary expression", () => {
    const parser = new Parser("x > 0 ? 1 : -1")
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const stmt = result.statements![0]
    expect(stmt).toBeInstanceOf(AST.ExpressionStatement)

    const expr = (stmt as AST.ExpressionStatement).expression
    expect(expr).toBeInstanceOf(AST.TernaryExpression)

    const ternary = expr as AST.TernaryExpression
    expect(ternary.condition).toBeInstanceOf(AST.BinaryOperation)
    expect(ternary.trueExpression).toBeInstanceOf(AST.Literal)
    expect(ternary.falseExpression).toBeInstanceOf(AST.Literal)
  })

  test.skip("should parse nested ternary expressions", () => {
    // TODO: ネストした三項演算子の優先順位を修正
    const parser = new Parser("x > 0 ? y > 0 ? 1 : 2 : -1")
    const result = parser.parse()

    if (result.errors.length > 0) {
      console.log(
        "Parse errors:",
        result.errors.map((e) => e.message)
      )
    }
    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const stmt = result.statements![0]
    expect(stmt).toBeInstanceOf(AST.ExpressionStatement)

    const expr = (stmt as AST.ExpressionStatement).expression
    expect(expr).toBeInstanceOf(AST.TernaryExpression)

    const ternary = expr as AST.TernaryExpression
    expect(ternary.trueExpression).toBeInstanceOf(AST.TernaryExpression)
  })

  test("should parse ternary with function calls", () => {
    const parser = new Parser("isValid x ? process x : defaultValue")
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const stmt = result.statements![0]
    expect(stmt).toBeInstanceOf(AST.ExpressionStatement)

    const expr = (stmt as AST.ExpressionStatement).expression
    expect(expr).toBeInstanceOf(AST.TernaryExpression)

    const ternary = expr as AST.TernaryExpression
    expect(ternary.condition).toBeInstanceOf(AST.FunctionApplication)
    expect(ternary.trueExpression).toBeInstanceOf(AST.FunctionApplication)
    expect(ternary.falseExpression).toBeInstanceOf(AST.Identifier)
  })

  test("should infer correct types for ternary expression", () => {
    const parser = new Parser(`
      fn test x: Int -> String = x > 0 ? "positive" : "negative"
    `)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)

    const inferenceResult = infer(new AST.Program(result.statements!, 1, 1))

    expect(inferenceResult.errors).toHaveLength(0)
  })

  test("should generate correct TypeScript code", () => {
    const parser = new Parser(`
      fn test x: Int -> String = x > 0 ? "positive" : "negative"
    `)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)

    const code = generateTypeScript(result.statements!, {})

    expect(code).toContain(
      '(__dispatchOperator(x, ">", 0) ? "positive" : "negative")'
    )
  })

  test("should handle ternary in function body", () => {
    const parser = new Parser(`
      fn abs x: Int -> Int = x < 0 ? -x : x
    `)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)

    const code = generateTypeScript(result.statements!, {})

    expect(code).toContain('(__dispatchOperator(x, "<", 0) ? (-x) : x)')
  })

  test("should handle complex ternary expressions", () => {
    const parser = new Parser(`
      fn classify x: Int -> String =
        x > 0 ? "positive" : x < 0 ? "negative" : "zero"
    `)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)

    const code = generateTypeScript(result.statements!, {})

    expect(code).toContain(
      '(__dispatchOperator(x, ">", 0) ? "positive" : (__dispatchOperator(x, "<", 0) ? "negative" : "zero"))'
    )
  })

  test("should handle boolean ternary", () => {
    const parser = new Parser(`
      fn toggle flag: Bool -> Bool = flag ? False : True
    `)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)

    const inferenceResult = infer(new AST.Program(result.statements!, 1, 1))

    expect(inferenceResult.errors).toHaveLength(0)
  })

  test("should enforce type consistency in ternary branches", () => {
    const parser = new Parser(`
      fn invalid x: Int -> String = x > 0 ? "text" : 42
    `)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)

    const inferenceResult = infer(new AST.Program(result.statements!, 1, 1))

    // 型エラーが発生するはず（String vs Int）
    expect(inferenceResult.errors.length).toBeGreaterThan(0)
  })

  test("should enforce boolean condition in ternary", () => {
    const parser = new Parser(`
      fn invalid x: Int -> String = x ? "text" : "other"
    `)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)

    const inferenceResult = infer(new AST.Program(result.statements!, 1, 1))

    // 型エラーが発生するはず（Int条件 vs Bool要求）
    expect(inferenceResult.errors.length).toBeGreaterThan(0)
  })

  test("should handle ternary with cons operator using parentheses", () => {
    const parser = new Parser(`
      let result = True ? (1 : [2, 3]) : []
    `)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(1)

    const stmt = result.statements![0]
    expect(stmt).toBeInstanceOf(AST.VariableDeclaration)

    const varDecl = stmt as AST.VariableDeclaration
    expect(varDecl.initializer).toBeInstanceOf(AST.TernaryExpression)

    const ternary = varDecl.initializer as AST.TernaryExpression
    // trueExpressionは括弧内のcons演算子
    expect(ternary.trueExpression).toBeInstanceOf(AST.BinaryOperation)
    const trueBranch = ternary.trueExpression as AST.BinaryOperation
    expect(trueBranch.operator).toBe(":")
  })

  test("should work with dollar operator", () => {
    const parser = new Parser(`
      fn test x: Int -> String = x > 0 ? "positive" : "negative"
      show $ test 5
    `)
    const result = parser.parse()

    expect(result.errors).toHaveLength(0)
    expect(result.statements).toHaveLength(2)

    // 2つ目のステートメントがshow $ test 5
    const stmt = result.statements![1]
    expect(stmt).toBeInstanceOf(AST.ExpressionStatement)

    const expr = (stmt as AST.ExpressionStatement).expression
    expect(expr).toBeInstanceOf(AST.FunctionApplicationOperator)

    const app = expr as AST.FunctionApplicationOperator
    expect(app.left).toBeInstanceOf(AST.Identifier)
    expect((app.left as AST.Identifier).name).toBe("show")

    // right は $ 演算子の右側（test 5）
    expect(app.right).toBeInstanceOf(AST.FunctionApplication)
  })
})
